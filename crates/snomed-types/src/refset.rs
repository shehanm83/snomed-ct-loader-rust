//! SNOMED CT Reference Set types.
//!
//! Reference sets (refsets) are used in SNOMED CT to group components together
//! for various purposes. The most common types are:
//!
//! - **Simple refsets**: Basic membership (concept belongs to refset)
//! - **Language refsets**: Preferred/acceptable terms per dialect
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
}
