//! SNOMED CT enumeration types.
//!
//! This module provides enum representations for various SNOMED CT coded values
//! such as definition status, description type, case significance, etc.

use crate::SctId;

/// Definition status for a SNOMED CT concept.
///
/// Indicates whether a concept is primitively defined (necessary conditions only)
/// or fully defined (necessary and sufficient conditions).
///
/// # Examples
///
/// ```
/// use snomed_types::DefinitionStatus;
///
/// let status = DefinitionStatus::from_id(900000000000074008);
/// assert_eq!(status, Some(DefinitionStatus::Primitive));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DefinitionStatus {
    /// Concept is primitively defined (necessary conditions only).
    Primitive,
    /// Concept is fully defined (necessary and sufficient conditions).
    FullyDefined,
}

impl DefinitionStatus {
    /// SCTID for primitive definition status.
    pub const PRIMITIVE_ID: SctId = 900000000000074008;
    /// SCTID for fully defined definition status.
    pub const FULLY_DEFINED_ID: SctId = 900000000000073002;

    /// Creates a DefinitionStatus from its SCTID.
    ///
    /// Returns `None` if the ID doesn't match a known definition status.
    pub fn from_id(id: SctId) -> Option<Self> {
        match id {
            Self::PRIMITIVE_ID => Some(Self::Primitive),
            Self::FULLY_DEFINED_ID => Some(Self::FullyDefined),
            _ => None,
        }
    }

    /// Returns the SCTID for this definition status.
    pub fn to_id(self) -> SctId {
        match self {
            Self::Primitive => Self::PRIMITIVE_ID,
            Self::FullyDefined => Self::FULLY_DEFINED_ID,
        }
    }
}

/// Description type for SNOMED CT descriptions.
///
/// Indicates whether a description is a Fully Specified Name (FSN),
/// a Synonym, or a Definition.
///
/// # Examples
///
/// ```
/// use snomed_types::DescriptionType;
///
/// let desc_type = DescriptionType::from_id(900000000000003001);
/// assert_eq!(desc_type, Some(DescriptionType::Fsn));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DescriptionType {
    /// Fully Specified Name - unambiguous description with semantic tag.
    Fsn,
    /// Synonym - additional acceptable term for the concept.
    Synonym,
    /// Definition - textual definition (from text definition refset).
    Definition,
}

impl DescriptionType {
    /// SCTID for Fully Specified Name type.
    pub const FSN_ID: SctId = 900000000000003001;
    /// SCTID for Synonym type.
    pub const SYNONYM_ID: SctId = 900000000000013009;
    /// SCTID for Definition type.
    pub const DEFINITION_ID: SctId = 900000000000550004;

    /// Creates a DescriptionType from its SCTID.
    ///
    /// Returns `None` if the ID doesn't match a known description type.
    pub fn from_id(id: SctId) -> Option<Self> {
        match id {
            Self::FSN_ID => Some(Self::Fsn),
            Self::SYNONYM_ID => Some(Self::Synonym),
            Self::DEFINITION_ID => Some(Self::Definition),
            _ => None,
        }
    }

    /// Returns the SCTID for this description type.
    pub fn to_id(self) -> SctId {
        match self {
            Self::Fsn => Self::FSN_ID,
            Self::Synonym => Self::SYNONYM_ID,
            Self::Definition => Self::DEFINITION_ID,
        }
    }
}

/// Case significance for SNOMED CT descriptions.
///
/// Indicates how case sensitivity should be applied to a description term.
///
/// # Examples
///
/// ```
/// use snomed_types::CaseSignificance;
///
/// let case_sig = CaseSignificance::from_id(900000000000448009);
/// assert_eq!(case_sig, Some(CaseSignificance::CaseInsensitive));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CaseSignificance {
    /// Entire term is case insensitive.
    CaseInsensitive,
    /// Entire term is case sensitive.
    EntireTermCaseSensitive,
    /// Only initial character is case sensitive.
    InitialCharacterCaseSensitive,
}

impl CaseSignificance {
    /// SCTID for case insensitive.
    pub const CASE_INSENSITIVE_ID: SctId = 900000000000448009;
    /// SCTID for entire term case sensitive.
    pub const ENTIRE_TERM_CASE_SENSITIVE_ID: SctId = 900000000000017005;
    /// SCTID for initial character case sensitive.
    pub const INITIAL_CHAR_CASE_SENSITIVE_ID: SctId = 900000000000020002;

    /// Creates a CaseSignificance from its SCTID.
    ///
    /// Returns `None` if the ID doesn't match a known case significance.
    pub fn from_id(id: SctId) -> Option<Self> {
        match id {
            Self::CASE_INSENSITIVE_ID => Some(Self::CaseInsensitive),
            Self::ENTIRE_TERM_CASE_SENSITIVE_ID => Some(Self::EntireTermCaseSensitive),
            Self::INITIAL_CHAR_CASE_SENSITIVE_ID => Some(Self::InitialCharacterCaseSensitive),
            _ => None,
        }
    }

