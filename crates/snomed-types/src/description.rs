//! SNOMED CT Description type.
//!
//! This module provides the `Rf2Description` struct representing a description
//! from an RF2 Description file.

use crate::{CaseSignificance, DescriptionType, SctId};

/// A SNOMED CT description from the RF2 Description file.
///
/// Represents a row from `sct2_Description_*.txt` files in an RF2 release.
///
/// # Examples
///
/// ```
/// use snomed_types::{Rf2Description, DescriptionType};
///
/// let description = Rf2Description {
///     id: 754786011,
///     effective_time: 20020131,
///     active: true,
///     module_id: 900000000000207008,
///     concept_id: 73211009,
///     language_code: "en".to_string(),
///     type_id: 900000000000003001, // FSN
///     term: "Diabetes mellitus (disorder)".to_string(),
///     case_significance_id: 900000000000448009,
/// };
///
/// assert!(description.is_fsn());
/// assert_eq!(description.description_type(), Some(DescriptionType::Fsn));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rf2Description {
    /// Unique identifier for this description (SCTID).
    pub id: SctId,
    /// Effective date in YYYYMMDD format.
    pub effective_time: u32,
    /// Whether this description is active.
    pub active: bool,
    /// The module containing this description.
    pub module_id: SctId,
    /// The concept this description belongs to.
    pub concept_id: SctId,
    /// ISO language code (e.g., "en").
    pub language_code: String,
    /// Type of description (FSN, Synonym, etc.).
    pub type_id: SctId,
    /// The description text/term.
    pub term: String,
    /// Case significance rules for this term.
    pub case_significance_id: SctId,
}

impl Rf2Description {
    /// Returns the description type enum value.
    ///
    /// Returns `None` if the type ID is not recognized.
    pub fn description_type(&self) -> Option<DescriptionType> {
        DescriptionType::from_id(self.type_id)
    }

    /// Returns true if this is a Fully Specified Name.
    ///
    /// FSN descriptions are unambiguous and include a semantic tag in parentheses.
    pub fn is_fsn(&self) -> bool {
        self.type_id == DescriptionType::FSN_ID
    }

    /// Returns true if this is a Synonym.
    ///
    /// Synonyms are acceptable alternative terms for a concept.
    pub fn is_synonym(&self) -> bool {
        self.type_id == DescriptionType::SYNONYM_ID
    }

    /// Returns true if this is a Definition.
    ///
    /// Definitions are textual explanations of a concept.
    pub fn is_definition(&self) -> bool {
        self.type_id == DescriptionType::DEFINITION_ID
    }

    /// Returns the case significance enum value.
    ///
    /// Returns `None` if the case significance ID is not recognized.
    pub fn case_significance(&self) -> Option<CaseSignificance> {
        CaseSignificance::from_id(self.case_significance_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_description(type_id: SctId) -> Rf2Description {
        Rf2Description {
            id: 754786011,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            concept_id: 73211009,
            language_code: "en".to_string(),
            type_id,
            term: "Diabetes mellitus (disorder)".to_string(),
            case_significance_id: 900000000000448009,
        }
    }

    #[test]
    fn test_rf2_description_fsn() {
        let desc = make_description(DescriptionType::FSN_ID);
        assert!(desc.is_fsn());
        assert!(!desc.is_synonym());
        assert!(!desc.is_definition());
        assert_eq!(desc.description_type(), Some(DescriptionType::Fsn));
    }

    #[test]
    fn test_rf2_description_synonym() {
        let desc = make_description(DescriptionType::SYNONYM_ID);
        assert!(!desc.is_fsn());
        assert!(desc.is_synonym());
        assert!(!desc.is_definition());
        assert_eq!(desc.description_type(), Some(DescriptionType::Synonym));
    }

    #[test]
    fn test_rf2_description_case_significance() {
        let desc = make_description(DescriptionType::FSN_ID);
        assert_eq!(
            desc.case_significance(),
            Some(CaseSignificance::CaseInsensitive)
        );
    }
}
