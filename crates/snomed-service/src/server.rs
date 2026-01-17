//! gRPC server implementation.

use std::sync::Arc;
use snomed_loader::SnomedStore;
use tonic::{Request, Response, Status};

use crate::proto::{
    concept_service_server::ConceptService,
    search_service_server::SearchService,
    Concept, Description, GetConceptRequest, GetConceptResponse,
    GetParentsRequest, GetParentsResponse, GetChildrenRequest, GetChildrenResponse,
    IsDescendantOfRequest, IsDescendantOfResponse,
    SearchRequest, SearchResponse,
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

        // Simple BFS to check ancestry
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![concept_id];

        while let Some(current) = queue.pop() {
            if current == ancestor_id {
                return Ok(Response::new(IsDescendantOfResponse { is_descendant: true }));
            }

            if visited.insert(current) {
                let parents = self.store.get_parents(current);
                queue.extend(parents);
            }
        }

        Ok(Response::new(IsDescendantOfResponse { is_descendant: false }))
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
