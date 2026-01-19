//! gRPC server implementation.

use std::sync::Arc;
use std::time::Instant;

use snomed_loader::{EclExecutor, SnomedStore};
use tonic::{Request, Response, Status};

use crate::proto::{
    concept_service_server::ConceptService,
    ecl_service_server::EclService,
    refset_service_server::RefsetService,
    search_service_server::SearchService,
    Concept, Description, Relationship, ConcreteRelationship, ConcreteValueType,
    OwlExpression, AssociationMember,
    GetConceptRequest, GetConceptResponse,
    GetParentsRequest, GetParentsResponse, GetChildrenRequest, GetChildrenResponse,
    IsDescendantOfRequest, IsDescendantOfResponse,
    SearchRequest, SearchResponse,
    ExecuteEclRequest, ExecuteEclResponse,
    MatchesEclRequest, MatchesEclResponse,
    GetDescendantsRequest, GetDescendantsResponse,
    GetAncestorsRequest, GetAncestorsResponse,
    ExplainEclRequest, ExplainEclResponse, QueryPlanStep,
    IsSubsumedByRequest, IsSubsumedByResponse,
    GetDirectParentsRequest, GetDirectParentsResponse,
    GetDirectChildrenRequest, GetDirectChildrenResponse,
    // New messages for functional gaps
    GetConceptsBatchRequest, GetConceptsBatchResponse,
    GetRelationshipsRequest, GetRelationshipsResponse,
    GetConcreteRelationshipsRequest, GetConcreteRelationshipsResponse,
    GetOwlExpressionsRequest, GetOwlExpressionsResponse,
    GetPreferredTermRequest, GetPreferredTermResponse,
    GetAssociationsRequest, GetAssociationsResponse,
    GetReplacementConceptRequest, GetReplacementConceptResponse,
    GetRefsetMembersRequest, GetRefsetMembersResponse,
    GetRefsetsForConceptRequest, GetRefsetsForConceptResponse,
};

/// SNOMED CT gRPC Server.
#[derive(Clone)]
pub struct SnomedServer {
    store: Arc<SnomedStore>,
}

impl SnomedServer {
    /// Creates a new server with the given store.
    pub fn new(store: SnomedStore) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    /// Returns a reference to the store.
    pub fn store(&self) -> &SnomedStore {
        &self.store
    }

    /// Convert internal concept to proto Concept
    fn to_proto_concept(&self, id: snomed_types::SctId) -> Option<Concept> {
        let rf2_concept = self.store.get_concept(id)?;
        let fsn = self.store.get_fsn(id)
            .map(|d| d.term.clone())
            .unwrap_or_default();

        Some(Concept {
            id: rf2_concept.id,
            effective_time: rf2_concept.effective_time,
            active: rf2_concept.active,
            module_id: rf2_concept.module_id,
            definition_status_id: rf2_concept.definition_status_id,
            fsn,
        })
    }

    /// Convert internal description to proto Description
    fn to_proto_description(desc: &snomed_types::Rf2Description) -> Description {
        Description {
            id: desc.id,
            concept_id: desc.concept_id,
            language_code: desc.language_code.clone(),
            type_id: desc.type_id,
            term: desc.term.clone(),
            active: desc.active,
        }
    }

    /// Convert internal relationship to proto Relationship
    fn to_proto_relationship(rel: &snomed_types::Rf2Relationship) -> Relationship {
        Relationship {
            id: rel.id,
            source_id: rel.source_id,
            destination_id: rel.destination_id,
            type_id: rel.type_id,
            relationship_group: rel.relationship_group as u32,
            active: rel.active,
            characteristic_type_id: rel.characteristic_type_id,
        }
    }

    /// Convert internal concrete relationship to proto ConcreteRelationship
    fn to_proto_concrete_relationship(
        rel: &snomed_types::Rf2ConcreteRelationship,
    ) -> ConcreteRelationship {
        let (value, value_type) = match &rel.value {
            snomed_types::ConcreteValue::String(s) => {
                (s.clone(), ConcreteValueType::String as i32)
            }
            snomed_types::ConcreteValue::Integer(i) => {
                (i.to_string(), ConcreteValueType::Integer as i32)
            }
            snomed_types::ConcreteValue::Decimal(d) => {
                (d.to_string(), ConcreteValueType::Decimal as i32)
            }
        };

        ConcreteRelationship {
            id: rel.id,
            source_id: rel.source_id,
            value,
            value_type,
            type_id: rel.type_id,
            relationship_group: rel.relationship_group as u32,
            active: rel.active,
        }
    }

