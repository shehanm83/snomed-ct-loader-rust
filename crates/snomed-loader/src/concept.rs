//! SNOMED CT Concept file parser.
//!
//! Parses sct2_Concept_*.txt RF2 files.

use csv::StringRecord;
use snomed_types::Rf2Concept;

use crate::parser::{parse, Rf2Record};
use crate::types::{Rf2Config, Rf2Result};

/// Expected columns in a concept file.
const CONCEPT_COLUMNS: &[&str] = &[
    "id",
    "effectiveTime",
    "active",
    "moduleId",
    "definitionStatusId",
];

impl Rf2Record for Rf2Concept {
    const EXPECTED_COLUMNS: &'static [&'static str] = CONCEPT_COLUMNS;

    fn from_record(record: &StringRecord) -> Rf2Result<Self> {
        Ok(Rf2Concept {
            id: parse::sctid(record.get(0).unwrap_or(""))?,
            effective_time: parse::effective_time(record.get(1).unwrap_or(""))?,
            active: parse::boolean(record.get(2).unwrap_or(""))?,
            module_id: parse::sctid(record.get(3).unwrap_or(""))?,
            definition_status_id: parse::sctid(record.get(4).unwrap_or(""))?,
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
    fn test_parse_concept_record() {
        let record = make_record(&[
            "404684003",
            "20020131",
            "1",
            "900000000000207008",
            "900000000000074008",
        ]);

        let concept = Rf2Concept::from_record(&record).unwrap();
        assert_eq!(concept.id, 404684003);
        assert_eq!(concept.effective_time, 20020131);
        assert!(concept.active);
        assert_eq!(concept.module_id, 900000000000207008);
        assert_eq!(concept.definition_status_id, 900000000000074008);
        assert!(concept.is_primitive());
    }

    #[test]
    fn test_parse_inactive_concept() {
        let record = make_record(&[
            "100005",
            "20020131",
            "0",
            "900000000000207008",
            "900000000000074008",
        ]);

        let concept = Rf2Concept::from_record(&record).unwrap();
        assert!(!concept.active);
    }

    #[test]
    fn test_filter_active_only() {
        let active_concept = Rf2Concept {
            id: 1,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            definition_status_id: 900000000000074008,
        };

        let inactive_concept = Rf2Concept {
            id: 2,
            effective_time: 20020131,
            active: false,
            module_id: 900000000000207008,
            definition_status_id: 900000000000074008,
        };

        let active_only_config = Rf2Config {
            active_only: true,
            ..Default::default()
        };

        let all_config = Rf2Config {
            active_only: false,
            ..Default::default()
        };

        assert!(active_concept.passes_filter(&active_only_config));
        assert!(!inactive_concept.passes_filter(&active_only_config));
        assert!(active_concept.passes_filter(&all_config));
        assert!(inactive_concept.passes_filter(&all_config));
    }
}
