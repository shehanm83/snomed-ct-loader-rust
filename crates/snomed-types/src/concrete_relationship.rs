//! Concrete relationship types for SNOMED CT RF2 files.
//!
//! Concrete relationships (also known as concrete domain relationships) allow
//! SNOMED CT concepts to have relationships with literal values (strings, integers,
//! or decimals) rather than other concepts.
//!
//! These are stored in files matching `sct2_RelationshipConcreteValues_*.txt`.
//!
//! # Example
//!
//! ```
//! use snomed_types::{Rf2ConcreteRelationship, ConcreteValue, SctId};
//!
//! // A medication with strength 500mg
//! let rel = Rf2ConcreteRelationship {
//!     id: 12345678901234,
//!     effective_time: 20230101,
//!     active: true,
//!     module_id: 900000000000207008,
//!     source_id: 322236009,  // Paracetamol 500mg tablet
//!     value: ConcreteValue::Integer(500),
//!     relationship_group: 1,
//!     type_id: 1142135004,  // Has presentation strength numerator value
//!     characteristic_type_id: 900000000000011006,
//!     modifier_id: 900000000000451002,
//! };
//!
//! assert!(rel.active);
//! assert_eq!(rel.value.as_integer(), Some(500));
//! ```

use crate::SctId;
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents a concrete value in SNOMED CT.
///
/// SNOMED CT supports three types of concrete values:
/// - String - text values (enclosed in quotes in RF2)
/// - Integer - whole number values (prefixed with # in RF2)
/// - Decimal - floating point values (prefixed with # in RF2)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ConcreteValue {
    /// A string value (e.g., "tablet").
    String(String),
    /// An integer value (e.g., 500 for 500mg).
    Integer(i64),
    /// A decimal value (e.g., 0.5 for 0.5ml).
    Decimal(f64),
}

impl ConcreteValue {
    /// Returns the value as a string if it is a String variant.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ConcreteValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the value as an integer if it is an Integer variant.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ConcreteValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Returns the value as a decimal if it is a Decimal variant.
    pub fn as_decimal(&self) -> Option<f64> {
        match self {
            ConcreteValue::Decimal(d) => Some(*d),
            _ => None,
        }
    }

    /// Returns true if this is a String value.
    pub fn is_string(&self) -> bool {
        matches!(self, ConcreteValue::String(_))
    }

    /// Returns true if this is an Integer value.
    pub fn is_integer(&self) -> bool {
        matches!(self, ConcreteValue::Integer(_))
    }

    /// Returns true if this is a Decimal value.
    pub fn is_decimal(&self) -> bool {
        matches!(self, ConcreteValue::Decimal(_))
    }

    /// Returns the type name as a string.
    pub fn type_name(&self) -> &'static str {
        match self {
            ConcreteValue::String(_) => "string",
            ConcreteValue::Integer(_) => "integer",
            ConcreteValue::Decimal(_) => "decimal",
        }
    }

    /// Parse a concrete value from RF2 format.
    ///
    /// RF2 format:
    /// - Strings are enclosed in quotes: `"value"`
    /// - Integers are prefixed with #: `#500`
    /// - Decimals are prefixed with # and contain a decimal point: `#0.5`
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        // String values are enclosed in quotes
        if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            return Some(ConcreteValue::String(s[1..s.len() - 1].to_string()));
        }

        // Numeric values are prefixed with #
        if let Some(num_str) = s.strip_prefix('#') {
            // Check if it's a decimal (contains a decimal point)
            if num_str.contains('.') {
                if let Ok(d) = num_str.parse::<f64>() {
                    return Some(ConcreteValue::Decimal(d));
                }
            } else {
                // Try parsing as integer
                if let Ok(i) = num_str.parse::<i64>() {
                    return Some(ConcreteValue::Integer(i));
                }
            }
        }

        None
    }
}

impl fmt::Display for ConcreteValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConcreteValue::String(s) => write!(f, "\"{}\"", s),
            ConcreteValue::Integer(i) => write!(f, "#{}", i),
            ConcreteValue::Decimal(d) => write!(f, "#{}", d),
        }
    }
}