    /// Convert internal OWL expression to proto OwlExpression
    fn to_proto_owl_expression(expr: &snomed_types::Rf2OwlExpression) -> OwlExpression {
        OwlExpression {
            id: expr.id,
            refset_id: expr.refset_id,
            referenced_concept_id: expr.referenced_component_id,
            owl_expression: expr.owl_expression.clone(),
            active: expr.active,
        }
    }

    /// Convert internal association to proto AssociationMember
    fn to_proto_association(assoc: &snomed_types::Rf2AssociationRefsetMember) -> AssociationMember {
        AssociationMember {
            id: assoc.id,
            refset_id: assoc.refset_id,
            source_component_id: assoc.referenced_component_id,
            target_component_id: assoc.target_component_id,
            active: assoc.active,
        }
    }
}

#[tonic::async_trait]
impl ConceptService for SnomedServer {
    async fn get_concept(
        &self,
        request: Request<GetConceptRequest>,
    ) -> Result<Response<GetConceptResponse>, Status> {
        let id = request.into_inner().id;

        let concept = self.to_proto_concept(id);

        let descriptions = self.store.get_descriptions(id)
            .map(|descs| descs.iter().map(Self::to_proto_description).collect())
            .unwrap_or_default();

        Ok(Response::new(GetConceptResponse {
            concept,
            descriptions,
        }))
    }

    async fn get_parents(
        &self,
        request: Request<GetParentsRequest>,
    ) -> Result<Response<GetParentsResponse>, Status> {
        let id = request.into_inner().id;

        let parent_ids = self.store.get_parents(id);
        let parents: Vec<Concept> = parent_ids
            .into_iter()
            .filter_map(|pid| self.to_proto_concept(pid))
            .collect();

        Ok(Response::new(GetParentsResponse { parents }))
    }

    async fn get_children(
        &self,
        request: Request<GetChildrenRequest>,
    ) -> Result<Response<GetChildrenResponse>, Status> {
        let id = request.into_inner().id;

        let child_ids = self.store.get_children(id);
        let children: Vec<Concept> = child_ids
            .into_iter()
            .filter_map(|cid| self.to_proto_concept(cid))
            .collect();

        Ok(Response::new(GetChildrenResponse { children }))
    }

    async fn is_descendant_of(
        &self,
        request: Request<IsDescendantOfRequest>,
    ) -> Result<Response<IsDescendantOfResponse>, Status> {
        let req = request.into_inner();
        let concept_id = req.concept_id;
        let ancestor_id = req.ancestor_id;

        // Use optimized O(1) lookup if transitive closure is built
        let is_descendant = self.store.is_descendant_of(concept_id, ancestor_id);

        Ok(Response::new(IsDescendantOfResponse { is_descendant }))
    }

    async fn get_concepts_batch(
        &self,
        request: Request<GetConceptsBatchRequest>,
    ) -> Result<Response<GetConceptsBatchResponse>, Status> {
        let req = request.into_inner();
        let ids = req.ids;

        let mut concepts = Vec::new();
        let mut not_found = Vec::new();

        for id in ids {
            if let Some(concept) = self.to_proto_concept(id) {
                concepts.push(concept);
            } else {
                not_found.push(id);
            }
        }

        Ok(Response::new(GetConceptsBatchResponse { concepts, not_found }))
    }

    async fn get_relationships(
        &self,
        request: Request<GetRelationshipsRequest>,
    ) -> Result<Response<GetRelationshipsResponse>, Status> {
        let req = request.into_inner();
        let concept_id = req.concept_id;
        let type_filter = req.type_filter;
        let include_incoming = req.include_incoming;

        // Get outgoing relationships
        let outgoing: Vec<Relationship> = self
            .store
            .get_outgoing_relationships(concept_id)
            .map(|rels| {
                rels.iter()
                    .filter(|r| type_filter.is_empty() || type_filter.contains(&r.type_id))
                    .map(Self::to_proto_relationship)
                    .collect()
            })
            .unwrap_or_default();

        // Get incoming relationships if requested
        let incoming: Vec<Relationship> = if include_incoming {
            self.store
                .get_incoming_relationships(concept_id)
                .map(|rels| {
                    rels.iter()
                        .filter(|r| type_filter.is_empty() || type_filter.contains(&r.type_id))
                        .map(Self::to_proto_relationship)
                        .collect()
                })
                .unwrap_or_default()
        } else {
            vec![]
        };

        Ok(Response::new(GetRelationshipsResponse { outgoing, incoming }))
    }

