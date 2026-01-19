//! SNOMED CT Reference Set types.
//!
//! Reference sets (refsets) are used in SNOMED CT to group components together
//! for various purposes. The most common types are:
//!
//! - **Simple refsets**: Basic membership (concept belongs to refset)
//! - **Language refsets**: Preferred/acceptable terms per dialect
//! - **Association refsets**: Links between components (e.g., replaced by, same as)
//! - **Attribute value refsets**: Additional metadata for components
//!
//! # Example
//!
//! ```
//! use snomed_types::Rf2SimpleRefsetMember;
//!
//! let member = Rf2SimpleRefsetMember {
//!     id: 12345678901,
//!     effective_time: 20200101,
//!     active: true,
//!     module_id: 900000000000207008,
//!     refset_id: 723264001,  // Lateralizable body structure reference set
//!     referenced_component_id: 12345678,  // A concept ID
//! };
//!
//! assert!(member.active);
//! ```

use crate::SctId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A simple reference set member from RF2 SimpleRefset files.
///
/// Simple refsets define membership without additional attributes.
/// The refset_id identifies which refset this member belongs to,
/// and referenced_component_id is the component (usually concept) that is a member.
///
/// # RF2 Columns
///
/// | Column | Type | Description |
/// |--------|------|-------------|
/// | id | SCTID | Unique identifier for this member |
/// | effectiveTime | Integer | Date in YYYYMMDD format |
/// | active | Boolean | Whether this membership is active |
/// | moduleId | SCTID | Module containing this member |
/// | refsetId | SCTID | The reference set this member belongs to |
/// | referencedComponentId | SCTID | The component that is a member |
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rf2SimpleRefsetMember {
    /// Unique identifier for this reference set member.
    pub id: SctId,
    /// Effective time as YYYYMMDD integer.
    pub effective_time: u32,
    /// Whether this membership is currently active.
    pub active: bool,
    /// Module this member belongs to.
    pub module_id: SctId,
    /// The reference set this member belongs to.
    pub refset_id: SctId,
    /// The component (usually concept) that is a member of the refset.
    pub referenced_component_id: SctId,
}

/// A language reference set member from RF2 Language refset files.
///
/// Language refsets indicate whether a description is preferred or acceptable
/// in a particular language/dialect context.
///
/// # RF2 Columns
///
/// | Column | Type | Description |
/// |--------|------|-------------|
/// | id | SCTID | Unique identifier for this member |
/// | effectiveTime | Integer | Date in YYYYMMDD format |
/// | active | Boolean | Whether this membership is active |
/// | moduleId | SCTID | Module containing this member |
/// | refsetId | SCTID | The language reference set (e.g., US English, GB English) |
/// | referencedComponentId | SCTID | The description ID |
/// | acceptabilityId | SCTID | Preferred (900000000000548007) or Acceptable (900000000000549004) |
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rf2LanguageRefsetMember {
    /// Unique identifier for this reference set member.
    pub id: SctId,
    /// Effective time as YYYYMMDD integer.
    pub effective_time: u32,
    /// Whether this membership is currently active.
    pub active: bool,
    /// Module this member belongs to.
    pub module_id: SctId,
    /// The language reference set (dialect) this member belongs to.
    pub refset_id: SctId,
    /// The description ID that is a member.
    pub referenced_component_id: SctId,
    /// Acceptability: Preferred (900000000000548007) or Acceptable (900000000000549004).
    pub acceptability_id: SctId,
}

impl Rf2LanguageRefsetMember {
    /// SCTID for "Preferred" acceptability.
    pub const PREFERRED_ID: SctId = 900000000000548007;
    /// SCTID for "Acceptable" acceptability.
    pub const ACCEPTABLE_ID: SctId = 900000000000549004;

    /// Returns true if this description is preferred in this dialect.
    pub fn is_preferred(&self) -> bool {
        self.acceptability_id == Self::PREFERRED_ID
    }

    /// Returns true if this description is acceptable (but not preferred) in this dialect.
    pub fn is_acceptable(&self) -> bool {
        self.acceptability_id == Self::ACCEPTABLE_ID
    }
}