/// A concrete relationship from RF2 Relationship Concrete Values files.
///
/// Unlike regular relationships that connect two concepts, concrete relationships
/// connect a concept to a literal value (string, integer, or decimal).
///
/// Common uses include:
/// - Medication strengths (e.g., 500mg)
/// - Counts and quantities
/// - Units and measurements
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rf2ConcreteRelationship {
    /// Unique identifier for this relationship.
    pub id: SctId,

    /// Effective time in YYYYMMDD format.
    pub effective_time: u32,

    /// Whether this relationship is currently active.
    pub active: bool,

    /// Module containing this relationship.
    pub module_id: SctId,

    /// Source concept ID (the concept being described).
    pub source_id: SctId,

    /// The concrete value (string, integer, or decimal).
    pub value: ConcreteValue,

    /// Relationship group for grouping related relationships.
    pub relationship_group: u16,

    /// Type of relationship (e.g., "Has strength numerator value").
    pub type_id: SctId,

    /// Characteristic type (inferred, stated, etc.).
    pub characteristic_type_id: SctId,

    /// Modifier type (existential, universal).
    pub modifier_id: SctId,
}

impl Rf2ConcreteRelationship {
    /// SNOMED CT ID for inferred characteristic type.
    pub const INFERRED_CHARACTERISTIC_TYPE: SctId = 900000000000011006;

    /// SNOMED CT ID for stated characteristic type.
    pub const STATED_CHARACTERISTIC_TYPE: SctId = 900000000000010007;

    /// Returns true if this is an inferred relationship.
    pub fn is_inferred(&self) -> bool {
        self.characteristic_type_id == Self::INFERRED_CHARACTERISTIC_TYPE
    }

    /// Returns true if this is a stated relationship.
    pub fn is_stated(&self) -> bool {
        self.characteristic_type_id == Self::STATED_CHARACTERISTIC_TYPE
    }

    /// Returns the value type name.
    pub fn value_type(&self) -> &'static str {
        self.value.type_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concrete_value_parse_string() {
        let value = ConcreteValue::parse("\"tablet\"").unwrap();
        assert!(value.is_string());
        assert_eq!(value.as_string(), Some("tablet"));
    }

    #[test]
    fn test_concrete_value_parse_integer() {
        let value = ConcreteValue::parse("#500").unwrap();
        assert!(value.is_integer());
        assert_eq!(value.as_integer(), Some(500));
    }

    #[test]
    fn test_concrete_value_parse_negative_integer() {
        let value = ConcreteValue::parse("#-100").unwrap();
        assert!(value.is_integer());
        assert_eq!(value.as_integer(), Some(-100));
    }

    #[test]
    fn test_concrete_value_parse_decimal() {
        let value = ConcreteValue::parse("#0.5").unwrap();
        assert!(value.is_decimal());
        assert_eq!(value.as_decimal(), Some(0.5));
    }

    #[test]
    fn test_concrete_value_parse_negative_decimal() {
        let value = ConcreteValue::parse("#-12.34").unwrap();
        assert!(value.is_decimal());
        assert_eq!(value.as_decimal(), Some(-12.34));
    }

    #[test]
    fn test_concrete_value_display() {
        assert_eq!(ConcreteValue::String("test".to_string()).to_string(), "\"test\"");
        assert_eq!(ConcreteValue::Integer(500).to_string(), "#500");
        assert_eq!(ConcreteValue::Decimal(0.5).to_string(), "#0.5");
    }

    #[test]
    fn test_concrete_relationship_characteristic_types() {
        let rel = Rf2ConcreteRelationship {
            id: 12345678901234,
            effective_time: 20230101,
            active: true,
            module_id: 900000000000207008,
            source_id: 322236009,
            value: ConcreteValue::Integer(500),
            relationship_group: 1,
            type_id: 1142135004,
            characteristic_type_id: Rf2ConcreteRelationship::INFERRED_CHARACTERISTIC_TYPE,
            modifier_id: 900000000000451002,
        };

        assert!(rel.is_inferred());
        assert!(!rel.is_stated());
        assert_eq!(rel.value_type(), "integer");
    }
}