    async fn get_concrete_relationships(
        &self,
        request: Request<GetConcreteRelationshipsRequest>,
    ) -> Result<Response<GetConcreteRelationshipsResponse>, Status> {
        let req = request.into_inner();
        let concept_id = req.concept_id;
        let type_filter = req.type_filter;

        let relationships: Vec<ConcreteRelationship> = self
            .store
            .get_concrete_relationships(concept_id)
            .map(|rels| {
                rels.iter()
                    .filter(|r| type_filter.is_empty() || type_filter.contains(&r.type_id))
                    .map(Self::to_proto_concrete_relationship)
                    .collect()
            })
            .unwrap_or_default();

        Ok(Response::new(GetConcreteRelationshipsResponse { relationships }))
    }

    async fn get_owl_expressions(
        &self,
        request: Request<GetOwlExpressionsRequest>,
    ) -> Result<Response<GetOwlExpressionsResponse>, Status> {
        let concept_id = request.into_inner().concept_id;

        let expressions: Vec<OwlExpression> = self
            .store
            .get_owl_expressions(concept_id)
            .map(|exprs| exprs.iter().map(Self::to_proto_owl_expression).collect())
            .unwrap_or_default();

        Ok(Response::new(GetOwlExpressionsResponse { expressions }))
    }

    async fn get_preferred_term(
        &self,
        request: Request<GetPreferredTermRequest>,
    ) -> Result<Response<GetPreferredTermResponse>, Status> {
        let req = request.into_inner();
        let concept_id = req.concept_id;
        let language_refset_id = req.language_refset_id;

        let (term, found) = if let Some(t) =
            self.store.get_preferred_term_for_language(concept_id, language_refset_id)
        {
            (t.to_string(), true)
        } else {
            // Fall back to general preferred term if language-specific not found
            if let Some(t) = self.store.get_preferred_term(concept_id) {
                (t.to_string(), true)
            } else {
                (String::new(), false)
            }
        };

        Ok(Response::new(GetPreferredTermResponse { term, found }))
    }

    async fn get_associations(
        &self,
        request: Request<GetAssociationsRequest>,
    ) -> Result<Response<GetAssociationsResponse>, Status> {
        let concept_id = request.into_inner().concept_id;

        let associations: Vec<AssociationMember> = self
            .store
            .get_associations_for_concept(concept_id)
            .map(|assocs| assocs.iter().map(Self::to_proto_association).collect())
            .unwrap_or_default();

        Ok(Response::new(GetAssociationsResponse { associations }))
    }

    async fn get_replacement_concept(
        &self,
        request: Request<GetReplacementConceptRequest>,
    ) -> Result<Response<GetReplacementConceptResponse>, Status> {
        let inactive_concept_id = request.into_inner().inactive_concept_id;

        let (replacement_id, found) = if let Some(id) =
            self.store.get_replacement_concept(inactive_concept_id)
        {
            (id, true)
        } else {
            (0, false)
        };

        Ok(Response::new(GetReplacementConceptResponse {
            replacement_id,
            found,
        }))
    }
}

#[tonic::async_trait]
impl SearchService for SnomedServer {
    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();
        let query = req.query.to_lowercase();
        let limit = if req.limit > 0 { req.limit as usize } else { 100 };
        let active_only = req.active_only;

        let mut results: Vec<Concept> = Vec::new();

        // Simple linear search through all concepts
        // In production, you'd want a proper search index
        for concept in self.store.concepts() {
            if active_only && !concept.active {
                continue;
            }

            // Check if any description matches the query
            if let Some(descriptions) = self.store.get_descriptions(concept.id) {
                let matches = descriptions.iter().any(|d| {
                    d.term.to_lowercase().contains(&query)
                });

                if matches {
                    if let Some(proto_concept) = self.to_proto_concept(concept.id) {
                        results.push(proto_concept);
                        if results.len() >= limit {
                            break;
                        }
                    }
                }
            }
        }

