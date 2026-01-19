//! # snomed-types
//!
//! Type definitions for SNOMED CT clinical terminology.
//!
//! This crate provides Rust type definitions for working with SNOMED CT
//! Release Format 2 (RF2) data structures, including concepts, descriptions,
//! and relationships.
//!
//! ## Features
//!
//! - `serde` (default): Enables serialization/deserialization support via serde.
//!   Disable this feature for zero-dependency usage.
//!
//! ## Usage
//!
//! ```rust
//! use snomed_types::{Rf2Concept, Rf2Description, Rf2Relationship, SctId};
//! use snomed_types::{DefinitionStatus, DescriptionType, CharacteristicType};
//! use snomed_types::well_known;
//!
//! // Create a concept
//! let concept = Rf2Concept {
//!     id: 73211009,
//!     effective_time: 20020131,
//!     active: true,
//!     module_id: well_known::SNOMED_CT_CORE_MODULE,
//!     definition_status_id: DefinitionStatus::PRIMITIVE_ID,
//! };
//!
//! assert!(concept.is_primitive());
//!
//! // Use well-known constants
//! let is_a_type: SctId = well_known::IS_A;
//! let clinical_finding: SctId = well_known::CLINICAL_FINDING;
//! ```
//!
//! ## Without Serde
//!
//! To use this crate without serde (zero dependencies):
//!
//! ```toml
//! [dependencies]
//! snomed-types = { version = "0.1", default-features = false }
//! ```

#![warn(missing_docs)]

mod concept;
mod description;
mod enums;
pub mod mrcm;
pub mod refset;
mod relationship;
mod sctid;
pub mod well_known;

// Re-export all public types at crate root
pub use concept::Rf2Concept;
pub use description::Rf2Description;
pub use enums::{
    CaseSignificance, CharacteristicType, DefinitionStatus, DescriptionType, ModifierType,
};
pub use mrcm::{
    Cardinality, CardinalityParseError, MrcmAttributeDomain, MrcmAttributeRange, MrcmDomain,
};
pub use refset::{Rf2LanguageRefsetMember, Rf2SimpleRefsetMember};
pub use relationship::Rf2Relationship;
pub use sctid::SctId;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_types_are_exported() {
        // Verify all types are accessible from crate root
        let _id: SctId = 73211009;
        let _status = DefinitionStatus::Primitive;
        let _desc_type = DescriptionType::Fsn;
        let _case_sig = CaseSignificance::CaseInsensitive;
        let _char_type = CharacteristicType::Inferred;
        let _modifier = ModifierType::Existential;
    }

    #[test]
    fn test_well_known_accessible() {
        assert_eq!(well_known::IS_A, 116680003);
        assert_eq!(well_known::CLINICAL_FINDING, 404684003);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_roundtrip() {
        let concept = Rf2Concept {
            id: 404684003,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            definition_status_id: 900000000000074008,
        };

        let json = serde_json::to_string(&concept).unwrap();
        let parsed: Rf2Concept = serde_json::from_str(&json).unwrap();
        assert_eq!(concept, parsed);
    }
}
