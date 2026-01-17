//! In-memory MRCM data store.
//!
//! Provides efficient storage and lookup for MRCM reference set data.

use std::collections::HashMap;
use std::path::Path;

use snomed_types::{MrcmAttributeDomain, MrcmAttributeRange, MrcmDomain, SctId};

use crate::types::{Rf2Config, Rf2Files, Rf2Result};

use super::{parse_attribute_domain_file, parse_attribute_range_file, parse_domain_file};

/// In-memory store for MRCM reference set data.
///
/// Provides efficient lookup of MRCM domains, attribute domains, and attribute ranges.
///
/// # Example
///
/// ```ignore
/// use snomed_loader::mrcm::MrcmStore;
/// use snomed_loader::Rf2Files;
///
/// let files = Rf2Files { /* ... */ };
/// let store = MrcmStore::from_files(&files)?;
///
/// // Look up attribute domains for Finding site
/// let finding_site = 363698007;
/// if let Some(domains) = store.get_attribute_domains(finding_site) {
///     for domain in domains {
///         println!("Valid in domain: {}", domain.domain_id);
///     }
/// }
/// ```
#[derive(Debug, Default)]
pub struct MrcmStore {
    /// Domains indexed by domain concept ID.
    domains: HashMap<SctId, Vec<MrcmDomain>>,
    /// Attribute domains indexed by attribute concept ID.
    attribute_domains: HashMap<SctId, Vec<MrcmAttributeDomain>>,
    /// Attribute ranges indexed by attribute concept ID.
    attribute_ranges: HashMap<SctId, Vec<MrcmAttributeRange>>,
}

impl MrcmStore {
    /// Creates a new empty MRCM store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a store with pre-allocated capacity.
    pub fn with_capacity(
        domain_count: usize,
        attribute_domain_count: usize,
        attribute_range_count: usize,
    ) -> Self {
        Self {
            domains: HashMap::with_capacity(domain_count),
            attribute_domains: HashMap::with_capacity(attribute_domain_count),
            attribute_ranges: HashMap::with_capacity(attribute_range_count),
        }
    }

    /// Loads all MRCM data from a directory.
    ///
    /// Searches for MRCM reference set files in the given directory
    /// and loads them with active-only filtering by default.
    ///
    /// # Arguments
    /// * `path` - Path to directory containing MRCM files (usually `Refset/Metadata/`)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use snomed_loader::mrcm::MrcmStore;
    ///
    /// let store = MrcmStore::load("/path/to/snomed/Snapshot/Refset/Metadata")?;
    /// println!("Loaded {} domains", store.domain_count());
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Rf2Result<Self> {
        let path = path.as_ref();
        let config = Rf2Config::default();
        let mut store = Self::new();

        // Find and load MRCM files
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy();

            if !filename_str.ends_with(".txt") {
                continue;
            }

            if filename_str.contains("MRCMDomainSnapshot") {
                store.load_domains(&entry.path(), config.clone())?;
            } else if filename_str.contains("MRCMAttributeDomainSnapshot") {
                store.load_attribute_domains(&entry.path(), config.clone())?;
            } else if filename_str.contains("MRCMAttributeRangeSnapshot") {
                store.load_attribute_ranges(&entry.path(), config.clone())?;
            }
        }

