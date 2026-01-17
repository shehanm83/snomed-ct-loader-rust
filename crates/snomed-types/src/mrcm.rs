//! MRCM (Machine Readable Concept Model) type definitions.
//!
//! This module provides types for working with SNOMED CT MRCM reference sets,
//! which define constraints for valid SNOMED CT expressions.
//!
//! # Overview
//!
//! The MRCM consists of three main reference sets:
//!
//! 1. **Domain Reference Set** - Defines semantic domains where attributes can be applied
//! 2. **Attribute Domain Reference Set** - Defines which attributes are valid in which domains
//! 3. **Attribute Range Reference Set** - Defines valid value ranges for attributes
//!
//! # Examples
//!
//! ```
//! use snomed_types::mrcm::{Cardinality, MrcmDomain, MrcmAttributeDomain, MrcmAttributeRange};
//!
//! // Parse cardinality constraints
//! let unbounded = Cardinality::parse("0..*").unwrap();
//! assert!(unbounded.allows(0));
//! assert!(unbounded.allows(100));
//!
//! let bounded = Cardinality::parse("0..1").unwrap();
//! assert!(bounded.allows(0));
//! assert!(bounded.allows(1));
//! assert!(!bounded.allows(2));
//! ```

use crate::SctId;

/// Error type for cardinality parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardinalityParseError {
    /// Invalid format - expected "min..max"
    InvalidFormat(String),
    /// Invalid minimum value
    InvalidMin(String),
    /// Invalid maximum value
    InvalidMax(String),
}

impl std::fmt::Display for CardinalityParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat(s) => write!(f, "invalid cardinality format: '{}' (expected min..max)", s),
            Self::InvalidMin(s) => write!(f, "invalid cardinality minimum: '{}'", s),
            Self::InvalidMax(s) => write!(f, "invalid cardinality maximum: '{}'", s),
        }
    }
}

impl std::error::Error for CardinalityParseError {}

/// Cardinality constraint for MRCM attributes.
///
/// Represents constraints like "0..*", "0..1", "1..1", "1..*".
///
/// # Examples
///
/// ```
/// use snomed_types::mrcm::Cardinality;
///
/// // Unbounded cardinality (0..*)
/// let card = Cardinality::parse("0..*").unwrap();
/// assert_eq!(card.min, 0);
/// assert_eq!(card.max, None);
/// assert!(card.allows(0));
/// assert!(card.allows(100));
///
/// // Bounded cardinality (0..1)
/// let card = Cardinality::parse("0..1").unwrap();
/// assert!(card.allows(0));
/// assert!(card.allows(1));
/// assert!(!card.allows(2));
///
/// // Exact cardinality (1..1)
/// let card = Cardinality::parse("1..1").unwrap();
/// assert!(!card.allows(0));
/// assert!(card.allows(1));
/// assert!(!card.allows(2));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cardinality {
    /// Minimum occurrences (inclusive).
    pub min: u32,
    /// Maximum occurrences (inclusive). None means unbounded (*).
    pub max: Option<u32>,
}

impl Cardinality {
    /// Creates a new cardinality with explicit min and max.
    pub const fn new(min: u32, max: Option<u32>) -> Self {
        Self { min, max }
    }

    /// Creates an unbounded cardinality (0..*).
    pub const fn unbounded() -> Self {
        Self { min: 0, max: None }
    }

    /// Creates an optional cardinality (0..1).
    pub const fn optional() -> Self {
        Self { min: 0, max: Some(1) }
    }

    /// Creates a required single cardinality (1..1).
    pub const fn required() -> Self {
        Self { min: 1, max: Some(1) }
    }

    /// Creates a required unbounded cardinality (1..*).
    pub const fn one_or_more() -> Self {
        Self { min: 1, max: None }
    }

    /// Parses a cardinality from a string like "0..*", "0..1", "1..1".
    ///
    /// # Arguments
    /// * `s` - Cardinality string in format "min..max" where max can be "*" for unbounded
    ///
    /// # Examples
    ///
    /// ```
    /// use snomed_types::mrcm::Cardinality;
    ///
    /// assert_eq!(Cardinality::parse("0..*").unwrap(), Cardinality::unbounded());
    /// assert_eq!(Cardinality::parse("0..1").unwrap(), Cardinality::optional());
    /// assert_eq!(Cardinality::parse("1..1").unwrap(), Cardinality::required());
    /// assert_eq!(Cardinality::parse("1..*").unwrap(), Cardinality::one_or_more());
    /// ```
    pub fn parse(s: &str) -> Result<Self, CardinalityParseError> {
        let parts: Vec<&str> = s.split("..").collect();
        if parts.len() != 2 {
            return Err(CardinalityParseError::InvalidFormat(s.to_string()));
        }

        let min = parts[0]
            .parse::<u32>()
            .map_err(|_| CardinalityParseError::InvalidMin(parts[0].to_string()))?;

        let max = if parts[1] == "*" {
            None
        } else {
            Some(
                parts[1]
                    .parse::<u32>()
                    .map_err(|_| CardinalityParseError::InvalidMax(parts[1].to_string()))?,
            )
        };

        Ok(Self { min, max })
    }