        Ok(Response::new(SearchResponse { concepts: results }))
    }
}

#[tonic::async_trait]
impl EclService for SnomedServer {
    async fn execute_ecl(
        &self,
        request: Request<ExecuteEclRequest>,
    ) -> Result<Response<ExecuteEclResponse>, Status> {
        let req = request.into_inner();
        let ecl = req.ecl;
        let limit = if req.limit > 0 { req.limit as usize } else { usize::MAX };
        let include_details = req.include_details;

        let start = Instant::now();

        // Create executor and execute query
        let executor = EclExecutor::new(self.store.as_ref());

        let result = executor.execute(&ecl).map_err(|e| {
            Status::invalid_argument(format!("ECL execution error: {}", e))
        })?;

        let execution_time_ms = start.elapsed().as_millis() as u64;
        let total_count = result.count() as u64;
        let truncated = total_count > limit as u64;

        // Collect results
        let all_ids: Vec<u64> = result.iter().take(limit).copied().collect();

        let (concept_ids, concepts) = if include_details {
            let concepts: Vec<Concept> = all_ids
                .iter()
                .filter_map(|&id| self.to_proto_concept(id))
                .collect();
            (vec![], concepts)
        } else {
            (all_ids, vec![])
        };

        Ok(Response::new(ExecuteEclResponse {
            concept_ids,
            concepts,
            total_count,
            execution_time_ms,
            truncated,
        }))
    }

    async fn matches_ecl(
        &self,
        request: Request<MatchesEclRequest>,
    ) -> Result<Response<MatchesEclResponse>, Status> {
        let req = request.into_inner();
        let concept_id = req.concept_id;
        let ecl = req.ecl;

        let executor = EclExecutor::new(self.store.as_ref());

        let matches = executor.matches(concept_id, &ecl).map_err(|e| {
            Status::invalid_argument(format!("ECL match error: {}", e))
        })?;

        Ok(Response::new(MatchesEclResponse { matches }))
    }

    async fn get_descendants(
        &self,
        request: Request<GetDescendantsRequest>,
    ) -> Result<Response<GetDescendantsResponse>, Status> {
        let req = request.into_inner();
        let concept_id = req.concept_id;
        let limit = if req.limit > 0 { Some(req.limit as usize) } else { None };
        let include_self = req.include_self;

        let executor = EclExecutor::new(self.store.as_ref());

        let concept_ids: Vec<u64> = if let Some(max) = limit {
            executor.get_descendants_limited(concept_id, max)
        } else {
            executor.get_descendants(concept_id)
        };

        let mut result = concept_ids;
        if include_self && self.store.has_concept(concept_id) {
            result.insert(0, concept_id);
        }

        let total_count = result.len() as u64;

        Ok(Response::new(GetDescendantsResponse {
            concept_ids: result,
            total_count,
        }))
    }

    async fn get_ancestors(
        &self,
        request: Request<GetAncestorsRequest>,
    ) -> Result<Response<GetAncestorsResponse>, Status> {
        let req = request.into_inner();
        let concept_id = req.concept_id;
        let include_self = req.include_self;

        let executor = EclExecutor::new(self.store.as_ref());

        let mut concept_ids = executor.get_ancestors(concept_id);

        if include_self && self.store.has_concept(concept_id) {
            concept_ids.insert(0, concept_id);
        }

        let total_count = concept_ids.len() as u64;

        Ok(Response::new(GetAncestorsResponse {
            concept_ids,
            total_count,
        }))
    }

