//! SNOMED CT Description file parser.
//!
//! Parses sct2_Description_*.txt RF2 files.

use csv::StringRecord;
use snomed_types::Rf2Description;

use crate::parser::{parse, Rf2Record};
use crate::types::{DescriptionConfig, Rf2Config, Rf2Result};

/// Expected columns in a description file.
const DESCRIPTION_COLUMNS: &[&str] = &[
    "id",
    "effectiveTime",
    "active",
    "moduleId",
    "conceptId",
    "languageCode",
    "typeId",
    "term",
    "caseSignificanceId",
];

impl Rf2Record for Rf2Description {
    const EXPECTED_COLUMNS: &'static [&'static str] = DESCRIPTION_COLUMNS;

    fn from_record(record: &StringRecord) -> Rf2Result<Self> {
        Ok(Rf2Description {
            id: parse::sctid(record.get(0).unwrap_or(""))?,
            effective_time: parse::effective_time(record.get(1).unwrap_or(""))?,
            active: parse::boolean(record.get(2).unwrap_or(""))?,
            module_id: parse::sctid(record.get(3).unwrap_or(""))?,
            concept_id: parse::sctid(record.get(4).unwrap_or(""))?,
            language_code: record.get(5).unwrap_or("").to_string(),
            type_id: parse::sctid(record.get(6).unwrap_or(""))?,
            term: record.get(7).unwrap_or("").to_string(),
            case_significance_id: parse::sctid(record.get(8).unwrap_or(""))?,
        })
    }

    fn passes_filter(&self, config: &Rf2Config) -> bool {
        if config.active_only && !self.active {
            return false;
        }
        true
    }
}

/// Extended filter for descriptions with language and type filtering.
pub trait DescriptionFilter {
    /// Returns true if the description passes the extended filter.
    fn passes_description_filter(&self, config: &DescriptionConfig) -> bool;
}

impl DescriptionFilter for Rf2Description {
    fn passes_description_filter(&self, config: &DescriptionConfig) -> bool {
        // Check base filter first
        if !self.passes_filter(&config.base) {
            return false;
        }

        // Check language filter
        if !config.language_codes.is_empty()
            && !config.language_codes.contains(&self.language_code)
        {
            return false;
        }

        // Check type filter
        if !config.type_ids.is_empty() && !config.type_ids.contains(&self.type_id) {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_types::DescriptionType;

    fn make_record(fields: &[&str]) -> StringRecord {
        let mut record = StringRecord::new();
        for field in fields {
            record.push_field(field);
        }
        record
    }

    #[test]
    fn test_parse_description_record() {
        let record = make_record(&[
            "754786011",
            "20020131",
            "1",
            "900000000000207008",
            "73211009",
            "en",
            "900000000000003001",
            "Diabetes mellitus (disorder)",
            "900000000000448009",
        ]);

        let desc = Rf2Description::from_record(&record).unwrap();
        assert_eq!(desc.id, 754786011);
        assert_eq!(desc.effective_time, 20020131);
        assert!(desc.active);
        assert_eq!(desc.concept_id, 73211009);
        assert_eq!(desc.language_code, "en");
        assert_eq!(desc.type_id, DescriptionType::FSN_ID);
        assert_eq!(desc.term, "Diabetes mellitus (disorder)");
        assert!(desc.is_fsn());
    }

    #[test]
    fn test_language_filter() {
        let english_desc = Rf2Description {
            id: 1,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            concept_id: 73211009,
            language_code: "en".to_string(),
            type_id: 900000000000003001,
            term: "Test term".to_string(),
            case_significance_id: 900000000000448009,
        };

        let spanish_desc = Rf2Description {
            language_code: "es".to_string(),
            ..english_desc.clone()
        };

        let english_only = DescriptionConfig {
            language_codes: vec!["en".to_string()],
            ..Default::default()
        };

        assert!(english_desc.passes_description_filter(&english_only));
        assert!(!spanish_desc.passes_description_filter(&english_only));
    }

    #[test]
    fn test_type_filter() {
        let fsn = Rf2Description {
            id: 1,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            concept_id: 73211009,
            language_code: "en".to_string(),
            type_id: 900000000000003001, // FSN
            term: "Test (finding)".to_string(),
            case_significance_id: 900000000000448009,
        };

        let synonym = Rf2Description {
            type_id: 900000000000013009, // Synonym
            term: "Test".to_string(),
            ..fsn.clone()
        };

        let fsn_only = DescriptionConfig::fsn_only();

        assert!(fsn.passes_description_filter(&fsn_only));
        assert!(!synonym.passes_description_filter(&fsn_only));
    }
}