    /// Returns true if the given count satisfies this cardinality constraint.
    ///
    /// # Examples
    ///
    /// ```
    /// use snomed_types::mrcm::Cardinality;
    ///
    /// let card = Cardinality::parse("0..1").unwrap();
    /// assert!(card.allows(0));
    /// assert!(card.allows(1));
    /// assert!(!card.allows(2));
    ///
    /// let unbounded = Cardinality::parse("0..*").unwrap();
    /// assert!(unbounded.allows(0));
    /// assert!(unbounded.allows(1000));
    /// ```
    pub fn allows(&self, count: u32) -> bool {
        count >= self.min && self.max.is_none_or(|max| count <= max)
    }

    /// Returns true if this cardinality is unbounded (max = *).
    pub fn is_unbounded(&self) -> bool {
        self.max.is_none()
    }

    /// Returns true if this cardinality requires at least one occurrence.
    pub fn is_required(&self) -> bool {
        self.min >= 1
    }
}

impl std::fmt::Display for Cardinality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.max {
            Some(max) => write!(f, "{}..{}", self.min, max),
            None => write!(f, "{}..*", self.min),
        }
    }
}

/// MRCM Domain reference set record.
///
/// Defines a semantic domain where specific attributes can be applied.
/// For example, the "Clinical finding" domain allows finding site, severity, etc.
///
/// # RF2 File
/// Pattern: `der2_cRefset_MRCMDomainSnapshot_*.txt`
///
/// # Fields
/// The `domain_constraint` and `proximal_primitive_constraint` fields contain ECL
/// expressions that define which concepts belong to this domain.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MrcmDomain {
    /// Unique row identifier (UUID in RF2, stored as string).
    pub id: String,
    /// Effective time (YYYYMMDD format).
    pub effective_time: u32,
    /// Whether this record is active.
    pub active: bool,
    /// Module that owns this record.
    pub module_id: SctId,
    /// Reference set identifier (should be MRCM Domain refset).
    pub refset_id: SctId,
    /// The domain concept ID (e.g., Clinical finding 404684003).
    pub referenced_component_id: SctId,
    /// ECL expression defining the domain.
    pub domain_constraint: String,
    /// Parent domain concept (if any).
    pub parent_domain: Option<SctId>,
    /// ECL for proximal primitive supertypes.
    pub proximal_primitive_constraint: String,
    /// Additional refinement for proximal primitives.
    pub proximal_primitive_refinement: Option<String>,
    /// Template for precoordinated expressions.
    pub domain_template_for_precoordination: String,
    /// Template for postcoordinated expressions.
    pub domain_template_for_postcoordination: String,
    /// Optional URL to editorial guide.
    pub guide_url: Option<String>,
}

/// MRCM Attribute Domain reference set record.
///
/// Defines which attributes are valid in which domains and their cardinality.
///
/// # RF2 File
/// Pattern: `der2_cRefset_MRCMAttributeDomainSnapshot_*.txt`
///
/// # Example
/// An attribute domain record might specify that "Finding site" (363698007)
/// is valid in the "Clinical finding" domain with cardinality 0..* overall
/// and 0..1 within a role group, and must be grouped.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MrcmAttributeDomain {
    /// Unique row identifier (UUID in RF2, stored as string).
    pub id: String,
    /// Effective time (YYYYMMDD format).
    pub effective_time: u32,
    /// Whether this record is active.
    pub active: bool,
    /// Module that owns this record.
    pub module_id: SctId,
    /// Reference set identifier (should be MRCM Attribute Domain refset).
    pub refset_id: SctId,
    /// The attribute concept ID (e.g., Finding site 363698007).
    pub referenced_component_id: SctId,
    /// The domain where this attribute applies.
    pub domain_id: SctId,
    /// Whether this attribute must be in a role group.
    pub grouped: bool,
    /// Overall cardinality for this attribute (e.g., "0..*").
    pub attribute_cardinality: Cardinality,
    /// Cardinality within a role group (e.g., "0..1").
    pub attribute_in_group_cardinality: Cardinality,
    /// Rule strength: mandatory (723597001) or optional (723598006).
    pub rule_strength_id: SctId,
    /// Content type where this rule applies.
    pub content_type_id: SctId,
}

impl MrcmAttributeDomain {
    /// Returns true if this is a mandatory rule.
    pub fn is_mandatory(&self) -> bool {
        self.rule_strength_id == super::well_known::MANDATORY_CONCEPT_MODEL_RULE
    }

    /// Returns true if this attribute must be grouped.
    pub fn is_grouped(&self) -> bool {
        self.grouped
    }
}

