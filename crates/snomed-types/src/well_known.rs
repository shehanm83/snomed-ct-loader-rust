//! Well-known SNOMED CT concept IDs.
//!
//! This module provides constants for commonly used SNOMED CT concept identifiers,
//! including root concepts, top-level hierarchies, and common relationship types.
//!
//! # Examples
//!
//! ```
//! use snomed_types::well_known;
//!
//! // Check if a concept is the IS_A type
//! let type_id: u64 = 116680003;
//! assert_eq!(type_id, well_known::IS_A);
//!
//! // Reference top-level hierarchies
//! assert_eq!(well_known::CLINICAL_FINDING, 404684003);
//! assert_eq!(well_known::BODY_STRUCTURE, 123037004);
//! ```

use crate::SctId;

// =============================================================================
// Root Concepts
// =============================================================================

/// SNOMED CT root concept (138875005).
///
/// The single root of the entire SNOMED CT hierarchy.
pub const SNOMED_CT_ROOT: SctId = 138875005;

// =============================================================================
// Top-Level Hierarchies
// =============================================================================

/// Clinical finding (finding) - 404684003.
///
/// Represents disorders, diseases, symptoms, signs, and other clinical observations.
pub const CLINICAL_FINDING: SctId = 404684003;

/// Procedure (procedure) - 71388002.
///
/// Represents medical procedures, interventions, and activities.
pub const PROCEDURE: SctId = 71388002;

/// Body structure (body structure) - 123037004.
///
/// Represents anatomical structures and body parts.
pub const BODY_STRUCTURE: SctId = 123037004;

/// Organism (organism) - 410607006.
///
/// Represents living organisms including microorganisms.
pub const ORGANISM: SctId = 410607006;

/// Substance (substance) - 105590001.
///
/// Represents chemical substances and materials.
pub const SUBSTANCE: SctId = 105590001;

/// Pharmaceutical/biologic product - 373873005.
///
/// Represents medications and biological products.
pub const PHARMACEUTICAL_PRODUCT: SctId = 373873005;

/// Qualifier value - 362981000.
///
/// Represents qualifier values used in postcoordination.
pub const QUALIFIER_VALUE: SctId = 362981000;

/// Observable entity - 363787002.
///
/// Represents things that can be observed or measured.
pub const OBSERVABLE_ENTITY: SctId = 363787002;

/// Event (event) - 272379006.
///
/// Represents events and occurrences.
pub const EVENT: SctId = 272379006;

/// Physical object (physical object) - 260787004.
///
/// Represents physical objects and devices.
pub const PHYSICAL_OBJECT: SctId = 260787004;

/// Specimen (specimen) - 123038009.
///
/// Represents biological specimens.
pub const SPECIMEN: SctId = 123038009;

// =============================================================================
// Common Relationship Types
// =============================================================================

/// IS_A relationship type - 116680003.
///
/// Defines the taxonomic (hierarchical) relationships between concepts.
pub const IS_A: SctId = 116680003;

/// Finding site attribute - 363698007.
///
/// Indicates the body structure where a finding is located.
pub const FINDING_SITE: SctId = 363698007;

/// Associated morphology attribute - 116676008.
///
/// Indicates the morphological abnormality associated with a finding.
pub const ASSOCIATED_MORPHOLOGY: SctId = 116676008;

/// Causative agent attribute - 246075003.
///
/// Indicates the agent causing a condition.
pub const CAUSATIVE_AGENT: SctId = 246075003;

/// Severity attribute - 246112005.
///
/// Indicates the severity of a condition.
pub const SEVERITY: SctId = 246112005;

/// Laterality attribute - 272741003.
///
/// Indicates the side of the body.
pub const LATERALITY: SctId = 272741003;

/// Clinical course attribute - 263502005.
///
/// Indicates the clinical course of a condition.
pub const CLINICAL_COURSE: SctId = 263502005;

/// Interprets attribute - 363714003.
///
/// Indicates what an observation interprets.
pub const INTERPRETS: SctId = 363714003;

