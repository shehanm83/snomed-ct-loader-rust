//! In-memory SNOMED CT data store.
//!
//! Provides efficient storage and lookup for parsed RF2 data.
//! Includes parallel parsing support via rayon for maximum performance.
//!
//! ## Optimized Hierarchy Queries
//!
//! After loading data, call `build_transitive_closure()` to enable O(1)
//! ancestor and descendant lookups instead of BFS traversal.
//!
//! ```ignore
//! let mut store = SnomedStore::new();
//! store.load_all(&files)?;
//! store.build_transitive_closure(); // One-time O(n*d) build
//!
//! // Now these are O(1) lookups:
//! let ancestors = store.get_all_ancestors(concept_id);
//! let is_descendant = store.is_descendant_of(child, ancestor);
//! ```

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use snomed_ecl_optimizer::TransitiveClosure;
use snomed_types::{
    Rf2AssociationRefsetMember, Rf2Concept, Rf2ConcreteRelationship, Rf2Description,
    Rf2LanguageRefsetMember, Rf2OwlExpression, Rf2Relationship, Rf2SimpleRefsetMember, SctId,
};

use crate::description::DescriptionFilter;
use crate::mrcm::MrcmStore;
use crate::parser::{parse, Rf2Parser};
use crate::relationship::RelationshipFilter;
use crate::types::{DescriptionConfig, Rf2Config, Rf2Files, Rf2Result, RelationshipConfig};

/// In-memory store for SNOMED CT data.
///
/// Provides efficient lookup of concepts, descriptions, and relationships
/// after loading from RF2 files.
///
/// # Example
///
/// ```ignore
/// use snomed_loader::{SnomedStore, Rf2Config};
///
/// let mut store = SnomedStore::new();
/// store.load_concepts("concepts.txt", Rf2Config::default())?;
///
/// if let Some(concept) = store.get_concept(73211009) {
///     println!("Found: {:?}", concept);
/// }
/// ```
///
/// # Optimized Hierarchy Queries
///
/// For O(1) ancestor/descendant lookups, build the transitive closure after loading:
///
/// ```ignore
/// store.build_transitive_closure();
/// let ancestors = store.get_all_ancestors(concept_id);
/// ```
pub struct SnomedStore {
    /// Concepts indexed by SCTID.
    concepts: HashMap<SctId, Rf2Concept>,
    /// Descriptions indexed by concept ID.
    descriptions_by_concept: HashMap<SctId, Vec<Rf2Description>>,
    /// Relationships indexed by source concept ID.
    relationships_by_source: HashMap<SctId, Vec<Rf2Relationship>>,
    /// Relationships indexed by destination concept ID (for reverse lookup).
    relationships_by_destination: HashMap<SctId, Vec<Rf2Relationship>>,
    /// Reference set members indexed by refset ID.
    /// Maps refset_id -> list of referenced_component_ids (usually concept IDs).
    refsets_by_id: HashMap<SctId, Vec<SctId>>,
    /// Reverse index: component_id -> list of refset_ids containing it.
    refsets_containing_component: HashMap<SctId, Vec<SctId>>,
    /// OWL expressions indexed by referenced concept ID.
    owl_expressions_by_concept: HashMap<SctId, Vec<Rf2OwlExpression>>,
    /// Concrete relationships indexed by source concept ID.
    concrete_relationships_by_source: HashMap<SctId, Vec<Rf2ConcreteRelationship>>,
    /// Language refset members indexed by description ID.
    language_members_by_description: HashMap<SctId, Vec<Rf2LanguageRefsetMember>>,
    /// Association refset members indexed by source component.
    associations_by_source: HashMap<SctId, Vec<Rf2AssociationRefsetMember>>,
    /// MRCM data (optional).
    mrcm: Option<MrcmStore>,
    /// Precomputed transitive closure for O(1) hierarchy lookups (optional).
    /// Call `build_transitive_closure()` after loading to enable.
    transitive_closure: Option<TransitiveClosure>,
}

impl Default for SnomedStore {
    fn default() -> Self {
        Self {
            concepts: HashMap::new(),
            descriptions_by_concept: HashMap::new(),
            relationships_by_source: HashMap::new(),
            relationships_by_destination: HashMap::new(),
            refsets_by_id: HashMap::new(),
            refsets_containing_component: HashMap::new(),
            owl_expressions_by_concept: HashMap::new(),
            concrete_relationships_by_source: HashMap::new(),
            language_members_by_description: HashMap::new(),
            associations_by_source: HashMap::new(),
            mrcm: None,
            transitive_closure: None,
        }
    }
}

impl std::fmt::Debug for SnomedStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnomedStore")
            .field("concepts", &self.concepts.len())
            .field("descriptions_by_concept", &self.descriptions_by_concept.len())
            .field("relationships_by_source", &self.relationships_by_source.len())
            .field("relationships_by_destination", &self.relationships_by_destination.len())
            .field("refsets_by_id", &self.refsets_by_id.len())
            .field("refsets_containing_component", &self.refsets_containing_component.len())
            .field("owl_expressions_by_concept", &self.owl_expressions_by_concept.len())
            .field("concrete_relationships_by_source", &self.concrete_relationships_by_source.len())
            .field("language_members_by_description", &self.language_members_by_description.len())
            .field("associations_by_source", &self.associations_by_source.len())
            .field("mrcm", &self.mrcm.is_some())
            .field("transitive_closure", &self.transitive_closure.is_some())
            .finish()
    }
}