    async fn explain_ecl(
        &self,
        request: Request<ExplainEclRequest>,
    ) -> Result<Response<ExplainEclResponse>, Status> {
        let req = request.into_inner();
        let ecl = req.ecl;

        let start = Instant::now();

        let executor = EclExecutor::new(self.store.as_ref());

        // Get the query plan
        let plan = executor.explain(&ecl).map_err(|e| {
            Status::invalid_argument(format!("ECL parse error: {}", e))
        })?;

        let parse_time_us = start.elapsed().as_micros() as u64;

        // Convert QueryPlan steps to proto QueryPlanStep
        fn convert_steps(steps: &[snomed_loader::QueryStep]) -> Vec<QueryPlanStep> {
            steps.iter().map(|step| {
                QueryPlanStep {
                    operation: format!("{:?}", step.operation),
                    description: step.expression.clone(),
                    estimated_count: step.estimated_cardinality as u64,
                    children: vec![], // QueryStep doesn't have nested children
                }
            }).collect()
        }

        // Create a root plan step that contains all the steps
        let plan_step = QueryPlanStep {
            operation: "Root".to_string(),
            description: format!("ECL: {}", plan.ecl),
            estimated_count: plan.estimated_total as u64,
            children: convert_steps(&plan.steps),
        };

        Ok(Response::new(ExplainEclResponse {
            parsed_ecl: plan.ecl.clone(),
            plan: Some(plan_step),
            parse_time_us,
        }))
    }

    async fn is_subsumed_by(
        &self,
        request: Request<IsSubsumedByRequest>,
    ) -> Result<Response<IsSubsumedByResponse>, Status> {
        let req = request.into_inner();
        let concept_id = req.concept_id;
        let ancestor_id = req.ancestor_id;

        // Same concept
        if concept_id == ancestor_id {
            return Ok(Response::new(IsSubsumedByResponse {
                is_subsumed: true,
                distance: 0,
            }));
        }

        let executor = EclExecutor::new(self.store.as_ref());
        let is_subsumed = executor.is_subsumed_by(concept_id, ancestor_id);

        // Calculate distance if subsumed (BFS)
        let distance = if is_subsumed {
            let mut visited = std::collections::HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            queue.push_back((concept_id, 0i32));

            let mut found_distance = -1i32;
            while let Some((current, dist)) = queue.pop_front() {
                if current == ancestor_id {
                    found_distance = dist;
                    break;
                }
                if visited.insert(current) {
                    for parent in self.store.get_parents(current) {
                        queue.push_back((parent, dist + 1));
                    }
                }
            }
            found_distance
        } else {
            -1
        };

        Ok(Response::new(IsSubsumedByResponse {
            is_subsumed,
            distance,
        }))
    }

    async fn get_direct_parents(
        &self,
        request: Request<GetDirectParentsRequest>,
    ) -> Result<Response<GetDirectParentsResponse>, Status> {
        let concept_id = request.into_inner().concept_id;

        let executor = EclExecutor::new(self.store.as_ref());
        let parent_ids = executor.get_parents(concept_id);

        Ok(Response::new(GetDirectParentsResponse { parent_ids }))
    }

    async fn get_direct_children(
        &self,
        request: Request<GetDirectChildrenRequest>,
    ) -> Result<Response<GetDirectChildrenResponse>, Status> {
        let concept_id = request.into_inner().concept_id;

        let executor = EclExecutor::new(self.store.as_ref());
        let child_ids = executor.get_children(concept_id);

        Ok(Response::new(GetDirectChildrenResponse { child_ids }))
    }
}

#[tonic::async_trait]
impl RefsetService for SnomedServer {
    async fn get_refset_members(
        &self,
        request: Request<GetRefsetMembersRequest>,
    ) -> Result<Response<GetRefsetMembersResponse>, Status> {
        let req = request.into_inner();
        let refset_id = req.refset_id;
        let limit = if req.limit > 0 { req.limit as usize } else { usize::MAX };
        let offset = if req.offset > 0 { req.offset as usize } else { 0 };

        let all_members = self.store.get_refset_members(refset_id);
        let total_count = all_members.len() as u64;

        // Apply pagination
        let member_ids: Vec<u64> = all_members
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        Ok(Response::new(GetRefsetMembersResponse {
            member_ids,
            total_count,
        }))
    }

    async fn get_refsets_for_concept(
        &self,
        request: Request<GetRefsetsForConceptRequest>,
    ) -> Result<Response<GetRefsetsForConceptResponse>, Status> {
        let concept_id = request.into_inner().concept_id;

        let refset_ids = self.store.get_refsets_for_concept(concept_id);

        Ok(Response::new(GetRefsetsForConceptResponse { refset_ids }))
    }
}
