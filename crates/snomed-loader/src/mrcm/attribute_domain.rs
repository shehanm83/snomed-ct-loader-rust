//! MRCM Attribute Domain reference set parser.
//!
//! Parses files matching pattern: `der2_cRefset_MRCMAttributeDomainSnapshot_*.txt`

use std::path::Path;

use csv::StringRecord;
use snomed_types::{Cardinality, MrcmAttributeDomain};

use crate::parser::{parse, Rf2Parser, Rf2Record};
use crate::types::{Rf2Config, Rf2Error, Rf2Result};

/// Expected columns for MRCM Attribute Domain reference set.
///
/// Order: id, effectiveTime, active, moduleId, refsetId, referencedComponentId,
/// domainId, grouped, attributeCardinality, attributeInGroupCardinality,
/// ruleStrengthId, contentTypeId
const ATTRIBUTE_DOMAIN_COLUMNS: &[&str] = &[
    "id",
    "effectiveTime",
    "active",
    "moduleId",
    "refsetId",
    "referencedComponentId",
    "domainId",
    "grouped",
    "attributeCardinality",
    "attributeInGroupCardinality",
    "ruleStrengthId",
    "contentTypeId",
];

impl Rf2Record for MrcmAttributeDomain {
    const EXPECTED_COLUMNS: &'static [&'static str] = ATTRIBUTE_DOMAIN_COLUMNS;

    fn from_record(record: &StringRecord) -> Rf2Result<Self> {
        let id = record
            .get(0)
            .ok_or_else(|| Rf2Error::MissingColumn {
                column: "id".to_string(),
            })?
            .to_string();

        let effective_time = parse::effective_time(record.get(1).ok_or_else(|| {
            Rf2Error::MissingColumn {
                column: "effectiveTime".to_string(),
            }
        })?)?;

        let active = parse::boolean(record.get(2).ok_or_else(|| Rf2Error::MissingColumn {
            column: "active".to_string(),
        })?)?;

        let module_id = parse::sctid(record.get(3).ok_or_else(|| Rf2Error::MissingColumn {
            column: "moduleId".to_string(),
        })?)?;

        let refset_id = parse::sctid(record.get(4).ok_or_else(|| Rf2Error::MissingColumn {
            column: "refsetId".to_string(),
        })?)?;

        let referenced_component_id =
            parse::sctid(record.get(5).ok_or_else(|| Rf2Error::MissingColumn {
                column: "referencedComponentId".to_string(),
            })?)?;

        let domain_id = parse::sctid(record.get(6).ok_or_else(|| Rf2Error::MissingColumn {
            column: "domainId".to_string(),
        })?)?;

        let grouped = parse::boolean(record.get(7).ok_or_else(|| Rf2Error::MissingColumn {
            column: "grouped".to_string(),
        })?)?;

        let attribute_cardinality_str =
            record.get(8).ok_or_else(|| Rf2Error::MissingColumn {
                column: "attributeCardinality".to_string(),
            })?;
        let attribute_cardinality =
            Cardinality::parse(attribute_cardinality_str).map_err(|_| Rf2Error::InvalidInteger {
                value: attribute_cardinality_str.to_string(),
            })?;

        let attribute_in_group_cardinality_str =
            record.get(9).ok_or_else(|| Rf2Error::MissingColumn {
                column: "attributeInGroupCardinality".to_string(),
            })?;
        let attribute_in_group_cardinality = Cardinality::parse(attribute_in_group_cardinality_str)
            .map_err(|_| Rf2Error::InvalidInteger {
                value: attribute_in_group_cardinality_str.to_string(),
            })?;

        let rule_strength_id =
            parse::sctid(record.get(10).ok_or_else(|| Rf2Error::MissingColumn {
                column: "ruleStrengthId".to_string(),
            })?)?;

        let content_type_id =
            parse::sctid(record.get(11).ok_or_else(|| Rf2Error::MissingColumn {
                column: "contentTypeId".to_string(),
            })?)?;

        Ok(MrcmAttributeDomain {
            id,
            effective_time,
            active,
            module_id,
            refset_id,
            referenced_component_id,
            domain_id,
            grouped,
            attribute_cardinality,
            attribute_in_group_cardinality,
            rule_strength_id,
            content_type_id,
        })
    }

    fn passes_filter(&self, config: &Rf2Config) -> bool {
        if config.active_only && !self.active {
            return false;
        }
        true
    }
}

