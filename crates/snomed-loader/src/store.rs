//! In-memory SNOMED CT data store.
//!
//! Provides efficient storage and lookup for parsed RF2 data.
//! Includes parallel parsing support via rayon for maximum performance.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use snomed_types::{Rf2Concept, Rf2Description, Rf2Relationship, SctId};

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
#[derive(Debug, Default)]
pub struct SnomedStore {
    /// Concepts indexed by SCTID.
    concepts: HashMap<SctId, Rf2Concept>,
    /// Descriptions indexed by concept ID.
    descriptions_by_concept: HashMap<SctId, Vec<Rf2Description>>,
    /// Relationships indexed by source concept ID.
    relationships_by_source: HashMap<SctId, Vec<Rf2Relationship>>,
    /// Relationships indexed by destination concept ID (for reverse lookup).
    relationships_by_destination: HashMap<SctId, Vec<Rf2Relationship>>,
    /// MRCM data (optional).
    mrcm: Option<MrcmStore>,
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
            mrcm: None,
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