    /// Returns the SCTID for this case significance.
    pub fn to_id(self) -> SctId {
        match self {
            Self::CaseInsensitive => Self::CASE_INSENSITIVE_ID,
            Self::EntireTermCaseSensitive => Self::ENTIRE_TERM_CASE_SENSITIVE_ID,
            Self::InitialCharacterCaseSensitive => Self::INITIAL_CHAR_CASE_SENSITIVE_ID,
        }
    }
}

/// Characteristic type for SNOMED CT relationships.
///
/// Indicates whether a relationship is stated (as authored) or inferred (computed).
///
/// # Examples
///
/// ```
/// use snomed_types::CharacteristicType;
///
/// let char_type = CharacteristicType::from_id(900000000000011006);
/// assert_eq!(char_type, Some(CharacteristicType::Inferred));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CharacteristicType {
    /// Stated relationship (as authored).
    Stated,
    /// Inferred relationship (computed by classifier).
    Inferred,
    /// Additional relationship.
    Additional,
}

impl CharacteristicType {
    /// SCTID for stated relationship.
    pub const STATED_ID: SctId = 900000000000010007;
    /// SCTID for inferred relationship.
    pub const INFERRED_ID: SctId = 900000000000011006;
    /// SCTID for additional relationship.
    pub const ADDITIONAL_ID: SctId = 900000000000227009;

    /// Creates a CharacteristicType from its SCTID.
    ///
    /// Returns `None` if the ID doesn't match a known characteristic type.
    pub fn from_id(id: SctId) -> Option<Self> {
        match id {
            Self::STATED_ID => Some(Self::Stated),
            Self::INFERRED_ID => Some(Self::Inferred),
            Self::ADDITIONAL_ID => Some(Self::Additional),
            _ => None,
        }
    }

    /// Returns the SCTID for this characteristic type.
    pub fn to_id(self) -> SctId {
        match self {
            Self::Stated => Self::STATED_ID,
            Self::Inferred => Self::INFERRED_ID,
            Self::Additional => Self::ADDITIONAL_ID,
        }
    }
}

/// Relationship modifier type.
///
/// Indicates whether a relationship uses existential (some) or universal (all)
/// quantification.
///
/// # Examples
///
/// ```
/// use snomed_types::ModifierType;
///
/// let modifier = ModifierType::from_id(900000000000451002);
/// assert_eq!(modifier, Some(ModifierType::Existential));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ModifierType {
    /// Existential modifier (some).
    Existential,
    /// Universal modifier (all).
    Universal,
}

impl ModifierType {
    /// SCTID for existential (some) modifier.
    pub const EXISTENTIAL_ID: SctId = 900000000000451002;
    /// SCTID for universal (all) modifier.
    pub const UNIVERSAL_ID: SctId = 900000000000450001;

    /// Creates a ModifierType from its SCTID.
    ///
    /// Returns `None` if the ID doesn't match a known modifier type.
    pub fn from_id(id: SctId) -> Option<Self> {
        match id {
            Self::EXISTENTIAL_ID => Some(Self::Existential),
            Self::UNIVERSAL_ID => Some(Self::Universal),
            _ => None,
        }
    }

    /// Returns the SCTID for this modifier type.
    pub fn to_id(self) -> SctId {
        match self {
            Self::Existential => Self::EXISTENTIAL_ID,
            Self::Universal => Self::UNIVERSAL_ID,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition_status_conversion() {
        assert_eq!(
            DefinitionStatus::from_id(900000000000074008),
            Some(DefinitionStatus::Primitive)
        );
        assert_eq!(
            DefinitionStatus::from_id(900000000000073002),
            Some(DefinitionStatus::FullyDefined)
        );
        assert_eq!(DefinitionStatus::from_id(12345), None);
        assert_eq!(DefinitionStatus::Primitive.to_id(), 900000000000074008);
    }

    #[test]
    fn test_description_type_conversion() {
        assert_eq!(
            DescriptionType::from_id(900000000000003001),
            Some(DescriptionType::Fsn)
        );
        assert_eq!(
            DescriptionType::from_id(900000000000013009),
            Some(DescriptionType::Synonym)
        );
        assert_eq!(DescriptionType::Fsn.to_id(), 900000000000003001);
    }

    #[test]
    fn test_characteristic_type_conversion() {
        assert_eq!(
            CharacteristicType::from_id(900000000000010007),
            Some(CharacteristicType::Stated)
        );
        assert_eq!(
            CharacteristicType::from_id(900000000000011006),
            Some(CharacteristicType::Inferred)
        );
    }

    #[test]
    fn test_case_significance_conversion() {
        assert_eq!(
            CaseSignificance::from_id(900000000000448009),
            Some(CaseSignificance::CaseInsensitive)
        );
        assert_eq!(
            CaseSignificance::from_id(900000000000017005),
            Some(CaseSignificance::EntireTermCaseSensitive)
        );
    }

    #[test]
    fn test_modifier_type_conversion() {
        assert_eq!(
            ModifierType::from_id(900000000000451002),
            Some(ModifierType::Existential)
        );
        assert_eq!(
            ModifierType::from_id(900000000000450001),
            Some(ModifierType::Universal)
        );
    }
}
