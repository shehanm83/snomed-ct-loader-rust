//! ECL (Expression Constraint Language) integration.
//!
//! This module implements the `EclQueryable` trait for `SnomedStore`,
//! enabling ECL query execution against loaded SNOMED CT data.

use snomed_ecl::SctId;
use snomed_ecl_executor::{
    ConcreteRelationshipInfo, DescriptionInfo, EclQueryable, RelationshipInfo,
};

use crate::SnomedStore;

/// Implements EclQueryable for SnomedStore to enable ECL execution.
impl EclQueryable for SnomedStore {
    fn get_children(&self, concept_id: SctId) -> Vec<SctId> {
        self.get_children(concept_id)
    }

    fn get_parents(&self, concept_id: SctId) -> Vec<SctId> {
        self.get_parents(concept_id)
    }

    fn has_concept(&self, concept_id: SctId) -> bool {
        self.has_concept(concept_id)
    }

    fn all_concept_ids(&self) -> Box<dyn Iterator<Item = SctId> + '_> {
        Box::new(self.concept_ids().copied())
    }

    fn get_refset_members(&self, refset_id: SctId) -> Vec<SctId> {
        SnomedStore::get_refset_members(self, refset_id)
    }

    // Optional: Override advanced methods for better ECL support

    fn get_attributes(&self, concept_id: SctId) -> Vec<RelationshipInfo> {
        const IS_A: SctId = 116680003;

        self.get_outgoing_relationships(concept_id)
            .map(|rels| {
                rels.iter()
                    .filter(|r| r.type_id != IS_A && r.active)
                    .map(|r| RelationshipInfo {
                        type_id: r.type_id,
                        destination_id: r.destination_id,
                        group: r.relationship_group as u16,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_descriptions(&self, concept_id: SctId) -> Vec<DescriptionInfo> {
        SnomedStore::get_descriptions(self, concept_id)
            .map(|descs| {
                descs
                    .iter()
                    .map(|d| DescriptionInfo {
                        description_id: d.id,
                        term: d.term.clone(),
                        language_code: d.language_code.clone(),
                        type_id: d.type_id,
                        case_significance_id: d.case_significance_id,
                        active: d.active,
                        effective_time: Some(d.effective_time),
                        module_id: d.module_id,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_preferred_term(&self, concept_id: SctId) -> Option<String> {
        SnomedStore::get_preferred_term(self, concept_id).map(|s| s.to_string())
    }

    fn is_concept_active(&self, concept_id: SctId) -> bool {
        self.get_concept(concept_id)
            .map(|c| c.active)
            .unwrap_or(false)
    }

    fn get_concept_module(&self, concept_id: SctId) -> Option<SctId> {
        self.get_concept(concept_id).map(|c| c.module_id)
    }

    fn get_concrete_values(&self, _concept_id: SctId) -> Vec<ConcreteRelationshipInfo> {
        // Concrete domain relationships not yet supported
        Vec::new()
    }

    fn get_inbound_relationships(&self, concept_id: SctId) -> Vec<RelationshipInfo> {
        const IS_A: SctId = 116680003;

        self.get_incoming_relationships(concept_id)
            .map(|rels| {
                rels.iter()
                    .filter(|r| r.type_id != IS_A && r.active)
                    .map(|r| RelationshipInfo {
                        type_id: r.type_id,
                        destination_id: r.source_id, // Note: source becomes destination for inbound
                        group: r.relationship_group as u16,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_semantic_tag(&self, concept_id: SctId) -> Option<String> {
        // Extract semantic tag from FSN (text between last parentheses)
        // e.g., "Diabetes mellitus (disorder)" -> "disorder"
        self.get_fsn(concept_id).and_then(|fsn| {
            let term = &fsn.term;
            if let (Some(start), Some(end)) = (term.rfind('('), term.rfind(')')) {
                if start < end {
                    return Some(term[start + 1..end].to_string());
                }
            }
            None
        })
    }

    fn get_concept_effective_time(&self, concept_id: SctId) -> Option<u32> {
        self.get_concept(concept_id).map(|c| c.effective_time)
    }

    fn is_concept_primitive(&self, concept_id: SctId) -> Option<bool> {
        use snomed_types::DefinitionStatus;
        self.get_concept(concept_id)
            .map(|c| c.definition_status_id == DefinitionStatus::PRIMITIVE_ID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_ecl_executor::EclExecutor;
    use snomed_types::{CharacteristicType, DefinitionStatus, ModifierType, Rf2Concept, Rf2Relationship};

    fn create_test_store() -> SnomedStore {
        let mut store = SnomedStore::new();

        // Create a small hierarchy:
        // 138875005 (root)
        //   └── 404684003 (clinical finding)
        //         ├── 73211009 (diabetes mellitus)
        //         │     ├── 46635009 (type 1 diabetes)
        //         │     └── 44054006 (type 2 diabetes)
        //         └── 22298006 (myocardial infarction)

        let concepts = vec![
            make_concept(138875005), // SNOMED root
            make_concept(404684003), // Clinical finding
            make_concept(73211009),  // Diabetes mellitus
            make_concept(46635009),  // Type 1 diabetes
            make_concept(44054006),  // Type 2 diabetes
            make_concept(22298006),  // Myocardial infarction
        ];

        store.insert_concepts(concepts);

        // IS_A relationships
        let relationships = vec![
            make_is_a(1, 404684003, 138875005), // Clinical finding IS_A Root
            make_is_a(2, 73211009, 404684003),  // Diabetes IS_A Clinical finding
            make_is_a(3, 46635009, 73211009),   // Type 1 IS_A Diabetes
            make_is_a(4, 44054006, 73211009),   // Type 2 IS_A Diabetes
            make_is_a(5, 22298006, 404684003),  // MI IS_A Clinical finding
        ];

        store.insert_relationships(relationships);

        store
    }

    fn make_concept(id: SctId) -> Rf2Concept {
        Rf2Concept {
            id,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            definition_status_id: DefinitionStatus::PRIMITIVE_ID,
        }
    }

    fn make_is_a(id: SctId, source: SctId, dest: SctId) -> Rf2Relationship {
        Rf2Relationship {
            id,
            effective_time: 20020131,
            active: true,
            module_id: 900000000000207008,
            source_id: source,
            destination_id: dest,
            relationship_group: 0,
            type_id: 116680003, // IS_A
            characteristic_type_id: CharacteristicType::INFERRED_ID,
            modifier_id: ModifierType::EXISTENTIAL_ID,
        }
    }

    #[test]
    fn test_ecl_queryable_children() {
        let store = create_test_store();

        let children = EclQueryable::get_children(&store, 73211009); // Diabetes
        assert_eq!(children.len(), 2);
        assert!(children.contains(&46635009)); // Type 1
        assert!(children.contains(&44054006)); // Type 2
    }

    #[test]
    fn test_ecl_queryable_parents() {
        let store = create_test_store();

        let parents = EclQueryable::get_parents(&store, 73211009); // Diabetes
        assert_eq!(parents, vec![404684003]); // Clinical finding
    }

    #[test]
    fn test_ecl_executor_descendant_query() {
        let store = create_test_store();
        let executor = EclExecutor::new(&store);

        // Find all descendants of Diabetes (excluding self)
        let result = executor.execute("< 73211009").unwrap();
        assert_eq!(result.count(), 2);
        assert!(result.contains(46635009));
        assert!(result.contains(44054006));
    }

    #[test]
    fn test_ecl_executor_descendant_or_self_query() {
        let store = create_test_store();
        let executor = EclExecutor::new(&store);

        // Find Diabetes and all descendants
        let result = executor.execute("<< 73211009").unwrap();
        assert_eq!(result.count(), 3);
        assert!(result.contains(73211009));
        assert!(result.contains(46635009));
        assert!(result.contains(44054006));
    }

    #[test]
    fn test_ecl_executor_ancestor_query() {
        let store = create_test_store();
        let executor = EclExecutor::new(&store);

        // Find ancestors of Type 1 diabetes
        let result = executor.execute(">> 46635009").unwrap();
        assert!(result.contains(46635009)); // Self
        assert!(result.contains(73211009)); // Diabetes
        assert!(result.contains(404684003)); // Clinical finding
        assert!(result.contains(138875005)); // Root
    }

    #[test]
    fn test_ecl_executor_and_query() {
        let store = create_test_store();
        let executor = EclExecutor::new(&store);

        // Find concepts that are descendants of both Clinical finding AND Diabetes
        // This should return Type 1 and Type 2 (children of Diabetes which is child of CF)
        let result = executor.execute("<< 404684003 AND << 73211009").unwrap();

        // Diabetes and its subtypes are descendants of Clinical finding
        // AND descendants of Diabetes (including self)
        assert!(result.contains(73211009)); // Diabetes is descendant of CF
        assert!(result.contains(46635009)); // Type 1
        assert!(result.contains(44054006)); // Type 2
    }

    #[test]
    fn test_ecl_executor_minus_query() {
        let store = create_test_store();
        let executor = EclExecutor::new(&store);

        // Clinical finding descendants minus Diabetes descendants
        let result = executor.execute("<< 404684003 MINUS << 73211009").unwrap();

        assert!(result.contains(404684003)); // Clinical finding itself
        assert!(result.contains(22298006)); // MI (not under Diabetes)
        assert!(!result.contains(73211009)); // Diabetes excluded
        assert!(!result.contains(46635009)); // Type 1 excluded
    }

    #[test]
    fn test_ecl_executor_self_query() {
        let store = create_test_store();
        let executor = EclExecutor::new(&store);

        let result = executor.execute("73211009").unwrap();
        assert_eq!(result.count(), 1);
        assert!(result.contains(73211009));
    }

    #[test]
    fn test_ecl_executor_matches() {
        let store = create_test_store();
        let executor = EclExecutor::new(&store);

        // Type 1 diabetes is a descendant of Diabetes
        assert!(executor.matches(46635009, "<< 73211009").unwrap());

        // MI is NOT a descendant of Diabetes
        assert!(!executor.matches(22298006, "<< 73211009").unwrap());
    }
}