/// An association reference set member from RF2 Association refset files.
///
/// Association refsets define relationships between components that are not
/// part of the core terminology model. Common uses include:
///
/// - **Historical associations**: Links inactive concepts to their replacements
/// - **Cross-map associations**: Links to external code systems
/// - **Similarity associations**: SAME AS, POSSIBLY EQUIVALENT TO, etc.
///
/// # RF2 Columns
///
/// | Column | Type | Description |
/// |--------|------|-------------|
/// | id | SCTID | Unique identifier for this member |
/// | effectiveTime | Integer | Date in YYYYMMDD format |
/// | active | Boolean | Whether this association is active |
/// | moduleId | SCTID | Module containing this member |
/// | refsetId | SCTID | The association reference set |
/// | referencedComponentId | SCTID | The source component |
/// | targetComponentId | SCTID | The target component |
///
/// # Example
///
/// ```
/// use snomed_types::Rf2AssociationRefsetMember;
///
/// let member = Rf2AssociationRefsetMember {
///     id: 12345678901,
///     effective_time: 20200101,
///     active: true,
///     module_id: 900000000000207008,
///     refset_id: 900000000000527005,  // SAME AS association
///     referenced_component_id: 12345678,  // Source concept
///     target_component_id: 87654321,  // Target concept
/// };
///
/// assert!(member.is_same_as_association());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rf2AssociationRefsetMember {
    /// Unique identifier for this reference set member.
    pub id: SctId,
    /// Effective time as YYYYMMDD integer.
    pub effective_time: u32,
    /// Whether this association is currently active.
    pub active: bool,
    /// Module this member belongs to.
    pub module_id: SctId,
    /// The association reference set this member belongs to.
    pub refset_id: SctId,
    /// The source component (usually the inactive or source concept).
    pub referenced_component_id: SctId,
    /// The target component (usually the replacement or related concept).
    pub target_component_id: SctId,
}

impl Rf2AssociationRefsetMember {
    // Historical association refset IDs
    /// REPLACED BY association reference set.
    pub const REPLACED_BY_REFSET: SctId = 900000000000526001;
    /// SAME AS association reference set.
    pub const SAME_AS_REFSET: SctId = 900000000000527005;
    /// WAS A association reference set.
    pub const WAS_A_REFSET: SctId = 900000000000528000;
    /// POSSIBLY EQUIVALENT TO association reference set.
    pub const POSSIBLY_EQUIVALENT_TO_REFSET: SctId = 900000000000523009;
    /// MOVED TO association reference set.
    pub const MOVED_TO_REFSET: SctId = 900000000000524003;
    /// MOVED FROM association reference set.
    pub const MOVED_FROM_REFSET: SctId = 900000000000525002;
    /// ALTERNATIVE association reference set.
    pub const ALTERNATIVE_REFSET: SctId = 900000000000530003;
    /// REFERS TO association reference set.
    pub const REFERS_TO_REFSET: SctId = 900000000000531004;

    /// Returns true if this is a REPLACED BY association.
    pub fn is_replaced_by_association(&self) -> bool {
        self.refset_id == Self::REPLACED_BY_REFSET
    }

    /// Returns true if this is a SAME AS association.
    pub fn is_same_as_association(&self) -> bool {
        self.refset_id == Self::SAME_AS_REFSET
    }

    /// Returns true if this is a WAS A association.
    pub fn is_was_a_association(&self) -> bool {
        self.refset_id == Self::WAS_A_REFSET
    }

    /// Returns true if this is a POSSIBLY EQUIVALENT TO association.
    pub fn is_possibly_equivalent_association(&self) -> bool {
        self.refset_id == Self::POSSIBLY_EQUIVALENT_TO_REFSET
    }

    /// Returns true if this is a MOVED TO association.
    pub fn is_moved_to_association(&self) -> bool {
        self.refset_id == Self::MOVED_TO_REFSET
    }

