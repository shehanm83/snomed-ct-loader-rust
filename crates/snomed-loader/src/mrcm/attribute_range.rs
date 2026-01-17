//! MRCM Attribute Range reference set parser.
//!
//! Parses files matching pattern: `der2_cRefset_MRCMAttributeRangeSnapshot_*.txt`

use std::path::Path;

use csv::StringRecord;
use snomed_types::MrcmAttributeRange;

use crate::parser::{parse, Rf2Parser, Rf2Record};
use crate::types::{Rf2Config, Rf2Error, Rf2Result};

/// Expected columns for MRCM Attribute Range reference set.
///
/// Order: id, effectiveTime, active, moduleId, refsetId, referencedComponentId,
/// rangeConstraint, attributeRule, ruleStrengthId, contentTypeId
const ATTRIBUTE_RANGE_COLUMNS: &[&str] = &[
    "id",
    "effectiveTime",
    "active",
    "moduleId",
    "refsetId",
    "referencedComponentId",
    "rangeConstraint",
    "attributeRule",
    "ruleStrengthId",
    "contentTypeId",
];

impl Rf2Record for MrcmAttributeRange {
    const EXPECTED_COLUMNS: &'static [&'static str] = ATTRIBUTE_RANGE_COLUMNS;

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

        let range_constraint = record
            .get(6)
            .ok_or_else(|| Rf2Error::MissingColumn {
                column: "rangeConstraint".to_string(),
            })?
            .to_string();

        let attribute_rule_str = record.get(7).ok_or_else(|| Rf2Error::MissingColumn {
            column: "attributeRule".to_string(),
        })?;
        let attribute_rule = if attribute_rule_str.is_empty() {
            None
        } else {
            Some(attribute_rule_str.to_string())
        };

        let rule_strength_id =
            parse::sctid(record.get(8).ok_or_else(|| Rf2Error::MissingColumn {
                column: "ruleStrengthId".to_string(),
            })?)?;

        let content_type_id =
            parse::sctid(record.get(9).ok_or_else(|| Rf2Error::MissingColumn {
                column: "contentTypeId".to_string(),
            })?)?;

        Ok(MrcmAttributeRange {
            id,
            effective_time,
            active,
            module_id,
            refset_id,
            referenced_component_id,
            range_constraint,
            attribute_rule,
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

/// Parses MRCM Attribute Range reference set from a file.
///
/// # Arguments
/// * `path` - Path to the MRCM Attribute Range reference set file
/// * `config` - Parser configuration
///
/// # Returns
/// Iterator over parsed `MrcmAttributeRange` records.
pub fn parse_attribute_range_file<P: AsRef<Path>>(
    path: P,
    config: Rf2Config,
) -> Rf2Result<impl Iterator<Item = Rf2Result<MrcmAttributeRange>>> {
    let parser = Rf2Parser::<_, MrcmAttributeRange>::from_path(path, config)?;
    Ok(parser)
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_types::well_known;

    fn make_test_record() -> StringRecord {
        StringRecord::from(vec![
            "550e8400-e29b-41d4-a716-446655440002",
            "20240101",
            "1",                                        // active
            "900000000000207008",                       // module_id
            "723592007",                                // MRCM Attribute Range refset
            "363698007",                                // Finding site attribute
            "<< 123037004 |Body structure|",            // range constraint
            "",                                         // no attribute rule
            "723597001",                                // Mandatory rule
            "723596005",                                // All SNOMED CT content
        ])
    }

    #[test]
    fn test_parse_mrcm_attribute_range() {
        let record = make_test_record();
        let attr_range = MrcmAttributeRange::from_record(&record).unwrap();

        assert_eq!(attr_range.id, "550e8400-e29b-41d4-a716-446655440002");
        assert_eq!(attr_range.effective_time, 20240101);
        assert!(attr_range.active);
        assert_eq!(attr_range.module_id, 900000000000207008);
        assert_eq!(attr_range.refset_id, well_known::MRCM_ATTRIBUTE_RANGE_REFSET);
        assert_eq!(attr_range.referenced_component_id, well_known::FINDING_SITE);
        assert_eq!(attr_range.range_constraint, "<< 123037004 |Body structure|");
        assert!(attr_range.attribute_rule.is_none());
        assert_eq!(attr_range.rule_strength_id, well_known::MANDATORY_CONCEPT_MODEL_RULE);
        assert_eq!(attr_range.content_type_id, well_known::ALL_SNOMED_CT_CONTENT);
    }

    #[test]
    fn test_with_attribute_rule() {
        let record = StringRecord::from(vec![
            "550e8400-e29b-41d4-a716-446655440002",
            "20240101",
            "1",
            "900000000000207008",
            "723592007",
            "363698007",
            "<< 123037004 |Body structure|",
            "Some additional rule text", // Has attribute rule
            "723597001",
            "723596005",
        ]);

        let attr_range = MrcmAttributeRange::from_record(&record).unwrap();
        assert!(attr_range.has_attribute_rule());
        assert_eq!(attr_range.attribute_rule.as_deref(), Some("Some additional rule text"));
    }

    #[test]
    fn test_is_mandatory() {
        let record = make_test_record();
        let attr_range = MrcmAttributeRange::from_record(&record).unwrap();
        assert!(attr_range.is_mandatory());
    }

    #[test]
    fn test_is_optional() {
        let record = StringRecord::from(vec![
            "550e8400-e29b-41d4-a716-446655440002",
            "20240101",
            "1",
            "900000000000207008",
            "723592007",
            "363698007",
            "<< 123037004",
            "",
            "723598006", // Optional rule
            "723596005",
        ]);

        let attr_range = MrcmAttributeRange::from_record(&record).unwrap();
        assert!(!attr_range.is_mandatory());
    }

    #[test]
    fn test_filter_inactive() {
        let record = StringRecord::from(vec![
            "550e8400-e29b-41d4-a716-446655440002",
            "20240101",
            "0", // Inactive
            "900000000000207008",
            "723592007",
            "363698007",
            "<< 123037004",
            "",
            "723597001",
            "723596005",
        ]);

        let attr_range = MrcmAttributeRange::from_record(&record).unwrap();

        let config_active_only = Rf2Config {
            active_only: true,
            ..Default::default()
        };
        assert!(!attr_range.passes_filter(&config_active_only));

        let config_all = Rf2Config {
            active_only: false,
            ..Default::default()
        };
        assert!(attr_range.passes_filter(&config_all));
    }

    #[test]
    fn test_complex_range_constraint() {
        // Test with a more complex ECL expression
        let record = StringRecord::from(vec![
            "550e8400-e29b-41d4-a716-446655440002",
            "20240101",
            "1",
            "900000000000207008",
            "723592007",
            "246112005", // Severity attribute
            "<< 272141005 |Severities (qualifier value)| OR << 371928007 |Severity modifier (attribute)|",
            "",
            "723597001",
            "723596005",
        ]);

        let attr_range = MrcmAttributeRange::from_record(&record).unwrap();
        assert_eq!(attr_range.referenced_component_id, well_known::SEVERITY);
        assert!(attr_range.range_constraint.contains("OR"));
    }
}
