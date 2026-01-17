//! SNOMED CT Relationship file parser.
//!
//! Parses sct2_Relationship_*.txt RF2 files.

use csv::StringRecord;
use snomed_types::Rf2Relationship;

use crate::parser::{parse, Rf2Record};
use crate::types::{RelationshipConfig, Rf2Config, Rf2Result};

/// Expected columns in a relationship file.
const RELATIONSHIP_COLUMNS: &[&str] = &[
    "id",
    "effectiveTime",
    "active",
    "moduleId",
    "sourceId",
    "destinationId",
    "relationshipGroup",
    "typeId",
    "characteristicTypeId",
    "modifierId",
];

impl Rf2Record for Rf2Relationship {
    const EXPECTED_COLUMNS: &'static [&'static str] = RELATIONSHIP_COLUMNS;

    fn from_record(record: &StringRecord) -> Rf2Result<Self> {
        Ok(Rf2Relationship {
            id: parse::sctid(record.get(0).unwrap_or(""))?,
            effective_time: parse::effective_time(record.get(1).unwrap_or(""))?,
            active: parse::boolean(record.get(2).unwrap_or(""))?,
            module_id: parse::sctid(record.get(3).unwrap_or(""))?,
            source_id: parse::sctid(record.get(4).unwrap_or(""))?,
            destination_id: parse::sctid(record.get(5).unwrap_or(""))?,
            relationship_group: parse::integer(record.get(6).unwrap_or(""))?,
            type_id: parse::sctid(record.get(7).unwrap_or(""))?,
            characteristic_type_id: parse::sctid(record.get(8).unwrap_or(""))?,
            modifier_id: parse::sctid(record.get(9).unwrap_or(""))?,
        })
    }

    fn passes_filter(&self, config: &Rf2Config) -> bool {
        if config.active_only && !self.active {
            return false;
        }
        true
    }
}

/// Extended filter for relationships with type and characteristic filtering.
pub trait RelationshipFilter {
    /// Returns true if the relationship passes the extended filter.
    fn passes_relationship_filter(&self, config: &RelationshipConfig) -> bool;
}

impl RelationshipFilter for Rf2Relationship {
    fn passes_relationship_filter(&self, config: &RelationshipConfig) -> bool {
        // Check base filter first
        if !self.passes_filter(&config.base) {
            return false;
        }

        // Check relationship type filter
        if !config.type_ids.is_empty() && !config.type_ids.contains(&self.type_id) {
            return false;
        }

        // Check characteristic type filter
        if !config.characteristic_type_ids.is_empty()
            && !config.characteristic_type_ids.contains(&self.characteristic_type_id)
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_types::CharacteristicType;

    fn make_record(fields: &[&str]) -> StringRecord {
        let mut record = StringRecord::new();
        for field in fields {
            record.push_field(field);
        }
        record
    }

    #[test]
    fn test_parse_relationship_record() {
        let record = make_record(&[
            "100000028",
            "20020131",
            "1",
            "900000000000207008",
            "73211009",
            "362969004",
            "0",
            "116680003",
            "900000000000011006",
            "900000000000451002",
        ]);

        let rel = Rf2Relationship::from_record(&record).unwrap();
        assert_eq!(rel.id, 100000028);
        assert_eq!(rel.source_id, 73211009);
        assert_eq!(rel.destination_id, 362969004);
        assert_eq!(rel.relationship_group, 0);
        assert!(rel.is_is_a());
        assert!(rel.is_inferred());
    }

    #[test]
    fn test_characteristic_type_filter() {
        let inferred = Rf2Relationship {
            id: 1,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            source_id: 73211009,
            destination_id: 362969004,
            relationship_group: 0,
            type_id: 116680003,
            characteristic_type_id: CharacteristicType::INFERRED_ID,
            modifier_id: 900000000000451002,
        };

        let stated = Rf2Relationship {
            characteristic_type_id: CharacteristicType::STATED_ID,
            ..inferred.clone()
        };

        let inferred_only = RelationshipConfig::inferred_only();

        assert!(inferred.passes_relationship_filter(&inferred_only));
        assert!(!stated.passes_relationship_filter(&inferred_only));
    }

    #[test]
    fn test_type_filter() {
        let is_a = Rf2Relationship {
            id: 1,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            source_id: 73211009,
            destination_id: 362969004,
            relationship_group: 0,
            type_id: 116680003, // IS_A
            characteristic_type_id: 900000000000011006,
            modifier_id: 900000000000451002,
        };

        let finding_site = Rf2Relationship {
            type_id: 363698007, // Finding site
            ..is_a.clone()
        };

        let is_a_only = RelationshipConfig::is_a_only();

        assert!(is_a.passes_relationship_filter(&is_a_only));
        assert!(!finding_site.passes_relationship_filter(&is_a_only));
    }
}