/// Has interpretation attribute - 363713009.
///
/// Indicates the interpretation of an observation.
pub const HAS_INTERPRETATION: SctId = 363713009;

// =============================================================================
// Modules
// =============================================================================

/// SNOMED CT core module - 900000000000207008.
///
/// The main module containing core SNOMED CT content.
pub const SNOMED_CT_CORE_MODULE: SctId = 900000000000207008;

/// SNOMED CT model component module - 900000000000012004.
///
/// Contains SNOMED CT model components.
pub const SNOMED_CT_MODEL_COMPONENT_MODULE: SctId = 900000000000012004;

// =============================================================================
// Common Qualifiers
// =============================================================================

/// Mild (qualifier value) - 255604002.
pub const MILD: SctId = 255604002;

/// Moderate (qualifier value) - 6736007.
pub const MODERATE: SctId = 6736007;

/// Severe (qualifier value) - 24484000.
pub const SEVERE: SctId = 24484000;

/// Left (qualifier value) - 7771000.
pub const LEFT: SctId = 7771000;

/// Right (qualifier value) - 24028007.
pub const RIGHT: SctId = 24028007;

/// Bilateral (qualifier value) - 51440002.
pub const BILATERAL: SctId = 51440002;

// =============================================================================
// MRCM Reference Sets
// =============================================================================

/// MRCM Domain Reference Set - 723589008.
///
/// Contains domain definitions for the Machine Readable Concept Model.
pub const MRCM_DOMAIN_REFSET: SctId = 723589008;

/// MRCM Attribute Domain Reference Set - 723604009.
///
/// Defines which attributes are valid in which domains.
pub const MRCM_ATTRIBUTE_DOMAIN_REFSET: SctId = 723604009;

/// MRCM Attribute Range Reference Set - 723592007.
///
/// Defines valid value ranges for attributes.
pub const MRCM_ATTRIBUTE_RANGE_REFSET: SctId = 723592007;

/// Mandatory concept model rule - 723597001.
///
/// Indicates a rule that must be followed for valid expressions.
pub const MANDATORY_CONCEPT_MODEL_RULE: SctId = 723597001;

/// Optional concept model rule - 723598006.
///
/// Indicates a rule that is recommended but not required.
pub const OPTIONAL_CONCEPT_MODEL_RULE: SctId = 723598006;

// =============================================================================
// MRCM Content Types
// =============================================================================

/// All SNOMED CT content - 723596005.
///
/// Content type indicating a rule applies to all content.
pub const ALL_SNOMED_CT_CONTENT: SctId = 723596005;

/// All precoordinated content - 723594008.
///
/// Content type indicating a rule applies to precoordinated content.
pub const ALL_PRECOORDINATED_CONTENT: SctId = 723594008;

/// All postcoordinated content - 723595009.
///
/// Content type indicating a rule applies to postcoordinated content.
pub const ALL_POSTCOORDINATED_CONTENT: SctId = 723595009;

/// All new precoordinated content - 723593002.
///
/// Content type for new precoordinated content.
pub const ALL_NEW_PRECOORDINATED_CONTENT: SctId = 723593002;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_well_known_constants() {
        // Verify some well-known IDs
        assert_eq!(SNOMED_CT_ROOT, 138875005);
        assert_eq!(CLINICAL_FINDING, 404684003);
        assert_eq!(IS_A, 116680003);
        assert_eq!(FINDING_SITE, 363698007);
        assert_eq!(SNOMED_CT_CORE_MODULE, 900000000000207008);
    }

    #[test]
    fn test_hierarchy_ids_are_different() {
        // Ensure no duplicate IDs
        let hierarchies = [
            CLINICAL_FINDING,
            PROCEDURE,
            BODY_STRUCTURE,
            ORGANISM,
            SUBSTANCE,
            PHARMACEUTICAL_PRODUCT,
            QUALIFIER_VALUE,
        ];

        for (i, id1) in hierarchies.iter().enumerate() {
            for (j, id2) in hierarchies.iter().enumerate() {
                if i != j {
                    assert_ne!(id1, id2, "Duplicate hierarchy ID found");
                }
            }
        }
    }
}
