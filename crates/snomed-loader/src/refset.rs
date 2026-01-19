//! RF2 Reference Set parser implementations.
//!
//! This module provides parsers for simple, language, and association reference sets.

use csv::StringRecord;
use snomed_types::{Rf2AssociationRefsetMember, Rf2LanguageRefsetMember, Rf2SimpleRefsetMember};

use crate::parser::{parse, Rf2Record};
use crate::types::{Rf2Config, Rf2Result};

/// Expected columns for simple reference set files.
const SIMPLE_REFSET_COLUMNS: &[&str] = &[
    "id",
    "effectiveTime",
    "active",
    "moduleId",
    "refsetId",
    "referencedComponentId",
];

/// Expected columns for language reference set files.
const LANGUAGE_REFSET_COLUMNS: &[&str] = &[
    "id",
    "effectiveTime",
    "active",
    "moduleId",
    "refsetId",
    "referencedComponentId",
    "acceptabilityId",
];

/// Expected columns for association reference set files.
const ASSOCIATION_REFSET_COLUMNS: &[&str] = &[
    "id",
    "effectiveTime",
    "active",
    "moduleId",
    "refsetId",
    "referencedComponentId",
    "targetComponentId",
];

impl Rf2Record for Rf2SimpleRefsetMember {
    const EXPECTED_COLUMNS: &'static [&'static str] = SIMPLE_REFSET_COLUMNS;

    fn from_record(record: &StringRecord) -> Rf2Result<Self> {
        Ok(Self {
            id: parse::sctid(record.get(0).unwrap_or(""))?,
            effective_time: parse::effective_time(record.get(1).unwrap_or(""))?,
            active: parse::boolean(record.get(2).unwrap_or(""))?,
            module_id: parse::sctid(record.get(3).unwrap_or(""))?,
            refset_id: parse::sctid(record.get(4).unwrap_or(""))?,
            referenced_component_id: parse::sctid(record.get(5).unwrap_or(""))?,
        })
    }

    fn passes_filter(&self, config: &Rf2Config) -> bool {
        if config.active_only && !self.active {
            return false;
        }
        true
    }
}

impl Rf2Record for Rf2LanguageRefsetMember {
    const EXPECTED_COLUMNS: &'static [&'static str] = LANGUAGE_REFSET_COLUMNS;

    fn from_record(record: &StringRecord) -> Rf2Result<Self> {
        Ok(Self {
            id: parse::sctid(record.get(0).unwrap_or(""))?,
            effective_time: parse::effective_time(record.get(1).unwrap_or(""))?,
            active: parse::boolean(record.get(2).unwrap_or(""))?,
            module_id: parse::sctid(record.get(3).unwrap_or(""))?,
            refset_id: parse::sctid(record.get(4).unwrap_or(""))?,
            referenced_component_id: parse::sctid(record.get(5).unwrap_or(""))?,
            acceptability_id: parse::sctid(record.get(6).unwrap_or(""))?,
        })
    }

    fn passes_filter(&self, config: &Rf2Config) -> bool {
        if config.active_only && !self.active {
            return false;
        }
        true
    }
}

impl Rf2Record for Rf2AssociationRefsetMember {
    const EXPECTED_COLUMNS: &'static [&'static str] = ASSOCIATION_REFSET_COLUMNS;

    fn from_record(record: &StringRecord) -> Rf2Result<Self> {
        Ok(Self {
            id: parse::sctid(record.get(0).unwrap_or(""))?,
            effective_time: parse::effective_time(record.get(1).unwrap_or(""))?,
            active: parse::boolean(record.get(2).unwrap_or(""))?,
            module_id: parse::sctid(record.get(3).unwrap_or(""))?,
            refset_id: parse::sctid(record.get(4).unwrap_or(""))?,
            referenced_component_id: parse::sctid(record.get(5).unwrap_or(""))?,
            target_component_id: parse::sctid(record.get(6).unwrap_or(""))?,
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

    #[test]
    fn test_parse_simple_refset_member() {
        let record = StringRecord::from(vec![
            "12345678901",           // id
            "20200101",              // effectiveTime
            "1",                     // active
            "900000000000207008",    // moduleId
            "723264001",             // refsetId
            "12345678",              // referencedComponentId
        ]);

        let member = Rf2SimpleRefsetMember::from_record(&record).unwrap();
        assert_eq!(member.id, 12345678901);
        assert!(member.active);
        assert_eq!(member.refset_id, 723264001);
        assert_eq!(member.referenced_component_id, 12345678);
    }

    #[test]
    fn test_parse_language_refset_member() {
        let record = StringRecord::from(vec![
            "12345678901",           // id
            "20200101",              // effectiveTime
            "1",                     // active
            "900000000000207008",    // moduleId
            "900000000000509007",    // refsetId (US English)
            "12345678",              // referencedComponentId
            "900000000000548007",    // acceptabilityId (Preferred)
        ]);

        let member = Rf2LanguageRefsetMember::from_record(&record).unwrap();
        assert_eq!(member.id, 12345678901);
        assert!(member.active);
        assert!(member.is_preferred());
        assert!(!member.is_acceptable());
    }

    #[test]
    fn test_simple_refset_filter_active_only() {
        let active_member = Rf2SimpleRefsetMember {
            id: 1,
            effective_time: 20200101,
            active: true,
            module_id: 900000000000207008,
            refset_id: 723264001,
            referenced_component_id: 12345,
        };

        let inactive_member = Rf2SimpleRefsetMember {
            id: 2,
            effective_time: 20200101,
            active: false,
            module_id: 900000000000207008,
            refset_id: 723264001,
            referenced_component_id: 12346,
        };

        let active_config = Rf2Config {
            active_only: true,
            batch_size: 1000,
        };

        let all_config = Rf2Config {
            active_only: false,
            batch_size: 1000,
        };

        assert!(active_member.passes_filter(&active_config));
        assert!(!inactive_member.passes_filter(&active_config));
        assert!(active_member.passes_filter(&all_config));
        assert!(inactive_member.passes_filter(&all_config));
    }

    #[test]
    fn test_parse_association_refset_member() {
        let record = StringRecord::from(vec![
            "12345678901",           // id
            "20200101",              // effectiveTime
            "1",                     // active
            "900000000000207008",    // moduleId
            "900000000000527005",    // refsetId (SAME AS)
            "12345678",              // referencedComponentId
            "87654321",              // targetComponentId
        ]);

        let member = Rf2AssociationRefsetMember::from_record(&record).unwrap();
        assert_eq!(member.id, 12345678901);
        assert!(member.active);
        assert_eq!(member.refset_id, Rf2AssociationRefsetMember::SAME_AS_REFSET);
        assert_eq!(member.referenced_component_id, 12345678);
        assert_eq!(member.target_component_id, 87654321);
        assert!(member.is_same_as_association());
        assert!(member.is_historical_association());
    }

    #[test]
    fn test_parse_replaced_by_association() {
        let record = StringRecord::from(vec![
            "12345678901",
            "20200101",
            "1",
            "900000000000207008",
            "900000000000526001",    // REPLACED BY
            "12345678",
            "87654321",
        ]);

        let member = Rf2AssociationRefsetMember::from_record(&record).unwrap();
        assert!(member.is_replaced_by_association());
        assert!(member.is_historical_association());
        assert!(!member.is_same_as_association());
    }
}
