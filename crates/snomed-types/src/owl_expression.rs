//! OWL Expression refset member type for SNOMED CT RF2 files.
//!
//! OWL expressions provide formal logical definitions for SNOMED CT concepts
//! using OWL 2 EL profile syntax. These are stored in reference sets with
//! pattern `sct2_sRefset_OWL*.txt`.
//!
//! # Example
//!
//! ```
//! use snomed_types::{Rf2OwlExpression, SctId};
//!
//! let owl = Rf2OwlExpression {
//!     id: 12345678901234,
//!     effective_time: 20230101,
//!     active: true,
//!     module_id: 900000000000207008,
//!     refset_id: 733073007,
//!     referenced_component_id: 404684003,
//!     owl_expression: "SubClassOf(:404684003 :138875005)".to_string(),
//! };
//!
//! assert!(owl.active);
//! assert!(owl.is_subclass_axiom());
//! ```

use crate::SctId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// OWL Expression refset member from RF2 OWL Expression Reference Set files.
///
/// Contains OWL 2 EL axioms that provide formal logical definitions for
/// SNOMED CT concepts. Common axiom types include:
/// - SubClassOf - defines hierarchical relationships
/// - EquivalentClasses - defines concept equivalence
/// - SubObjectPropertyOf - defines property hierarchies
/// - ObjectPropertyChain - defines property chains
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rf2OwlExpression {
    /// Unique identifier for this refset member (UUID as SCTID).
    pub id: SctId,

    /// Effective time in YYYYMMDD format.
    pub effective_time: u32,

    /// Whether this member is currently active.
    pub active: bool,

    /// Module containing this member.
    pub module_id: SctId,

    /// Reference set this member belongs to.
    /// Common values:
    /// - 733073007 = OWL Axiom Reference Set
    /// - 762103008 = OWL Ontology Reference Set
    pub refset_id: SctId,

    /// The concept this OWL expression defines or relates to.
    pub referenced_component_id: SctId,

    /// The OWL expression in OWL Functional Syntax.
    /// Examples:
    /// - `SubClassOf(:404684003 :138875005)`
    /// - `EquivalentClasses(:404684003 ObjectIntersectionOf(:138875005 ...))`
    pub owl_expression: String,
}

impl Rf2OwlExpression {
    /// Well-known refset ID for OWL Axiom Reference Set.
    pub const OWL_AXIOM_REFSET_ID: SctId = 733073007;

    /// Well-known refset ID for OWL Ontology Reference Set.
    pub const OWL_ONTOLOGY_REFSET_ID: SctId = 762103008;

    /// Returns true if this is from the OWL Axiom refset.
    pub fn is_axiom(&self) -> bool {
        self.refset_id == Self::OWL_AXIOM_REFSET_ID
    }

    /// Returns true if this is from the OWL Ontology refset.
    pub fn is_ontology(&self) -> bool {
        self.refset_id == Self::OWL_ONTOLOGY_REFSET_ID
    }

    /// Returns true if the OWL expression is a SubClassOf axiom.
    pub fn is_subclass_axiom(&self) -> bool {
        self.owl_expression.starts_with("SubClassOf")
    }

    /// Returns true if the OWL expression is an EquivalentClasses axiom.
    pub fn is_equivalent_classes_axiom(&self) -> bool {
        self.owl_expression.starts_with("EquivalentClasses")
    }

    /// Returns true if the OWL expression is an ObjectPropertyChain axiom.
    pub fn is_property_chain_axiom(&self) -> bool {
        self.owl_expression.contains("ObjectPropertyChain")
    }

    /// Returns true if the OWL expression is a SubObjectPropertyOf axiom.
    pub fn is_sub_property_axiom(&self) -> bool {
        self.owl_expression.starts_with("SubObjectPropertyOf")
    }

    /// Returns true if the OWL expression is a TransitiveObjectProperty axiom.
    pub fn is_transitive_property_axiom(&self) -> bool {
        self.owl_expression.starts_with("TransitiveObjectProperty")
    }

    /// Returns true if the OWL expression is a ReflexiveObjectProperty axiom.
    pub fn is_reflexive_property_axiom(&self) -> bool {
        self.owl_expression.starts_with("ReflexiveObjectProperty")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_owl_expression(owl_expr: &str) -> Rf2OwlExpression {
        Rf2OwlExpression {
            id: 12345678901234,
            effective_time: 20230101,
            active: true,
            module_id: 900000000000207008,
            refset_id: Rf2OwlExpression::OWL_AXIOM_REFSET_ID,
            referenced_component_id: 404684003,
            owl_expression: owl_expr.to_string(),
        }
    }

    #[test]
    fn test_is_axiom() {
        let owl = make_owl_expression("SubClassOf(:404684003 :138875005)");
        assert!(owl.is_axiom());
        assert!(!owl.is_ontology());
    }

    #[test]
    fn test_is_subclass_axiom() {
        let owl = make_owl_expression("SubClassOf(:404684003 :138875005)");
        assert!(owl.is_subclass_axiom());
        assert!(!owl.is_equivalent_classes_axiom());
    }

    #[test]
    fn test_is_equivalent_classes_axiom() {
        let owl = make_owl_expression(
            "EquivalentClasses(:404684003 ObjectIntersectionOf(:138875005 :123456789))",
        );
        assert!(owl.is_equivalent_classes_axiom());
        assert!(!owl.is_subclass_axiom());
    }

    #[test]
    fn test_is_property_chain_axiom() {
        let owl = make_owl_expression(
            "SubObjectPropertyOf(ObjectPropertyChain(:123 :456) :789)",
        );
        assert!(owl.is_property_chain_axiom());
        assert!(owl.is_sub_property_axiom());
    }

    #[test]
    fn test_is_transitive_property() {
        let owl = make_owl_expression("TransitiveObjectProperty(:116680003)");
        assert!(owl.is_transitive_property_axiom());
    }
}
