//! OWL Expression refset parser for SNOMED CT RF2 files.
//!
//! Parses `sct2_sRefset_OWL*.txt` files containing OWL axiom expressions.

use csv::StringRecord;
use snomed_types::Rf2OwlExpression;

use crate::parser::{parse, Rf2Record};
use crate::types::{Rf2Config, Rf2Result};

/// Expected column headers for OWL expression refset files.
const OWL_EXPRESSION_COLUMNS: &[&str] = &[
    "id",
    "effectiveTime",
    "active",
    "moduleId",
    "refsetId",
    "referencedComponentId",
    "owlExpression",
];

impl Rf2Record for Rf2OwlExpression {
    const EXPECTED_COLUMNS: &'static [&'static str] = OWL_EXPRESSION_COLUMNS;

    fn from_record(record: &StringRecord) -> Rf2Result<Self> {
        Ok(Rf2OwlExpression {
            id: parse::sctid(record.get(0).unwrap_or(""))?,
            effective_time: parse::effective_time(record.get(1).unwrap_or(""))?,
            active: parse::boolean(record.get(2).unwrap_or(""))?,
            module_id: parse::sctid(record.get(3).unwrap_or(""))?,
            refset_id: parse::sctid(record.get(4).unwrap_or(""))?,
            referenced_component_id: parse::sctid(record.get(5).unwrap_or(""))?,
            owl_expression: record.get(6).unwrap_or("").to_string(),
        })
    }

    fn passes_filter(&self, config: &Rf2Config) -> bool {
        if config.active_only && !self.active {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(fields: &[&str]) -> StringRecord {
        let mut record = StringRecord::new();
        for field in fields {
            record.push_field(field);
        }
        record
    }

    #[test]
    fn test_parse_owl_expression_record() {
        let record = make_record(&[
            "12345678901234",                           // id
            "20230101",                                 // effectiveTime
            "1",                                        // active
            "900000000000207008",                       // moduleId
            "733073007",                                // refsetId (OWL Axiom)
            "404684003",                                // referencedComponentId
            "SubClassOf(:404684003 :138875005)",       // owlExpression
        ]);

        let owl = Rf2OwlExpression::from_record(&record).unwrap();

        assert_eq!(owl.id, 12345678901234);
        assert_eq!(owl.effective_time, 20230101);
        assert!(owl.active);
        assert_eq!(owl.module_id, 900000000000207008);
        assert_eq!(owl.refset_id, Rf2OwlExpression::OWL_AXIOM_REFSET_ID);
        assert_eq!(owl.referenced_component_id, 404684003);
        assert_eq!(owl.owl_expression, "SubClassOf(:404684003 :138875005)");
        assert!(owl.is_subclass_axiom());
    }

    #[test]
    fn test_parse_inactive_owl_expression() {
        let record = make_record(&[
            "12345678901234",
            "20230101",
            "0",  // inactive
            "900000000000207008",
            "733073007",
            "404684003",
            "SubClassOf(:404684003 :138875005)",
        ]);

        let owl = Rf2OwlExpression::from_record(&record).unwrap();
        assert!(!owl.active);
    }

    #[test]
    fn test_filter_active_only() {
        let record = make_record(&[
            "12345678901234",
            "20230101",
            "0",  // inactive
            "900000000000207008",
            "733073007",
            "404684003",
            "SubClassOf(:404684003 :138875005)",
        ]);

        let owl = Rf2OwlExpression::from_record(&record).unwrap();

        let config = Rf2Config { active_only: true, ..Rf2Config::default() };
        assert!(!owl.passes_filter(&config));

        let config = Rf2Config { active_only: false, ..Rf2Config::default() };
        assert!(owl.passes_filter(&config));
    }
}