impl SnomedStore {
    /// Creates a new empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a store with pre-allocated capacity.
    ///
    /// Use this when you know approximately how many records to expect.
    pub fn with_capacity(
        concept_count: usize,
        _description_count: usize,
        _relationship_count: usize,
    ) -> Self {
        Self {
            concepts: HashMap::with_capacity(concept_count),
            descriptions_by_concept: HashMap::with_capacity(concept_count),
            relationships_by_source: HashMap::with_capacity(concept_count),
            relationships_by_destination: HashMap::with_capacity(concept_count),
            refsets_by_id: HashMap::new(),
            refsets_containing_component: HashMap::new(),
            owl_expressions_by_concept: HashMap::new(),
            concrete_relationships_by_source: HashMap::new(),
            language_members_by_description: HashMap::new(),
            associations_by_source: HashMap::new(),
            mrcm: None,
            transitive_closure: None,
        }
    }

    /// Loads concepts from an RF2 file.
    pub fn load_concepts<P: AsRef<Path>>(&mut self, path: P, config: Rf2Config) -> Rf2Result<usize> {
        let parser = Rf2Parser::<_, Rf2Concept>::from_path(path, config)?;
        let mut count = 0;

        for concept in parser.flatten() {
            self.concepts.insert(concept.id, concept);
            count += 1;
        }

        Ok(count)
    }