/// MRCM Attribute Range reference set record.
///
/// Defines valid value ranges for attributes using ECL expressions.
///
/// # RF2 File
/// Pattern: `der2_cRefset_MRCMAttributeRangeSnapshot_*.txt`
///
/// # Example
/// An attribute range record might specify that "Finding site" (363698007)
/// has range constraint "<< 123037004 |Body structure|", meaning only
/// descendants of Body structure are valid values.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MrcmAttributeRange {
    /// Unique row identifier (UUID in RF2, stored as string).
    pub id: String,
    /// Effective time (YYYYMMDD format).
    pub effective_time: u32,
    /// Whether this record is active.
    pub active: bool,
    /// Module that owns this record.
    pub module_id: SctId,
    /// Reference set identifier (should be MRCM Attribute Range refset).
    pub refset_id: SctId,
    /// The attribute concept ID (e.g., Finding site 363698007).
    pub referenced_component_id: SctId,
    /// ECL expression defining valid values for this attribute.
    pub range_constraint: String,
    /// Additional validation rule (optional).
    pub attribute_rule: Option<String>,
    /// Rule strength: mandatory (723597001) or optional (723598006).
    pub rule_strength_id: SctId,
    /// Content type where this rule applies.
    pub content_type_id: SctId,
}

impl MrcmAttributeRange {
    /// Returns true if this is a mandatory rule.
    pub fn is_mandatory(&self) -> bool {
        self.rule_strength_id == super::well_known::MANDATORY_CONCEPT_MODEL_RULE
    }

    /// Returns true if this record has an additional attribute rule.
    pub fn has_attribute_rule(&self) -> bool {
        self.attribute_rule.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinality_parse_unbounded() {
        let card = Cardinality::parse("0..*").unwrap();
        assert_eq!(card.min, 0);
        assert_eq!(card.max, None);
        assert!(card.is_unbounded());
    }

    #[test]
    fn test_cardinality_parse_bounded() {
        let card = Cardinality::parse("1..1").unwrap();
        assert_eq!(card.min, 1);
        assert_eq!(card.max, Some(1));
        assert!(!card.is_unbounded());
        assert!(card.is_required());
    }

    #[test]
    fn test_cardinality_parse_optional() {
        let card = Cardinality::parse("0..1").unwrap();
        assert_eq!(card.min, 0);
        assert_eq!(card.max, Some(1));
        assert!(!card.is_required());
    }

    #[test]
    fn test_cardinality_parse_one_or_more() {
        let card = Cardinality::parse("1..*").unwrap();
        assert_eq!(card.min, 1);
        assert_eq!(card.max, None);
        assert!(card.is_unbounded());
        assert!(card.is_required());
    }

    #[test]
    fn test_cardinality_allows() {
        let card = Cardinality::parse("0..1").unwrap();
        assert!(card.allows(0));
        assert!(card.allows(1));
        assert!(!card.allows(2));

        let unbounded = Cardinality::parse("0..*").unwrap();
        assert!(unbounded.allows(0));
        assert!(unbounded.allows(100));
        assert!(unbounded.allows(u32::MAX));

        let required = Cardinality::parse("1..1").unwrap();
        assert!(!required.allows(0));
        assert!(required.allows(1));
        assert!(!required.allows(2));

        let one_or_more = Cardinality::parse("1..*").unwrap();
        assert!(!one_or_more.allows(0));
        assert!(one_or_more.allows(1));
        assert!(one_or_more.allows(100));
    }

    #[test]
    fn test_cardinality_parse_error_invalid_format() {
        assert!(Cardinality::parse("0").is_err());
        assert!(Cardinality::parse("0-1").is_err());
        assert!(Cardinality::parse("").is_err());
    }

    #[test]
    fn test_cardinality_parse_error_invalid_min() {
        assert!(Cardinality::parse("abc..1").is_err());
        assert!(Cardinality::parse("-1..1").is_err());
    }

    #[test]
    fn test_cardinality_parse_error_invalid_max() {
        assert!(Cardinality::parse("0..abc").is_err());
    }

    #[test]
    fn test_cardinality_display() {
        assert_eq!(Cardinality::parse("0..*").unwrap().to_string(), "0..*");
        assert_eq!(Cardinality::parse("0..1").unwrap().to_string(), "0..1");
        assert_eq!(Cardinality::parse("1..1").unwrap().to_string(), "1..1");
        assert_eq!(Cardinality::parse("1..*").unwrap().to_string(), "1..*");
    }

    #[test]
    fn test_cardinality_constructors() {
        assert_eq!(Cardinality::unbounded(), Cardinality::parse("0..*").unwrap());
        assert_eq!(Cardinality::optional(), Cardinality::parse("0..1").unwrap());
        assert_eq!(Cardinality::required(), Cardinality::parse("1..1").unwrap());
        assert_eq!(Cardinality::one_or_more(), Cardinality::parse("1..*").unwrap());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_cardinality_serde() {
        let card = Cardinality::parse("0..1").unwrap();
        let json = serde_json::to_string(&card).unwrap();
        let parsed: Cardinality = serde_json::from_str(&json).unwrap();
        assert_eq!(card, parsed);
    }
}