    /// Returns true if this is a historical association (any type).
    pub fn is_historical_association(&self) -> bool {
        matches!(
            self.refset_id,
            Self::REPLACED_BY_REFSET
                | Self::SAME_AS_REFSET
                | Self::WAS_A_REFSET
                | Self::POSSIBLY_EQUIVALENT_TO_REFSET
                | Self::MOVED_TO_REFSET
                | Self::MOVED_FROM_REFSET
                | Self::ALTERNATIVE_REFSET
        )
    }
}

/// Well-known reference set IDs.
pub mod well_known_refsets {
    use crate::SctId;

    // Language reference sets
    /// US English language reference set.
    pub const US_ENGLISH_LANG_REFSET: SctId = 900000000000509007;
    /// GB English language reference set.
    pub const GB_ENGLISH_LANG_REFSET: SctId = 900000000000508004;

    // Content reference sets
    /// ICD-10 simple map reference set.
    pub const ICD10_SIMPLE_MAP: SctId = 447562003;

    // Metadata reference sets
    /// Description format reference set.
    pub const DESCRIPTION_FORMAT_REFSET: SctId = 900000000000538005;

    // Association reference sets
    /// REPLACED BY association reference set.
    pub const REPLACED_BY_REFSET: SctId = 900000000000526001;
    /// SAME AS association reference set.
    pub const SAME_AS_REFSET: SctId = 900000000000527005;
    /// WAS A association reference set.
    pub const WAS_A_REFSET: SctId = 900000000000528000;
    /// POSSIBLY EQUIVALENT TO association reference set.
    pub const POSSIBLY_EQUIVALENT_TO_REFSET: SctId = 900000000000523009;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_refset_member() {
        let member = Rf2SimpleRefsetMember {
            id: 12345678901,
            effective_time: 20200101,
            active: true,
            module_id: 900000000000207008,
            refset_id: 723264001,
            referenced_component_id: 12345678,
        };

        assert!(member.active);
        assert_eq!(member.refset_id, 723264001);
    }

    #[test]
    fn test_language_refset_preferred() {
        let member = Rf2LanguageRefsetMember {
            id: 12345678901,
            effective_time: 20200101,
            active: true,
            module_id: 900000000000207008,
            refset_id: well_known_refsets::US_ENGLISH_LANG_REFSET,
            referenced_component_id: 12345678,
            acceptability_id: Rf2LanguageRefsetMember::PREFERRED_ID,
        };

        assert!(member.is_preferred());
        assert!(!member.is_acceptable());
    }

    #[test]
    fn test_language_refset_acceptable() {
        let member = Rf2LanguageRefsetMember {
            id: 12345678901,
            effective_time: 20200101,
            active: true,
            module_id: 900000000000207008,
            refset_id: well_known_refsets::GB_ENGLISH_LANG_REFSET,
            referenced_component_id: 12345678,
            acceptability_id: Rf2LanguageRefsetMember::ACCEPTABLE_ID,
        };

        assert!(!member.is_preferred());
        assert!(member.is_acceptable());
    }

    #[test]
    fn test_association_refset_replaced_by() {
        let member = Rf2AssociationRefsetMember {
            id: 12345678901,
            effective_time: 20200101,
            active: true,
            module_id: 900000000000207008,
            refset_id: Rf2AssociationRefsetMember::REPLACED_BY_REFSET,
            referenced_component_id: 12345678,
            target_component_id: 87654321,
        };

        assert!(member.is_replaced_by_association());
        assert!(member.is_historical_association());
        assert!(!member.is_same_as_association());
    }

    #[test]
    fn test_association_refset_same_as() {
        let member = Rf2AssociationRefsetMember {
            id: 12345678901,
            effective_time: 20200101,
            active: true,
            module_id: 900000000000207008,
            refset_id: Rf2AssociationRefsetMember::SAME_AS_REFSET,
            referenced_component_id: 12345678,
            target_component_id: 87654321,
        };

        assert!(member.is_same_as_association());
        assert!(member.is_historical_association());
        assert!(!member.is_replaced_by_association());
    }
}
