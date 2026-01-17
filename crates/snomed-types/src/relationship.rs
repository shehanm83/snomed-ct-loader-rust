//! SNOMED CT Relationship type.
//!
//! This module provides the `Rf2Relationship` struct representing a relationship
//! from an RF2 Relationship file.

use crate::{CharacteristicType, ModifierType, SctId};

/// A SNOMED CT relationship from the RF2 Relationship file.
///
/// Represents a row from `sct2_Relationship_*.txt` files in an RF2 release.
///
/// # Examples
///
/// ```
/// use snomed_types::{Rf2Relationship, CharacteristicType};
///
/// let relationship = Rf2Relationship {
///     id: 100000028,
///     effective_time: 20020131,
///     active: true,
///     module_id: 900000000000207008,
///     source_id: 73211009,        // Diabetes mellitus
///     destination_id: 362969004,  // Disorder of endocrine system
///     relationship_group: 0,
///     type_id: 116680003,         // IS_A
///     characteristic_type_id: 900000000000011006, // Inferred
///     modifier_id: 900000000000451002, // Existential
/// };
///
/// assert!(relationship.is_is_a());
/// assert!(relationship.is_inferred());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rf2Relationship {
    /// Unique identifier for this relationship (SCTID).
    pub id: SctId,
    /// Effective date in YYYYMMDD format.
    pub effective_time: u32,
    /// Whether this relationship is active.
    pub active: bool,
    /// The module containing this relationship.
    pub module_id: SctId,
    /// Source concept (subject).
    pub source_id: SctId,
    /// Destination concept (object/value).
    pub destination_id: SctId,
    /// Role group number (0 = ungrouped).
    pub relationship_group: u16,
    /// Relationship type (e.g., IS_A, Finding site).
    pub type_id: SctId,
    /// Whether this is stated or inferred.
    pub characteristic_type_id: SctId,
    /// Modifier (existential or universal).
    pub modifier_id: SctId,
}

impl Rf2Relationship {
    /// SCTID for the IS_A relationship type.
    pub const IS_A_TYPE_ID: SctId = 116680003;

    /// Returns true if this is an IS_A (subtype) relationship.
    ///
    /// IS_A relationships define the taxonomy/hierarchy of SNOMED CT.
    pub fn is_is_a(&self) -> bool {
        self.type_id == Self::IS_A_TYPE_ID
    }

    /// Returns the characteristic type enum value.
    ///
    /// Returns `None` if the characteristic type ID is not recognized.
    pub fn characteristic_type(&self) -> Option<CharacteristicType> {
        CharacteristicType::from_id(self.characteristic_type_id)
    }

    /// Returns true if this is a stated relationship.
    ///
    /// Stated relationships are as authored by SNOMED CT editors.
    pub fn is_stated(&self) -> bool {
        self.characteristic_type_id == CharacteristicType::STATED_ID
    }

    /// Returns true if this is an inferred relationship.
    ///
    /// Inferred relationships are computed by the classifier.
    pub fn is_inferred(&self) -> bool {
        self.characteristic_type_id == CharacteristicType::INFERRED_ID
    }

    /// Returns the modifier type enum value.
    ///
    /// Returns `None` if the modifier ID is not recognized.
    pub fn modifier_type(&self) -> Option<ModifierType> {
        ModifierType::from_id(self.modifier_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_relationship(type_id: SctId, characteristic_type_id: SctId) -> Rf2Relationship {
        Rf2Relationship {
            id: 100000028,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            source_id: 73211009,
            destination_id: 362969004,
            relationship_group: 0,
            type_id,
            characteristic_type_id,
            modifier_id: ModifierType::EXISTENTIAL_ID,
        }
    }

    #[test]
    fn test_rf2_relationship_is_a() {
        let rel = make_relationship(Rf2Relationship::IS_A_TYPE_ID, CharacteristicType::INFERRED_ID);
        assert!(rel.is_is_a());
        assert!(rel.is_inferred());
        assert!(!rel.is_stated());
    }

    #[test]
    fn test_rf2_relationship_stated() {
        let rel = make_relationship(Rf2Relationship::IS_A_TYPE_ID, CharacteristicType::STATED_ID);
        assert!(rel.is_stated());
        assert!(!rel.is_inferred());
        assert_eq!(rel.characteristic_type(), Some(CharacteristicType::Stated));
    }

    #[test]
    fn test_rf2_relationship_modifier() {
        let rel = make_relationship(Rf2Relationship::IS_A_TYPE_ID, CharacteristicType::INFERRED_ID);
        assert_eq!(rel.modifier_type(), Some(ModifierType::Existential));
    }

    #[test]
    fn test_rf2_relationship_non_is_a() {
        // Finding site relationship
        let rel = make_relationship(363698007, CharacteristicType::INFERRED_ID);
        assert!(!rel.is_is_a());
    }
}
