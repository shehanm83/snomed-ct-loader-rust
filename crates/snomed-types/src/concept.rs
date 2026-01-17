//! SNOMED CT Concept type.
//!
//! This module provides the `Rf2Concept` struct representing a concept
//! from an RF2 Concept file.

use crate::{DefinitionStatus, SctId};

/// A SNOMED CT concept from the RF2 Concept file.
///
/// Represents a row from `sct2_Concept_*.txt` files in an RF2 release.
///
/// # Examples
///
/// ```
/// use snomed_types::{Rf2Concept, DefinitionStatus};
///
/// let concept = Rf2Concept {
///     id: 73211009,
///     effective_time: 20020131,
///     active: true,
///     module_id: 900000000000207008,
///     definition_status_id: 900000000000074008, // Primitive
/// };
///
/// assert!(concept.is_primitive());
/// assert!(!concept.is_fully_defined());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rf2Concept {
    /// Unique identifier for this concept (SCTID).
    pub id: SctId,
    /// Effective date in YYYYMMDD format (stored as u32 for efficiency).
    pub effective_time: u32,
    /// Whether this concept is active (true) or inactive (false).
    pub active: bool,
    /// The module containing this concept.
    pub module_id: SctId,
    /// Whether this concept is primitive or fully defined.
    pub definition_status_id: SctId,
}

impl Rf2Concept {
    /// Returns the definition status enum value.
    ///
    /// Returns `None` if the definition status ID is not recognized.
    pub fn definition_status(&self) -> Option<DefinitionStatus> {
        DefinitionStatus::from_id(self.definition_status_id)
    }

    /// Returns true if this concept is primitively defined.
    ///
    /// A primitive concept has only necessary conditions.
    pub fn is_primitive(&self) -> bool {
        self.definition_status_id == DefinitionStatus::PRIMITIVE_ID
    }

    /// Returns true if this concept is fully defined.
    ///
    /// A fully defined concept has necessary and sufficient conditions.
    pub fn is_fully_defined(&self) -> bool {
        self.definition_status_id == DefinitionStatus::FULLY_DEFINED_ID
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rf2_concept_helpers() {
        let concept = Rf2Concept {
            id: 404684003,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            definition_status_id: 900000000000074008,
        };

        assert!(concept.is_primitive());
        assert!(!concept.is_fully_defined());
        assert_eq!(
            concept.definition_status(),
            Some(DefinitionStatus::Primitive)
        );
    }

    #[test]
    fn test_fully_defined_concept() {
        let concept = Rf2Concept {
            id: 73211009,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            definition_status_id: DefinitionStatus::FULLY_DEFINED_ID,
        };

        assert!(!concept.is_primitive());
        assert!(concept.is_fully_defined());
        assert_eq!(
            concept.definition_status(),
            Some(DefinitionStatus::FullyDefined)
        );
    }
}