/// Parses MRCM Attribute Domain reference set from a file.
///
/// # Arguments
/// * `path` - Path to the MRCM Attribute Domain reference set file
/// * `config` - Parser configuration
///
/// # Returns
/// Iterator over parsed `MrcmAttributeDomain` records.
pub fn parse_attribute_domain_file<P: AsRef<Path>>(
    path: P,
    config: Rf2Config,
) -> Rf2Result<impl Iterator<Item = Rf2Result<MrcmAttributeDomain>>> {
    let parser = Rf2Parser::<_, MrcmAttributeDomain>::from_path(path, config)?;
    Ok(parser)
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_types::well_known;

    fn make_test_record() -> StringRecord {
        StringRecord::from(vec![
            "550e8400-e29b-41d4-a716-446655440001",
            "20240101",
            "1",                    // active
            "900000000000207008",   // module_id
            "723604009",            // MRCM Attribute Domain refset
            "363698007",            // Finding site attribute
            "404684003",            // Clinical finding domain
            "1",                    // grouped = true
            "0..*",                 // attribute cardinality
            "0..1",                 // attribute in group cardinality
            "723597001",            // Mandatory rule
            "723596005",            // All SNOMED CT content
        ])
    }

    #[test]
    fn test_parse_mrcm_attribute_domain() {
        let record = make_test_record();
        let attr_domain = MrcmAttributeDomain::from_record(&record).unwrap();

        assert_eq!(attr_domain.id, "550e8400-e29b-41d4-a716-446655440001");
        assert_eq!(attr_domain.effective_time, 20240101);
        assert!(attr_domain.active);
        assert_eq!(attr_domain.module_id, 900000000000207008);
        assert_eq!(attr_domain.refset_id, well_known::MRCM_ATTRIBUTE_DOMAIN_REFSET);
        assert_eq!(attr_domain.referenced_component_id, well_known::FINDING_SITE);
        assert_eq!(attr_domain.domain_id, well_known::CLINICAL_FINDING);
        assert!(attr_domain.grouped);
        assert_eq!(attr_domain.attribute_cardinality, Cardinality::unbounded());
        assert_eq!(attr_domain.attribute_in_group_cardinality, Cardinality::optional());
        assert_eq!(attr_domain.rule_strength_id, well_known::MANDATORY_CONCEPT_MODEL_RULE);
        assert_eq!(attr_domain.content_type_id, well_known::ALL_SNOMED_CT_CONTENT);
    }

    #[test]
    fn test_is_mandatory() {
        let record = make_test_record();
        let attr_domain = MrcmAttributeDomain::from_record(&record).unwrap();

        assert!(attr_domain.is_mandatory());
    }

    #[test]
    fn test_is_optional() {
        let record = StringRecord::from(vec![
            "550e8400-e29b-41d4-a716-446655440001",
            "20240101",
            "1",
            "900000000000207008",
            "723604009",
            "363698007",
            "404684003",
            "1",
            "0..*",
            "0..1",
            "723598006", // Optional rule
            "723596005",
        ]);

        let attr_domain = MrcmAttributeDomain::from_record(&record).unwrap();
        assert!(!attr_domain.is_mandatory());
    }

    #[test]
    fn test_is_grouped() {
        let record = make_test_record();
        let attr_domain = MrcmAttributeDomain::from_record(&record).unwrap();
        assert!(attr_domain.is_grouped());

        let record_ungrouped = StringRecord::from(vec![
            "550e8400-e29b-41d4-a716-446655440001",
            "20240101",
            "1",
            "900000000000207008",
            "723604009",
            "363698007",
            "404684003",
            "0", // Not grouped
            "0..*",
            "0..1",
            "723597001",
            "723596005",
        ]);

        let attr_domain_ungrouped = MrcmAttributeDomain::from_record(&record_ungrouped).unwrap();
        assert!(!attr_domain_ungrouped.is_grouped());
    }

    #[test]
    fn test_cardinality_parsing() {
        // Test different cardinality values
        let record = StringRecord::from(vec![
            "550e8400-e29b-41d4-a716-446655440001",
            "20240101",
            "1",
            "900000000000207008",
            "723604009",
            "363698007",
            "404684003",
            "1",
            "1..*", // At least one
            "1..1", // Exactly one in group
            "723597001",
            "723596005",
        ]);

        let attr_domain = MrcmAttributeDomain::from_record(&record).unwrap();
        assert_eq!(attr_domain.attribute_cardinality, Cardinality::one_or_more());
        assert_eq!(attr_domain.attribute_in_group_cardinality, Cardinality::required());
    }

    #[test]
    fn test_filter_inactive() {
        let record = StringRecord::from(vec![
            "550e8400-e29b-41d4-a716-446655440001",
            "20240101",
            "0", // Inactive
            "900000000000207008",
            "723604009",
            "363698007",
            "404684003",
            "1",
            "0..*",
            "0..1",
            "723597001",
            "723596005",
        ]);

        let attr_domain = MrcmAttributeDomain::from_record(&record).unwrap();

        let config_active_only = Rf2Config {
            active_only: true,
            ..Default::default()
        };
        assert!(!attr_domain.passes_filter(&config_active_only));

        let config_all = Rf2Config {
            active_only: false,
            ..Default::default()
        };
        assert!(attr_domain.passes_filter(&config_all));
    }
}