        Ok(store)
    }

    /// Loads MRCM data from discovered RF2 files.
    pub fn from_files(files: &Rf2Files) -> Rf2Result<Self> {
        let config = Rf2Config::default();
        let mut store = Self::new();

        if let Some(ref path) = files.mrcm_domain {
            store.load_domains(path, config.clone())?;
        }

        if let Some(ref path) = files.mrcm_attribute_domain {
            store.load_attribute_domains(path, config.clone())?;
        }

        if let Some(ref path) = files.mrcm_attribute_range {
            store.load_attribute_ranges(path, config.clone())?;
        }

        Ok(store)
    }

    /// Loads domain reference set from a file.
    pub fn load_domains<P: AsRef<Path>>(&mut self, path: P, config: Rf2Config) -> Rf2Result<usize> {
        let parser = parse_domain_file(path, config)?;
        let mut count = 0;

        for result in parser {
            let domain = result?;
            self.domains
                .entry(domain.referenced_component_id)
                .or_default()
                .push(domain);
            count += 1;
        }

        Ok(count)
    }

    /// Loads attribute domain reference set from a file.
    pub fn load_attribute_domains<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: Rf2Config,
    ) -> Rf2Result<usize> {
        let parser = parse_attribute_domain_file(path, config)?;
        let mut count = 0;

        for result in parser {
            let attr_domain = result?;
            self.attribute_domains
                .entry(attr_domain.referenced_component_id)
                .or_default()
                .push(attr_domain);
            count += 1;
        }

        Ok(count)
    }

    /// Loads attribute range reference set from a file.
    pub fn load_attribute_ranges<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: Rf2Config,
    ) -> Rf2Result<usize> {
        let parser = parse_attribute_range_file(path, config)?;
        let mut count = 0;

        for result in parser {
            let attr_range = result?;
            self.attribute_ranges
                .entry(attr_range.referenced_component_id)
                .or_default()
                .push(attr_range);
            count += 1;
        }

        Ok(count)
    }

    // Query methods

    /// Gets domains for a concept.
    ///
    /// Returns all MRCM domain records where the concept is the domain.
    pub fn get_domains_for_concept(&self, concept_id: SctId) -> Option<&Vec<MrcmDomain>> {
        self.domains.get(&concept_id)
    }

    /// Gets attribute domain records for an attribute.
    ///
    /// Returns records defining which domains this attribute can be used in.
    pub fn get_attribute_domains(&self, attribute_id: SctId) -> Option<&Vec<MrcmAttributeDomain>> {
        self.attribute_domains.get(&attribute_id)
    }

    /// Gets attribute range records for an attribute.
    ///
    /// Returns records defining valid value ranges for this attribute.
    pub fn get_attribute_range(&self, attribute_id: SctId) -> Option<&Vec<MrcmAttributeRange>> {
        self.attribute_ranges.get(&attribute_id)
    }

    /// Checks if an attribute is valid for a specific domain.
    ///
    /// Returns true if there is an active attribute domain record
    /// that allows this attribute in the given domain.
    pub fn is_attribute_valid_for_domain(
        &self,
        attribute_id: SctId,
        domain_concept_id: SctId,
    ) -> bool {
        self.attribute_domains
            .get(&attribute_id)
            .map(|domains| {
                domains
                    .iter()
                    .any(|d| d.domain_id == domain_concept_id && d.active)
            })
            .unwrap_or(false)
    }

    /// Checks if an attribute must be grouped.
    ///
    /// Returns true if any active attribute domain record for this
    /// attribute specifies that it must be grouped.
    pub fn is_attribute_grouped(&self, attribute_id: SctId) -> bool {
        self.attribute_domains
            .get(&attribute_id)
            .map(|domains| domains.iter().any(|d| d.grouped && d.active))
            .unwrap_or(false)
    }

    /// Gets the range constraint ECL for an attribute.
    ///
    /// Returns the first active range constraint found for this attribute.
    pub fn get_range_constraint(&self, attribute_id: SctId) -> Option<&str> {
        self.attribute_ranges.get(&attribute_id).and_then(|ranges| {
            ranges
                .iter()
                .find(|r| r.active)
                .map(|r| r.range_constraint.as_str())
        })
    }

    /// Gets all valid domains for an attribute.
    ///
    /// Returns a list of domain concept IDs where this attribute can be used.
    pub fn get_valid_domains_for_attribute(&self, attribute_id: SctId) -> Vec<SctId> {
        self.attribute_domains
            .get(&attribute_id)
            .map(|domains| {
                domains
                    .iter()
                    .filter(|d| d.active)
                    .map(|d| d.domain_id)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets all attributes valid in a domain.
    ///
    /// Returns a list of attribute concept IDs that can be used in this domain.
    pub fn get_valid_attributes_for_domain(&self, domain_concept_id: SctId) -> Vec<SctId> {
        self.attribute_domains
            .iter()
            .filter_map(|(attr_id, domains)| {
                if domains
                    .iter()
                    .any(|d| d.domain_id == domain_concept_id && d.active)
                {
                    Some(*attr_id)
                } else {
                    None
                }
            })
            .collect()
    }

    // Statistics

    /// Returns the number of unique domain concepts.
    pub fn domain_count(&self) -> usize {
        self.domains.len()
    }

    /// Returns the total number of domain records.
    pub fn total_domain_records(&self) -> usize {
        self.domains.values().map(|v| v.len()).sum()
    }

    /// Returns the number of unique attributes with domain definitions.
    pub fn attribute_domain_count(&self) -> usize {
        self.attribute_domains.len()
    }

    /// Returns the total number of attribute domain records.
    pub fn total_attribute_domain_records(&self) -> usize {
        self.attribute_domains.values().map(|v| v.len()).sum()
    }

    /// Returns the number of unique attributes with range definitions.
    pub fn attribute_range_count(&self) -> usize {
        self.attribute_ranges.len()
    }

    /// Returns the total number of attribute range records.
    pub fn total_attribute_range_records(&self) -> usize {
        self.attribute_ranges.values().map(|v| v.len()).sum()
    }

    /// Returns true if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.domains.is_empty()
            && self.attribute_domains.is_empty()
            && self.attribute_ranges.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_types::{Cardinality, well_known};

    fn make_test_domain() -> MrcmDomain {
        MrcmDomain {
            id: "test-domain-1".to_string(),
            effective_time: 20240101,
            active: true,
            module_id: 900000000000207008,
            refset_id: well_known::MRCM_DOMAIN_REFSET,
            referenced_component_id: well_known::CLINICAL_FINDING,
            domain_constraint: "<< 404684003".to_string(),
            parent_domain: None,
            proximal_primitive_constraint: "<< 404684003".to_string(),
            proximal_primitive_refinement: None,
            domain_template_for_precoordination: "template".to_string(),
            domain_template_for_postcoordination: "template".to_string(),
            guide_url: None,
        }
    }

    fn make_test_attribute_domain() -> MrcmAttributeDomain {
        MrcmAttributeDomain {
            id: "test-attr-domain-1".to_string(),
            effective_time: 20240101,
            active: true,
            module_id: 900000000000207008,
            refset_id: well_known::MRCM_ATTRIBUTE_DOMAIN_REFSET,
            referenced_component_id: well_known::FINDING_SITE,
            domain_id: well_known::CLINICAL_FINDING,
            grouped: true,
            attribute_cardinality: Cardinality::unbounded(),
            attribute_in_group_cardinality: Cardinality::optional(),
            rule_strength_id: well_known::MANDATORY_CONCEPT_MODEL_RULE,
            content_type_id: well_known::ALL_SNOMED_CT_CONTENT,
        }
    }

    fn make_test_attribute_range() -> MrcmAttributeRange {
        MrcmAttributeRange {
            id: "test-attr-range-1".to_string(),
            effective_time: 20240101,
            active: true,
            module_id: 900000000000207008,
            refset_id: well_known::MRCM_ATTRIBUTE_RANGE_REFSET,
            referenced_component_id: well_known::FINDING_SITE,
            range_constraint: "<< 123037004 |Body structure|".to_string(),
            attribute_rule: None,
            rule_strength_id: well_known::MANDATORY_CONCEPT_MODEL_RULE,
            content_type_id: well_known::ALL_SNOMED_CT_CONTENT,
        }
    }

    #[test]
    fn test_new_store_is_empty() {
        let store = MrcmStore::new();
        assert!(store.is_empty());
        assert_eq!(store.domain_count(), 0);
        assert_eq!(store.attribute_domain_count(), 0);
        assert_eq!(store.attribute_range_count(), 0);
    }

    #[test]
    fn test_insert_and_retrieve_domain() {
        let mut store = MrcmStore::new();
        let domain = make_test_domain();

        store
            .domains
            .entry(domain.referenced_component_id)
            .or_default()
            .push(domain.clone());

        assert_eq!(store.domain_count(), 1);
        let retrieved = store.get_domains_for_concept(well_known::CLINICAL_FINDING);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().len(), 1);
        assert_eq!(retrieved.unwrap()[0].id, domain.id);
    }

    #[test]
    fn test_insert_and_retrieve_attribute_domain() {
        let mut store = MrcmStore::new();
        let attr_domain = make_test_attribute_domain();

        store
            .attribute_domains
            .entry(attr_domain.referenced_component_id)
            .or_default()
            .push(attr_domain.clone());

        assert_eq!(store.attribute_domain_count(), 1);
        let retrieved = store.get_attribute_domains(well_known::FINDING_SITE);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().len(), 1);
    }

    #[test]
    fn test_insert_and_retrieve_attribute_range() {
        let mut store = MrcmStore::new();
        let attr_range = make_test_attribute_range();

        store
            .attribute_ranges
            .entry(attr_range.referenced_component_id)
            .or_default()
            .push(attr_range.clone());

        assert_eq!(store.attribute_range_count(), 1);
        let retrieved = store.get_attribute_range(well_known::FINDING_SITE);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_is_attribute_valid_for_domain() {
        let mut store = MrcmStore::new();
        let attr_domain = make_test_attribute_domain();

        store
            .attribute_domains
            .entry(attr_domain.referenced_component_id)
            .or_default()
            .push(attr_domain);

        assert!(store.is_attribute_valid_for_domain(
            well_known::FINDING_SITE,
            well_known::CLINICAL_FINDING
        ));

        // Not valid in procedure domain
        assert!(!store.is_attribute_valid_for_domain(
            well_known::FINDING_SITE,
            well_known::PROCEDURE
        ));
    }

    #[test]
    fn test_is_attribute_grouped() {
        let mut store = MrcmStore::new();
        let attr_domain = make_test_attribute_domain();

        store
            .attribute_domains
            .entry(attr_domain.referenced_component_id)
            .or_default()
            .push(attr_domain);

        assert!(store.is_attribute_grouped(well_known::FINDING_SITE));
    }

    #[test]
    fn test_get_range_constraint() {
        let mut store = MrcmStore::new();
        let attr_range = make_test_attribute_range();

        store
            .attribute_ranges
            .entry(attr_range.referenced_component_id)
            .or_default()
            .push(attr_range);

        let constraint = store.get_range_constraint(well_known::FINDING_SITE);
        assert!(constraint.is_some());
        assert!(constraint.unwrap().contains("Body structure"));
    }

    #[test]
    fn test_get_valid_domains_for_attribute() {
        let mut store = MrcmStore::new();
        let attr_domain = make_test_attribute_domain();

        store
            .attribute_domains
            .entry(attr_domain.referenced_component_id)
            .or_default()
            .push(attr_domain);

        let domains = store.get_valid_domains_for_attribute(well_known::FINDING_SITE);
        assert_eq!(domains.len(), 1);
        assert!(domains.contains(&well_known::CLINICAL_FINDING));
    }

    #[test]
    fn test_get_valid_attributes_for_domain() {
        let mut store = MrcmStore::new();
        let attr_domain = make_test_attribute_domain();

        store
            .attribute_domains
            .entry(attr_domain.referenced_component_id)
            .or_default()
            .push(attr_domain);

        let attrs = store.get_valid_attributes_for_domain(well_known::CLINICAL_FINDING);
        assert_eq!(attrs.len(), 1);
        assert!(attrs.contains(&well_known::FINDING_SITE));
    }

    #[test]
    fn test_inactive_records_filtered_in_queries() {
        let mut store = MrcmStore::new();
        let mut attr_domain = make_test_attribute_domain();
        attr_domain.active = false;

        store
            .attribute_domains
            .entry(attr_domain.referenced_component_id)
            .or_default()
            .push(attr_domain);

        // Should not be valid because record is inactive
        assert!(!store.is_attribute_valid_for_domain(
            well_known::FINDING_SITE,
            well_known::CLINICAL_FINDING
        ));

        // Should not be grouped because record is inactive
        assert!(!store.is_attribute_grouped(well_known::FINDING_SITE));
    }

    #[test]
    fn test_statistics() {
        let mut store = MrcmStore::new();

        // Add multiple records
        store
            .domains
            .entry(well_known::CLINICAL_FINDING)
            .or_default()
            .push(make_test_domain());

        let attr_domain1 = make_test_attribute_domain();
        let mut attr_domain2 = make_test_attribute_domain();
        attr_domain2.domain_id = well_known::PROCEDURE;

        store
            .attribute_domains
            .entry(well_known::FINDING_SITE)
            .or_default()
            .push(attr_domain1);
        store
            .attribute_domains
            .entry(well_known::FINDING_SITE)
            .or_default()
            .push(attr_domain2);

        store
            .attribute_ranges
            .entry(well_known::FINDING_SITE)
            .or_default()
            .push(make_test_attribute_range());

        assert_eq!(store.domain_count(), 1);
        assert_eq!(store.total_domain_records(), 1);
        assert_eq!(store.attribute_domain_count(), 1); // 1 unique attribute
        assert_eq!(store.total_attribute_domain_records(), 2); // 2 records for that attribute
        assert_eq!(store.attribute_range_count(), 1);
        assert_eq!(store.total_attribute_range_records(), 1);
        assert!(!store.is_empty());
    }
}