    /// Loads descriptions from an RF2 file.
    pub fn load_descriptions<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: DescriptionConfig,
    ) -> Rf2Result<usize> {
        let parser = Rf2Parser::<_, Rf2Description>::from_path(path, config.base.clone())?;
        let mut count = 0;

        for desc in parser.flatten() {
            if desc.passes_description_filter(&config) {
                self.descriptions_by_concept
                    .entry(desc.concept_id)
                    .or_default()
                    .push(desc);
                count += 1;
            }
        }

        Ok(count)
    }

    /// Loads relationships from an RF2 file.
    pub fn load_relationships<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: RelationshipConfig,
    ) -> Rf2Result<usize> {
        let parser = Rf2Parser::<_, Rf2Relationship>::from_path(path, config.base.clone())?;
        let mut count = 0;

        for rel in parser.flatten() {
            if rel.passes_relationship_filter(&config) {
                let rel_clone = rel.clone();
                self.relationships_by_source
                    .entry(rel.source_id)
                    .or_default()
                    .push(rel);
                self.relationships_by_destination
                    .entry(rel_clone.destination_id)
                    .or_default()
                    .push(rel_clone);
                count += 1;
            }
        }

        Ok(count)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PARALLEL LOADING METHODS (requires "parallel" feature)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Loads concepts from an RF2 file using parallel parsing.
    ///
    /// This method reads all lines into memory, then parses them in parallel
    /// using rayon. Significantly faster for large files on multi-core systems.
    #[cfg(feature = "parallel")]
    pub fn load_concepts_parallel<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: Rf2Config,
    ) -> Rf2Result<usize> {
        let lines = read_lines_skip_header(path)?;

        let concepts: Vec<Rf2Concept> = lines
            .par_iter()
            .filter_map(|line| parse_concept_line(line, &config))
            .collect();

        let count = concepts.len();
        for concept in concepts {
            self.concepts.insert(concept.id, concept);
        }

        Ok(count)
    }

    /// Loads descriptions from an RF2 file using parallel parsing.
    #[cfg(feature = "parallel")]
    pub fn load_descriptions_parallel<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: DescriptionConfig,
    ) -> Rf2Result<usize> {
        let lines = read_lines_skip_header(path)?;

        let descriptions: Vec<Rf2Description> = lines
            .par_iter()
            .filter_map(|line| parse_description_line(line, &config))
            .collect();

        let count = descriptions.len();
        for desc in descriptions {
            self.descriptions_by_concept
                .entry(desc.concept_id)
                .or_default()
                .push(desc);
        }

        Ok(count)
    }

    /// Loads relationships from an RF2 file using parallel parsing.
    #[cfg(feature = "parallel")]
    pub fn load_relationships_parallel<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: RelationshipConfig,
    ) -> Rf2Result<usize> {
        let lines = read_lines_skip_header(path)?;

        let relationships: Vec<Rf2Relationship> = lines
            .par_iter()
            .filter_map(|line| parse_relationship_line(line, &config))
            .collect();

        let count = relationships.len();
        for rel in relationships {
            let rel_clone = rel.clone();
            self.relationships_by_source
                .entry(rel.source_id)
                .or_default()
                .push(rel);
            self.relationships_by_destination
                .entry(rel_clone.destination_id)
                .or_default()
                .push(rel_clone);
        }

        Ok(count)
    }

    /// Loads all RF2 files in parallel (each file type loaded concurrently).
    ///
    /// This is the fastest way to load a complete SNOMED CT release.
    /// Each file is parsed using parallel line processing, and all three
    /// file types are loaded concurrently.
    #[cfg(feature = "parallel")]
    pub fn load_all_parallel(&mut self, files: &Rf2Files) -> Rf2Result<(usize, usize, usize)> {
        let concept_path = files.concept_file.clone();
        let description_path = files.description_file.clone();
        let relationship_path = files.relationship_file.clone();

        // Parse all files in parallel using nested rayon::join
        let ((concepts, descriptions), relationships) = rayon::join(
            || {
                rayon::join(
                    || {
                        concept_path.as_ref().map(|p| {
                            let lines = read_lines_skip_header(p).unwrap_or_default();
                            let config = Rf2Config::default();
                            lines
                                .par_iter()
                                .filter_map(|line| parse_concept_line(line, &config))
                                .collect::<Vec<_>>()
                        })
                    },
                    || {
                        description_path.as_ref().map(|p| {
                            let lines = read_lines_skip_header(p).unwrap_or_default();
                            let config = DescriptionConfig::english_terms();
                            lines
                                .par_iter()
                                .filter_map(|line| parse_description_line(line, &config))
                                .collect::<Vec<_>>()
                        })
                    },
                )
            },
            || {
                relationship_path.as_ref().map(|p| {
                    let lines = read_lines_skip_header(p).unwrap_or_default();
                    let config = RelationshipConfig::inferred_only();
                    lines
                        .par_iter()
                        .filter_map(|line| parse_relationship_line(line, &config))
                        .collect::<Vec<_>>()
                })
            },
        );

        // Insert into store
        let concept_count = if let Some(concepts) = concepts {
            let count = concepts.len();
            for concept in concepts {
                self.concepts.insert(concept.id, concept);
            }
            count
        } else {
            0
        };

        let desc_count = if let Some(descriptions) = descriptions {
            let count = descriptions.len();
            for desc in descriptions {
                self.descriptions_by_concept
                    .entry(desc.concept_id)
                    .or_default()
                    .push(desc);
            }
            count
        } else {
            0
        };

        let rel_count = if let Some(relationships) = relationships {
            let count = relationships.len();
            for rel in relationships {
                let rel_clone = rel.clone();
                self.relationships_by_source
                    .entry(rel.source_id)
                    .or_default()
                    .push(rel);
                self.relationships_by_destination
                    .entry(rel_clone.destination_id)
                    .or_default()
                    .push(rel_clone);
            }
            count
        } else {
            0
        };

        Ok((concept_count, desc_count, rel_count))
    }

    /// Loads all RF2 files from a discovered file set.
    pub fn load_all(&mut self, files: &Rf2Files) -> Rf2Result<()> {
        if let Some(ref concept_path) = files.concept_file {
            self.load_concepts(concept_path, Rf2Config::default())?;
        }

        if let Some(ref description_path) = files.description_file {
            self.load_descriptions(description_path, DescriptionConfig::english_terms())?;
        }

        if let Some(ref relationship_path) = files.relationship_file {
            self.load_relationships(relationship_path, RelationshipConfig::inferred_only())?;
        }

        Ok(())
    }

    /// Loads simple reference sets from RF2 files.
    ///
    /// Reference sets are loaded from all discovered simple refset files.
    /// Members are indexed by refset ID for efficient lookup.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use snomed_loader::{SnomedStore, discover_rf2_files, Rf2Config};
    ///
    /// let files = discover_rf2_files("/path/to/snomed")?;
    /// let mut store = SnomedStore::new();
    /// store.load_all(&files)?;
    /// store.load_simple_refsets(&files, Rf2Config::default())?;
    ///
    /// // Query members of a specific refset
    /// let members = store.get_refset_members(723264001); // Lateralizable body structure reference set
    /// ```
    pub fn load_simple_refsets(&mut self, files: &Rf2Files, config: Rf2Config) -> Rf2Result<usize> {
        let mut total_count = 0;

        for refset_path in &files.simple_refset_files {
            let parser =
                Rf2Parser::<_, Rf2SimpleRefsetMember>::from_path(refset_path, config.clone())?;

            for member in parser.flatten() {
                // Forward index: refset_id -> members
                self.refsets_by_id
                    .entry(member.refset_id)
                    .or_default()
                    .push(member.referenced_component_id);

                // Reverse index: component_id -> refsets containing it
                self.refsets_containing_component
                    .entry(member.referenced_component_id)
                    .or_default()
                    .push(member.refset_id);

                total_count += 1;
            }
        }

        Ok(total_count)
    }

    /// Loads MRCM reference set data from discovered files.
    ///
    /// This loads the MRCM domain, attribute domain, and attribute range
    /// reference sets if present in the file set.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use snomed_loader::{SnomedStore, discover_rf2_files};
    ///
    /// let files = discover_rf2_files("/path/to/snomed")?;
    /// let mut store = SnomedStore::new();
    /// store.load_all(&files)?;
    /// store.load_mrcm(&files)?;
    ///
    /// if let Some(mrcm) = store.get_mrcm() {
    ///     println!("MRCM domains: {}", mrcm.domain_count());
    /// }
    /// ```
    pub fn load_mrcm(&mut self, files: &Rf2Files) -> Rf2Result<()> {
        // Only load if at least one MRCM file is present
        if files.mrcm_domain.is_none()
            && files.mrcm_attribute_domain.is_none()
            && files.mrcm_attribute_range.is_none()
        {
            return Ok(());
        }

        let mrcm_store = MrcmStore::from_files(files)?;


        self.mrcm = Some(mrcm_store);
        Ok(())
    }

    /// Returns a reference to the MRCM store if loaded.
    pub fn get_mrcm(&self) -> Option<&MrcmStore> {
        self.mrcm.as_ref()
    }

    /// Returns true if MRCM data has been loaded.
    pub fn has_mrcm(&self) -> bool {
        self.mrcm.is_some()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // NEW DATA TYPE LOADING METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Loads OWL expression refset files.
    ///
    /// OWL expressions contain OWL 2 EL axioms that define concept semantics.
    /// Members are indexed by the referenced concept ID for efficient lookup.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let files = discover_rf2_files("/path/to/snomed")?;
    /// let mut store = SnomedStore::new();
    /// store.load_owl_expressions(&files, Rf2Config::default())?;
    ///
    /// // Get OWL axioms for a concept
    /// if let Some(axioms) = store.get_owl_expressions(404684003) {
    ///     for axiom in axioms {
    ///         println!("OWL: {}", axiom.owl_expression);
    ///     }
    /// }
    /// ```
    pub fn load_owl_expressions(&mut self, files: &Rf2Files, config: Rf2Config) -> Rf2Result<usize> {
        let mut total_count = 0;

        for owl_path in &files.owl_expression_files {
            let parser = Rf2Parser::<_, Rf2OwlExpression>::from_path(owl_path, config.clone())?;

            for expr in parser.flatten() {
                self.owl_expressions_by_concept
                    .entry(expr.referenced_component_id)
                    .or_default()
                    .push(expr);
                total_count += 1;
            }
        }

        Ok(total_count)
    }

    /// Loads concrete relationship file.
    ///
    /// Concrete relationships have literal values (string, integer, decimal)
    /// instead of reference to another concept. Common in drug dosages, measurements, etc.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let files = discover_rf2_files("/path/to/snomed")?;
    /// let mut store = SnomedStore::new();
    /// store.load_concrete_relationships(&files, Rf2Config::default())?;
    ///
    /// // Get concrete relationships for a drug concept
    /// if let Some(rels) = store.get_concrete_relationships(322236009) {
    ///     for rel in rels {
    ///         println!("Value: {:?}", rel.value);
    ///     }
    /// }
    /// ```
    pub fn load_concrete_relationships(
        &mut self,
        files: &Rf2Files,
        config: Rf2Config,
    ) -> Rf2Result<usize> {
        let Some(ref concrete_path) = files.concrete_relationship_file else {
            return Ok(0);
        };

        let parser = Rf2Parser::<_, Rf2ConcreteRelationship>::from_path(concrete_path, config)?;
        let mut count = 0;

        for rel in parser.flatten() {
            self.concrete_relationships_by_source
                .entry(rel.source_id)
                .or_default()
                .push(rel);
            count += 1;
        }

        Ok(count)
    }

    /// Loads language reference set files.
    ///
    /// Language refsets indicate which descriptions are preferred/acceptable
    /// for a specific language/dialect. Members are indexed by description ID.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let files = discover_rf2_files("/path/to/snomed")?;
    /// let mut store = SnomedStore::new();
    /// store.load_language_refsets(&files, Rf2Config::default())?;
    ///
    /// // Check acceptability of a description
    /// if let Some(members) = store.get_language_members_for_description(desc_id) {
    ///     for member in members {
    ///         if member.is_preferred() {
    ///             println!("Preferred in refset {}", member.refset_id);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn load_language_refsets(&mut self, files: &Rf2Files, config: Rf2Config) -> Rf2Result<usize> {
        let mut total_count = 0;

        for lang_path in &files.language_refset_files {
            let parser =
                Rf2Parser::<_, Rf2LanguageRefsetMember>::from_path(lang_path, config.clone())?;

            for member in parser.flatten() {
                self.language_members_by_description
                    .entry(member.referenced_component_id)
                    .or_default()
                    .push(member);
                total_count += 1;
            }
        }

        Ok(total_count)
    }

    /// Loads association reference set files.
    ///
    /// Association refsets link concepts to their replacements, alternatives,
    /// or related concepts. Common for tracking historical changes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let files = discover_rf2_files("/path/to/snomed")?;
    /// let mut store = SnomedStore::new();
    /// store.load_association_refsets(&files, Rf2Config::default())?;
    ///
    /// // Find replacements for an inactive concept
    /// if let Some(assocs) = store.get_associations_for_concept(inactive_concept_id) {
    ///     for assoc in assocs {
    ///         if assoc.is_replaced_by_association() {
    ///             println!("Replaced by: {}", assoc.target_component_id);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn load_association_refsets(
        &mut self,
        files: &Rf2Files,
        config: Rf2Config,
    ) -> Rf2Result<usize> {
        let mut total_count = 0;

        for assoc_path in &files.association_refset_files {
            let parser =
                Rf2Parser::<_, Rf2AssociationRefsetMember>::from_path(assoc_path, config.clone())?;

            for member in parser.flatten() {
                self.associations_by_source
                    .entry(member.referenced_component_id)
                    .or_default()
                    .push(member);
                total_count += 1;
            }
        }

        Ok(total_count)
    }

    /// Bulk inserts concepts.
    pub fn insert_concepts(&mut self, concepts: impl IntoIterator<Item = Rf2Concept>) {
        for concept in concepts {
            self.concepts.insert(concept.id, concept);
        }
    }

    /// Bulk inserts descriptions.
    pub fn insert_descriptions(&mut self, descriptions: impl IntoIterator<Item = Rf2Description>) {
        for desc in descriptions {
            self.descriptions_by_concept
                .entry(desc.concept_id)
                .or_default()
                .push(desc);
        }
    }

    /// Bulk inserts relationships.
    pub fn insert_relationships(
        &mut self,
        relationships: impl IntoIterator<Item = Rf2Relationship>,
    ) {
        for rel in relationships {
            let rel_clone = rel.clone();
            self.relationships_by_source
                .entry(rel.source_id)
                .or_default()
                .push(rel);
            self.relationships_by_destination
                .entry(rel_clone.destination_id)
                .or_default()
                .push(rel_clone);
        }
    }

    // Query methods

    /// Gets a concept by its ID.
    pub fn get_concept(&self, id: SctId) -> Option<&Rf2Concept> {
        self.concepts.get(&id)
    }

    /// Returns true if a concept exists in the store.
    pub fn has_concept(&self, id: SctId) -> bool {
        self.concepts.contains_key(&id)
    }

    /// Gets all descriptions for a concept.
    pub fn get_descriptions(&self, concept_id: SctId) -> Option<&Vec<Rf2Description>> {
        self.descriptions_by_concept.get(&concept_id)
    }

    /// Gets the FSN (Fully Specified Name) for a concept.
    pub fn get_fsn(&self, concept_id: SctId) -> Option<&Rf2Description> {
        self.descriptions_by_concept
            .get(&concept_id)?
            .iter()
            .find(|d| d.is_fsn())
    }

    /// Gets the preferred term for a concept (first synonym, or FSN if no synonym).
    pub fn get_preferred_term(&self, concept_id: SctId) -> Option<&str> {
        let descriptions = self.descriptions_by_concept.get(&concept_id)?;

        // First try to find a synonym
        if let Some(synonym) = descriptions.iter().find(|d| d.is_synonym()) {
            return Some(&synonym.term);
        }

        // Fall back to FSN
        if let Some(fsn) = descriptions.iter().find(|d| d.is_fsn()) {
            return Some(&fsn.term);
        }

        None
    }

    /// Gets relationships where this concept is the source.
    pub fn get_outgoing_relationships(&self, source_id: SctId) -> Option<&Vec<Rf2Relationship>> {
        self.relationships_by_source.get(&source_id)
    }

    /// Gets relationships where this concept is the destination.
    pub fn get_incoming_relationships(
        &self,
        destination_id: SctId,
    ) -> Option<&Vec<Rf2Relationship>> {
        self.relationships_by_destination.get(&destination_id)
    }

    /// Gets parent concepts (via IS_A relationship).
    pub fn get_parents(&self, concept_id: SctId) -> Vec<SctId> {
        self.relationships_by_source
            .get(&concept_id)
            .map(|rels| {
                rels.iter()
                    .filter(|r| r.is_is_a())
                    .map(|r| r.destination_id)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets child concepts (via IS_A relationship).
    pub fn get_children(&self, concept_id: SctId) -> Vec<SctId> {
        self.relationships_by_destination
            .get(&concept_id)
            .map(|rels| {
                rels.iter()
                    .filter(|r| r.is_is_a())
                    .map(|r| r.source_id)
                    .collect()
            })
            .unwrap_or_default()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRANSITIVE CLOSURE - O(1) HIERARCHY QUERIES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Builds the transitive closure for O(1) ancestor/descendant lookups.
    ///
    /// This is a one-time operation that precomputes all ancestor and descendant
    /// relationships. After calling this method, hierarchy queries like
    /// `get_all_ancestors()`, `get_all_descendants()`, and `is_descendant_of()`
    /// become O(1) operations instead of requiring BFS traversal.
    ///
    /// # Performance
    ///
    /// - Build time: O(n × d) where n = concepts, d = average hierarchy depth
    /// - Memory: Stores all transitive relationships (can be significant)
    /// - Query time after build: O(1)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut store = SnomedStore::new();
    /// store.load_all(&files)?;
    /// store.build_transitive_closure();
    ///
    /// // Now O(1) lookups:
    /// let ancestors = store.get_all_ancestors(concept_id);
    /// let is_desc = store.is_descendant_of(child, ancestor);
    /// ```
    pub fn build_transitive_closure(&mut self) {
        let closure = TransitiveClosure::build(self);
        self.transitive_closure = Some(closure);
    }

    /// Returns true if the transitive closure has been built.
    pub fn has_transitive_closure(&self) -> bool {
        self.transitive_closure.is_some()
    }

    /// Gets all ancestors of a concept (O(1) if transitive closure is built).
    ///
    /// Returns all concepts that are ancestors of the given concept via IS_A
    /// relationships. If the transitive closure has not been built, falls back
    /// to BFS traversal.
    pub fn get_all_ancestors(&self, concept_id: SctId) -> Vec<SctId> {
        if let Some(ref closure) = self.transitive_closure {
            closure
                .get_ancestors(concept_id)
                .map(|set| set.iter().copied().collect())
                .unwrap_or_default()
        } else {
            // Fallback to BFS if closure not built
            self.compute_ancestors_bfs(concept_id)
        }
    }

    /// Gets all ancestors of a concept including itself.
    pub fn get_all_ancestors_or_self(&self, concept_id: SctId) -> Vec<SctId> {
        if let Some(ref closure) = self.transitive_closure {
            closure.get_ancestors_or_self(concept_id).into_iter().collect()
        } else {
            let mut result = self.compute_ancestors_bfs(concept_id);
            if !result.contains(&concept_id) {
                result.push(concept_id);
            }
            result
        }
    }

    /// Gets all descendants of a concept (O(1) if transitive closure is built).
    ///
    /// Returns all concepts that are descendants of the given concept via IS_A
    /// relationships. If the transitive closure has not been built, falls back
    /// to BFS traversal.
    pub fn get_all_descendants(&self, concept_id: SctId) -> Vec<SctId> {
        if let Some(ref closure) = self.transitive_closure {
            closure
                .get_descendants(concept_id)
                .map(|set| set.iter().copied().collect())
                .unwrap_or_default()
        } else {
            // Fallback to BFS if closure not built
            self.compute_descendants_bfs(concept_id)
        }
    }

    /// Gets all descendants of a concept including itself.
    pub fn get_all_descendants_or_self(&self, concept_id: SctId) -> Vec<SctId> {
        if let Some(ref closure) = self.transitive_closure {
            closure.get_descendants_or_self(concept_id).into_iter().collect()
        } else {
            let mut result = self.compute_descendants_bfs(concept_id);
            if !result.contains(&concept_id) {
                result.push(concept_id);
            }
            result
        }
    }

    /// Checks if a concept is a descendant of another (O(1) if closure built).
    ///
    /// Returns true if `descendant` is a descendant of `ancestor` via IS_A
    /// relationships (not including self).
    pub fn is_descendant_of(&self, descendant: SctId, ancestor: SctId) -> bool {
        if let Some(ref closure) = self.transitive_closure {
            closure.is_descendant_of(descendant, ancestor)
        } else {
            self.get_all_ancestors(descendant).contains(&ancestor)
        }
    }

    /// Checks if a concept is an ancestor of another (O(1) if closure built).
    pub fn is_ancestor_of(&self, ancestor: SctId, descendant: SctId) -> bool {
        if let Some(ref closure) = self.transitive_closure {
            closure.is_ancestor_of(ancestor, descendant)
        } else {
            self.get_all_descendants(ancestor).contains(&descendant)
        }
    }

    /// BFS fallback for computing ancestors when closure not available.
    fn compute_ancestors_bfs(&self, concept_id: SctId) -> Vec<SctId> {
        use std::collections::{HashSet, VecDeque};

        let mut ancestors = HashSet::new();
        let mut queue = VecDeque::new();

        for parent in self.get_parents(concept_id) {
            queue.push_back(parent);
        }

        while let Some(current) = queue.pop_front() {
            if ancestors.insert(current) {
                for parent in self.get_parents(current) {
                    queue.push_back(parent);
                }
            }
        }

        ancestors.into_iter().collect()
    }

    /// BFS fallback for computing descendants when closure not available.
    fn compute_descendants_bfs(&self, concept_id: SctId) -> Vec<SctId> {
        use std::collections::{HashSet, VecDeque};

        let mut descendants = HashSet::new();
        let mut queue = VecDeque::new();

        for child in self.get_children(concept_id) {
            queue.push_back(child);
        }

        while let Some(current) = queue.pop_front() {
            if descendants.insert(current) {
                for child in self.get_children(current) {
                    queue.push_back(child);
                }
            }
        }

        descendants.into_iter().collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // REFERENCE SET QUERIES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Gets members of a reference set by refset ID.
    ///
    /// Returns a list of referenced component IDs (usually concept IDs) that
    /// are members of the specified reference set.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Get members of the "Lateralizable body structure" reference set
    /// let members = store.get_refset_members(723264001);
    /// for concept_id in members {
    ///     println!("Member: {}", concept_id);
    /// }
    /// ```
    pub fn get_refset_members(&self, refset_id: SctId) -> Vec<SctId> {
        self.refsets_by_id
            .get(&refset_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Returns true if a reference set exists in the store.
    pub fn has_refset(&self, refset_id: SctId) -> bool {
        self.refsets_by_id.contains_key(&refset_id)
    }

    /// Returns the number of reference sets loaded.
    pub fn refset_count(&self) -> usize {
        self.refsets_by_id.len()
    }

    /// Returns total number of refset members across all reference sets.
    pub fn refset_member_count(&self) -> usize {
        self.refsets_by_id.values().map(|v| v.len()).sum()
    }

    /// Gets all refset IDs that contain a specific component (reverse lookup).
    ///
    /// This is the inverse of `get_refset_members`: instead of asking "what concepts
    /// are in this refset?", you ask "what refsets contain this concept?".
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Find all reference sets containing a specific concept
    /// let refsets = store.get_refsets_for_concept(80146002); // Appendectomy
    /// for refset_id in refsets {
    ///     println!("Member of refset: {}", refset_id);
    /// }
    /// ```
    pub fn get_refsets_for_concept(&self, concept_id: SctId) -> Vec<SctId> {
        self.refsets_containing_component
            .get(&concept_id)
            .cloned()
            .unwrap_or_default()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // NEW DATA TYPE QUERIES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Gets OWL expressions for a concept.
    ///
    /// Returns OWL 2 EL axioms that define the semantics of the concept.
    /// These include SubClassOf, EquivalentClasses, and other axiom types.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(axioms) = store.get_owl_expressions(404684003) {
    ///     for axiom in axioms {
    ///         if axiom.is_subclass_axiom() {
    ///             println!("SubClassOf: {}", axiom.owl_expression);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn get_owl_expressions(&self, concept_id: SctId) -> Option<&Vec<Rf2OwlExpression>> {
        self.owl_expressions_by_concept.get(&concept_id)
    }

    /// Returns the total number of OWL expressions loaded.
    pub fn owl_expression_count(&self) -> usize {
        self.owl_expressions_by_concept.values().map(|v| v.len()).sum()
    }

    /// Gets concrete relationships for a source concept.
    ///
    /// Concrete relationships have literal values (string, integer, decimal)
    /// instead of referencing another concept.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(rels) = store.get_concrete_relationships(322236009) {
    ///     for rel in rels {
    ///         if let Some(value) = rel.value.as_integer() {
    ///             println!("Has numeric value: {}", value);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn get_concrete_relationships(
        &self,
        source_id: SctId,
    ) -> Option<&Vec<Rf2ConcreteRelationship>> {
        self.concrete_relationships_by_source.get(&source_id)
    }

    /// Returns the total number of concrete relationships loaded.
    pub fn concrete_relationship_count(&self) -> usize {
        self.concrete_relationships_by_source
            .values()
            .map(|v| v.len())
            .sum()
    }

    /// Gets language refset members for a description.
    ///
    /// Returns information about which language refsets include this description
    /// and whether it's preferred or acceptable in each.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(members) = store.get_language_members_for_description(desc_id) {
    ///     for member in members {
    ///         if member.is_preferred() {
    ///             println!("Preferred in language refset {}", member.refset_id);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn get_language_members_for_description(
        &self,
        description_id: SctId,
    ) -> Option<&Vec<Rf2LanguageRefsetMember>> {
        self.language_members_by_description.get(&description_id)
    }

    /// Returns the total number of language refset members loaded.
    pub fn language_member_count(&self) -> usize {
        self.language_members_by_description
            .values()
            .map(|v| v.len())
            .sum()
    }

    /// Gets the preferred term for a concept in a specific language refset.
    ///
    /// This finds the description that is marked as "preferred" in the given
    /// language reference set (e.g., US English, GB English).
    ///
    /// # Example
    ///
    /// ```ignore
    /// // US English language refset
    /// const US_ENGLISH: SctId = 900000000000509007;
    ///
    /// if let Some(term) = store.get_preferred_term_for_language(73211009, US_ENGLISH) {
    ///     println!("Preferred term: {}", term);
    /// }
    /// ```
    pub fn get_preferred_term_for_language(
        &self,
        concept_id: SctId,
        language_refset_id: SctId,
    ) -> Option<&str> {
        // Get all descriptions for this concept
        let descriptions = self.descriptions_by_concept.get(&concept_id)?;

        // Find a description that is preferred in the given language refset
        for desc in descriptions {
            if let Some(lang_members) = self.language_members_by_description.get(&desc.id) {
                for member in lang_members {
                    if member.refset_id == language_refset_id && member.is_preferred() {
                        return Some(&desc.term);
                    }
                }
            }
        }

        None
    }

    /// Gets association refset members for a component.
    ///
    /// Returns associations that link this component to replacements,
    /// alternatives, or related components.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(assocs) = store.get_associations_for_concept(inactive_concept_id) {
    ///     for assoc in assocs {
    ///         if assoc.is_replaced_by_association() {
    ///             println!("Replaced by: {}", assoc.target_component_id);
    ///         } else if assoc.is_same_as_association() {
    ///             println!("Same as: {}", assoc.target_component_id);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn get_associations_for_concept(
        &self,
        concept_id: SctId,
    ) -> Option<&Vec<Rf2AssociationRefsetMember>> {
        self.associations_by_source.get(&concept_id)
    }

    /// Returns the total number of association refset members loaded.
    pub fn association_count(&self) -> usize {
        self.associations_by_source.values().map(|v| v.len()).sum()
    }

    /// Gets the replacement concept for an inactive concept.
    ///
    /// This is a convenience method that looks for a REPLACED_BY association
    /// and returns the target concept ID if found.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(replacement) = store.get_replacement_concept(inactive_concept_id) {
    ///     println!("Use {} instead", replacement);
    /// }
    /// ```
    pub fn get_replacement_concept(&self, concept_id: SctId) -> Option<SctId> {
        self.associations_by_source
            .get(&concept_id)?
            .iter()
            .find(|a| a.is_replaced_by_association())
            .map(|a| a.target_component_id)
    }

    // Statistics

    /// Returns the number of concepts in the store.
    pub fn concept_count(&self) -> usize {
        self.concepts.len()
    }

    /// Returns the number of descriptions in the store.
    pub fn description_count(&self) -> usize {
        self.descriptions_by_concept.values().map(|v| v.len()).sum()
    }

    /// Returns the number of relationships in the store.
    pub fn relationship_count(&self) -> usize {
        self.relationships_by_source.values().map(|v| v.len()).sum()
    }

    /// Returns an iterator over all concepts.
    pub fn concepts(&self) -> impl Iterator<Item = &Rf2Concept> {
        self.concepts.values()
    }

    /// Returns an iterator over all concept IDs.
    pub fn concept_ids(&self) -> impl Iterator<Item = &SctId> {
        self.concepts.keys()
    }

    /// Estimates memory usage in bytes.
    pub fn estimated_memory_bytes(&self) -> usize {
        use std::mem::size_of;

        let concept_size = self.concepts.len() * (size_of::<SctId>() + size_of::<Rf2Concept>());

        let desc_size: usize = self
            .descriptions_by_concept
            .values()
            .map(|descs| {
                size_of::<SctId>()
                    + descs
                        .iter()
                        .map(|d| size_of::<Rf2Description>() + d.term.len())
                        .sum::<usize>()
            })
            .sum();

        let rel_size = self.relationships_by_source.len()
            * 2 // Both source and dest maps
            * (size_of::<SctId>() + size_of::<Rf2Relationship>());

        concept_size + desc_size + rel_size
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARALLEL PARSING HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Reads all lines from a file, skipping the header row.
#[cfg(feature = "parallel")]
fn read_lines_skip_header<P: AsRef<Path>>(path: P) -> Rf2Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .skip(1) // Skip header
        .filter_map(Result::ok)
        .filter(|line| !line.is_empty())
        .collect();
    Ok(lines)
}

/// Parses a single concept line.
#[cfg(feature = "parallel")]
fn parse_concept_line(line: &str, config: &Rf2Config) -> Option<Rf2Concept> {
    let fields: Vec<&str> = line.split('\t').collect();
    if fields.len() < 5 {
        return None;
    }

    let active = parse::boolean(fields[2]).ok()?;
    if config.active_only && !active {
        return None;
    }

    Some(Rf2Concept {
        id: parse::sctid(fields[0]).ok()?,
        effective_time: parse::effective_time(fields[1]).ok()?,
        active,
        module_id: parse::sctid(fields[3]).ok()?,
        definition_status_id: parse::sctid(fields[4]).ok()?,
    })
}

/// Parses a single description line.
#[cfg(feature = "parallel")]
fn parse_description_line(line: &str, config: &DescriptionConfig) -> Option<Rf2Description> {
    let fields: Vec<&str> = line.split('\t').collect();
    if fields.len() < 9 {
        return None;
    }

    let active = parse::boolean(fields[2]).ok()?;
    if config.base.active_only && !active {
        return None;
    }

    let language_code = fields[5].to_string();
    if !config.language_codes.is_empty() && !config.language_codes.contains(&language_code) {
        return None;
    }

    let type_id = parse::sctid(fields[6]).ok()?;
    if !config.type_ids.is_empty() && !config.type_ids.contains(&type_id) {
        return None;
    }

    Some(Rf2Description {
        id: parse::sctid(fields[0]).ok()?,
        effective_time: parse::effective_time(fields[1]).ok()?,
        active,
        module_id: parse::sctid(fields[3]).ok()?,
        concept_id: parse::sctid(fields[4]).ok()?,
        language_code,
        type_id,
        term: fields[7].to_string(),
        case_significance_id: parse::sctid(fields[8]).ok()?,
    })
}

/// Parses a single relationship line.
#[cfg(feature = "parallel")]
fn parse_relationship_line(line: &str, config: &RelationshipConfig) -> Option<Rf2Relationship> {
    let fields: Vec<&str> = line.split('\t').collect();
    if fields.len() < 10 {
        return None;
    }

    let active = parse::boolean(fields[2]).ok()?;
    if config.base.active_only && !active {
        return None;
    }

    let type_id = parse::sctid(fields[7]).ok()?;
    if !config.type_ids.is_empty() && !config.type_ids.contains(&type_id) {
        return None;
    }

    let characteristic_type_id = parse::sctid(fields[8]).ok()?;
    if !config.characteristic_type_ids.is_empty()
        && !config.characteristic_type_ids.contains(&characteristic_type_id)
    {
        return None;
    }

    Some(Rf2Relationship {
        id: parse::sctid(fields[0]).ok()?,
        effective_time: parse::effective_time(fields[1]).ok()?,
        active,
        module_id: parse::sctid(fields[3]).ok()?,
        source_id: parse::sctid(fields[4]).ok()?,
        destination_id: parse::sctid(fields[5]).ok()?,
        relationship_group: parse::integer(fields[6]).ok()?,
        type_id,
        characteristic_type_id,
        modifier_id: parse::sctid(fields[9]).ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_types::{CharacteristicType, DefinitionStatus, DescriptionType, ModifierType};

    fn make_test_concept(id: SctId) -> Rf2Concept {
        Rf2Concept {
            id,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            definition_status_id: DefinitionStatus::PRIMITIVE_ID,
        }
    }

    fn make_test_description(id: SctId, concept_id: SctId, is_fsn: bool) -> Rf2Description {
        Rf2Description {
            id,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            concept_id,
            language_code: "en".to_string(),
            type_id: if is_fsn {
                DescriptionType::FSN_ID
            } else {
                DescriptionType::SYNONYM_ID
            },
            term: format!("Test term {}", id),
            case_significance_id: 900000000000448009,
        }
    }

    fn make_test_relationship(
        id: SctId,
        source_id: SctId,
        destination_id: SctId,
        is_is_a: bool,
    ) -> Rf2Relationship {
        Rf2Relationship {
            id,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            source_id,
            destination_id,
            relationship_group: 0,
            type_id: if is_is_a { 116680003 } else { 363698007 },
            characteristic_type_id: CharacteristicType::INFERRED_ID,
            modifier_id: ModifierType::EXISTENTIAL_ID,
        }
    }

    #[test]
    fn test_store_concepts() {
        let mut store = SnomedStore::new();

        let concept1 = make_test_concept(100);
        let concept2 = make_test_concept(200);

        store.insert_concepts([concept1.clone(), concept2.clone()]);

        assert_eq!(store.concept_count(), 2);
        assert!(store.has_concept(100));
        assert!(store.has_concept(200));
        assert!(!store.has_concept(300));

        let retrieved = store.get_concept(100).unwrap();
        assert_eq!(retrieved.id, 100);
    }

    #[test]
    fn test_store_descriptions() {
        let mut store = SnomedStore::new();

        let desc_fsn = make_test_description(1, 100, true);
        let desc_syn = make_test_description(2, 100, false);
        let desc_other = make_test_description(3, 200, true);

        store.insert_descriptions([desc_fsn, desc_syn, desc_other]);

        assert_eq!(store.description_count(), 3);

        let descs = store.get_descriptions(100).unwrap();
        assert_eq!(descs.len(), 2);

        let fsn = store.get_fsn(100).unwrap();
        assert!(fsn.is_fsn());
    }

    #[test]
    fn test_store_relationships() {
        let mut store = SnomedStore::new();

        // 100 IS_A 200
        // 100 finding_site 300
        let rel1 = make_test_relationship(1, 100, 200, true);
        let rel2 = make_test_relationship(2, 100, 300, false);

        store.insert_relationships([rel1, rel2]);

        assert_eq!(store.relationship_count(), 2);

        let outgoing = store.get_outgoing_relationships(100).unwrap();
        assert_eq!(outgoing.len(), 2);

        let parents = store.get_parents(100);
        assert_eq!(parents, vec![200]);

        let children = store.get_children(200);
        assert_eq!(children, vec![100]);
    }

    #[test]
    fn test_preferred_term() {
        let mut store = SnomedStore::new();

        // Concept with FSN and synonym
        let fsn = Rf2Description {
            id: 1,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            concept_id: 100,
            language_code: "en".to_string(),
            type_id: DescriptionType::FSN_ID,
            term: "Test concept (finding)".to_string(),
            case_significance_id: 900000000000448009,
        };

        let synonym = Rf2Description {
            id: 2,
            type_id: DescriptionType::SYNONYM_ID,
            term: "Test concept".to_string(),
            ..fsn.clone()
        };

        store.insert_descriptions([fsn, synonym]);

        // Should prefer synonym over FSN
        let term = store.get_preferred_term(100).unwrap();
        assert_eq!(term, "Test concept");

        // Concept with only FSN
        let fsn_only = Rf2Description {
            id: 3,
            concept_id: 200,
            type_id: DescriptionType::FSN_ID,
            term: "Another concept (procedure)".to_string(),
            ..make_test_description(3, 200, true)
        };

        store.insert_descriptions([fsn_only]);

        let term = store.get_preferred_term(200).unwrap();
        assert_eq!(term, "Another concept (procedure)");
    }
}
