//! Concrete relationship parser for SNOMED CT RF2 files.
//!
//! Parses `sct2_RelationshipConcreteValues_*.txt` files containing
//! relationships with concrete values (strings, integers, decimals).

use csv::StringRecord;
use snomed_types::{ConcreteValue, Rf2ConcreteRelationship};

use crate::parser::{parse, Rf2Record};
use crate::types::{Rf2Config, Rf2Result, Rf2Error};

/// Expected column headers for concrete relationship files.
const CONCRETE_RELATIONSHIP_COLUMNS: &[&str] = &[
    "id",
    "effectiveTime",
    "active",
    "moduleId",
    "sourceId",
    "value",
    "relationshipGroup",
    "typeId",
    "characteristicTypeId",
    "modifierId",
];

impl Rf2Record for Rf2ConcreteRelationship {
    const EXPECTED_COLUMNS: &'static [&'static str] = CONCRETE_RELATIONSHIP_COLUMNS;

    fn from_record(record: &StringRecord) -> Rf2Result<Self> {
        let value_str = record.get(5).unwrap_or("");
        let value = ConcreteValue::parse(value_str)
            .ok_or_else(|| Rf2Error::Parse(format!("Invalid concrete value: {}", value_str)))?;

        Ok(Rf2ConcreteRelationship {
            id: parse::sctid(record.get(0).unwrap_or(""))?,
            effective_time: parse::effective_time(record.get(1).unwrap_or(""))?,
            active: parse::boolean(record.get(2).unwrap_or(""))?,
            module_id: parse::sctid(record.get(3).unwrap_or(""))?,
            source_id: parse::sctid(record.get(4).unwrap_or(""))?,
            value,
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

/// Configuration for filtering concrete relationships during parsing.
#[derive(Debug, Clone)]
pub struct ConcreteRelationshipConfig {
    /// Base RF2 configuration.
    pub base: Rf2Config,
    /// Filter to specific relationship types (empty = all types).
    pub type_ids: Vec<u64>,
    /// If true, only include inferred relationships.
    pub inferred_only: bool,
    /// If true, only include stated relationships.
    pub stated_only: bool,
}

impl Default for ConcreteRelationshipConfig {
    fn default() -> Self {
        Self {
            base: Rf2Config::default(),
            type_ids: vec![],
            inferred_only: false,
            stated_only: false,
        }
    }
}

impl ConcreteRelationshipConfig {
    /// Creates a config that only includes inferred relationships.
    pub fn inferred_only() -> Self {
        Self {
            inferred_only: true,
            ..Default::default()
        }
    }

    /// Creates a config that only includes stated relationships.
    pub fn stated_only() -> Self {
        Self {
            stated_only: true,
            ..Default::default()
        }
    }
}

/// Trait for filtering concrete relationships.
pub trait ConcreteRelationshipFilter {
    /// Returns true if this relationship passes the filter.
    fn passes_concrete_filter(&self, config: &ConcreteRelationshipConfig) -> bool;
}

impl ConcreteRelationshipFilter for Rf2ConcreteRelationship {
    fn passes_concrete_filter(&self, config: &ConcreteRelationshipConfig) -> bool {
        // Check base filter
        if !self.passes_filter(&config.base) {
            return false;
        }

        // Check type filter
        if !config.type_ids.is_empty() && !config.type_ids.contains(&self.type_id) {
            return false;
        }

        // Check characteristic type filters
        if config.inferred_only && !self.is_inferred() {
            return false;
        }
        if config.stated_only && !self.is_stated() {
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
    fn test_parse_integer_value() {
        let record = make_record(&[
            "12345678901234",           // id
            "20230101",                 // effectiveTime
            "1",                        // active
            "900000000000207008",       // moduleId
            "322236009",                // sourceId
            "#500",                     // value (integer)
            "1",                        // relationshipGroup
            "1142135004",               // typeId
            "900000000000011006",       // characteristicTypeId (inferred)
            "900000000000451002",       // modifierId
        ]);

        let rel = Rf2ConcreteRelationship::from_record(&record).unwrap();

        assert_eq!(rel.id, 12345678901234);
        assert!(rel.active);
        assert_eq!(rel.source_id, 322236009);
        assert!(rel.value.is_integer());
        assert_eq!(rel.value.as_integer(), Some(500));
        assert!(rel.is_inferred());
    }

    #[test]
    fn test_parse_decimal_value() {
        let record = make_record(&[
            "12345678901234",
            "20230101",
            "1",
            "900000000000207008",
            "322236009",
            "#0.5",                     // decimal value
            "1",
            "1142135004",
            "900000000000011006",
            "900000000000451002",
        ]);

        let rel = Rf2ConcreteRelationship::from_record(&record).unwrap();
        assert!(rel.value.is_decimal());
        assert_eq!(rel.value.as_decimal(), Some(0.5));
    }

    #[test]
    fn test_parse_string_value() {
        let record = make_record(&[
            "12345678901234",
            "20230101",
            "1",
            "900000000000207008",
            "322236009",
            "\"tablet\"",              // string value
            "1",
            "1142135004",
            "900000000000011006",
            "900000000000451002",
        ]);

        let rel = Rf2ConcreteRelationship::from_record(&record).unwrap();
        assert!(rel.value.is_string());
        assert_eq!(rel.value.as_string(), Some("tablet"));
    }

    #[test]
    fn test_filter_inferred_only() {
        let record = make_record(&[
            "12345678901234",
            "20230101",
            "1",
            "900000000000207008",
            "322236009",
            "#500",
            "1",
            "1142135004",
            "900000000000010007",       // stated (not inferred)
            "900000000000451002",
        ]);

        let rel = Rf2ConcreteRelationship::from_record(&record).unwrap();

        let config = ConcreteRelationshipConfig::inferred_only();
        assert!(!rel.passes_concrete_filter(&config));

        let config = ConcreteRelationshipConfig::stated_only();
        assert!(rel.passes_concrete_filter(&config));
    }

    #[test]
    fn test_filter_by_type() {
        let record = make_record(&[
            "12345678901234",
            "20230101",
            "1",
            "900000000000207008",
            "322236009",
            "#500",
            "1",
            "1142135004",
            "900000000000011006",
            "900000000000451002",
        ]);

        let rel = Rf2ConcreteRelationship::from_record(&record).unwrap();

        let mut config = ConcreteRelationshipConfig::default();
        config.type_ids = vec![1142135004];
        assert!(rel.passes_concrete_filter(&config));

        config.type_ids = vec![999999999];
        assert!(!rel.passes_concrete_filter(&config));
    }
}
