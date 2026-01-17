//! SNOMED CT Identifier (SCTID) type.
//!
//! This module provides a type alias for SNOMED CT identifiers (SCTIDs).
//! SCTIDs are 64-bit unsigned integers that uniquely identify components
//! within SNOMED CT.

/// A SNOMED CT identifier (SCTID).
///
/// SCTIDs are 64-bit unsigned integers that uniquely identify components
/// within SNOMED CT. They follow a specific structure with check digits.
///
/// # Examples
///
/// ```
/// use snomed_types::SctId;
///
/// let concept_id: SctId = 73211009; // Diabetes mellitus
/// let is_a_type: SctId = 116680003; // IS_A relationship type
/// ```
pub type SctId = u64;
