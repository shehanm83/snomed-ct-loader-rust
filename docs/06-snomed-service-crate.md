# snomed-service Crate

## Overview

The `snomed-service` crate provides a gRPC-based API service for querying SNOMED CT data. It builds on top of `snomed-loader` to provide network-accessible terminology services including ECL (Expression Constraint Language) query support.

> **Status**: Fully implemented with ConceptService, SearchService, and EclService

## Module Structure

```
snomed-service/
├── Cargo.toml
├── build.rs              # Protocol buffer compilation
├── proto/
│   └── snomed.proto      # gRPC service definitions
└── src/
    ├── lib.rs            # Library exports
    ├── main.rs           # Server binary entry point
    ├── server.rs         # SnomedServer implementation (all services)
    └── services/
        └── mod.rs        # Service implementations placeholder
```

## Services

### ConceptService

Operations for retrieving and navigating SNOMED CT concepts:

- **GetConcept** - Retrieve a concept by ID with FSN and descriptions
- **GetParents** - Get direct parent concepts (via IS_A relationships)
- **GetChildren** - Get direct child concepts (reverse IS_A)
- **IsDescendantOf** - Check if a concept is a descendant of another (subsumption)

### SearchService

Text search operations:

- **Search** - Search concepts by term (case-insensitive substring match)

### EclService

Expression Constraint Language (ECL) query operations:

- **ExecuteEcl** - Execute an ECL expression and return matching concept IDs/details
- **MatchesEcl** - Check if a specific concept matches an ECL expression
- **GetDescendants** - Get all descendants of a concept (optimized traversal)
- **GetAncestors** - Get all ancestors of a concept (optimized traversal)

## Protocol Buffer Definitions

The service is defined using Protocol Buffers (proto3):

```protobuf
syntax = "proto3";
package snomed;

// Core messages
message Concept {
  uint64 id = 1;
  uint32 effective_time = 2;
  bool active = 3;
  uint64 module_id = 4;
  uint64 definition_status_id = 5;
  string fsn = 6;
}

message Description {
  uint64 id = 1;
  uint64 concept_id = 2;
  string language_code = 3;
  uint64 type_id = 4;
  string term = 5;
  bool active = 6;
}

// ECL messages
message ExecuteEclRequest {
  string ecl = 1;              // ECL expression (e.g., "<< 73211009")
  int32 limit = 2;             // Max results (0 = unlimited)
  bool include_details = 3;    // Include concept details vs just IDs
}

message ExecuteEclResponse {
  repeated uint64 concept_ids = 1;  // IDs (if include_details is false)
  repeated Concept concepts = 2;     // Details (if include_details is true)
  uint64 total_count = 3;
  uint64 execution_time_ms = 4;
  bool truncated = 5;
}

message MatchesEclRequest {
  uint64 concept_id = 1;
  string ecl = 2;
}

message MatchesEclResponse {
  bool matches = 1;
}

// Services
service ConceptService {
  rpc GetConcept(GetConceptRequest) returns (GetConceptResponse);
  rpc GetParents(GetParentsRequest) returns (GetParentsResponse);
  rpc GetChildren(GetChildrenRequest) returns (GetChildrenResponse);
  rpc IsDescendantOf(IsDescendantOfRequest) returns (IsDescendantOfResponse);
}

service SearchService {
  rpc Search(SearchRequest) returns (SearchResponse);
}

service EclService {
  rpc ExecuteEcl(ExecuteEclRequest) returns (ExecuteEclResponse);
  rpc MatchesEcl(MatchesEclRequest) returns (MatchesEclResponse);
  rpc GetDescendants(GetDescendantsRequest) returns (GetDescendantsResponse);
  rpc GetAncestors(GetAncestorsRequest) returns (GetAncestorsResponse);
}
```

## Dependencies

```toml
[dependencies]
snomed-types = { workspace = true }
snomed-loader = { workspace = true }  # Includes ECL support
tonic = { workspace = true }
prost = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

[build-dependencies]
tonic-build = { workspace = true }
```

## Server Implementation

### SnomedServer

The `SnomedServer` struct holds the SNOMED CT data store and implements all three gRPC services:

```rust
use snomed_loader::SnomedStore;
use std::sync::Arc;

/// Main server holding the SNOMED CT data store.
#[derive(Clone)]
pub struct SnomedServer {
    store: Arc<SnomedStore>,
}

impl SnomedServer {
    pub fn new(store: SnomedStore) -> Self {
        Self { store: Arc::new(store) }
    }

    pub fn store(&self) -> &SnomedStore {
        &self.store
    }
}
```

### Running the Server

```rust
use snomed_loader::{discover_rf2_files, SnomedStore};
use snomed_service::proto::{
    concept_service_server::ConceptServiceServer,
    search_service_server::SearchServiceServer,
    ecl_service_server::EclServiceServer,
};
use snomed_service::SnomedServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load SNOMED CT data
    let files = discover_rf2_files("path/to/snomed/release")?;
    let mut store = SnomedStore::new();
    store.load_all(&files)?;

    // Create server
    let server = SnomedServer::new(store);

    // Start gRPC server with all services
    let addr = "[::1]:50051".parse()?;
    Server::builder()
        .add_service(ConceptServiceServer::new(server.clone()))
        .add_service(SearchServiceServer::new(server.clone()))
        .add_service(EclServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
```

### Environment Variables

- `SNOMED_DATA_PATH` - Path to SNOMED CT RF2 release directory
- `SNOMED_PORT` - Server port (default: 50051)

## Implementation Status

### Completed Features

- [x] Protocol buffer definitions for all services
- [x] Server skeleton with multi-service support
- [x] **ConceptService**
  - [x] GetConcept - Returns concept with FSN and descriptions
  - [x] GetParents - Returns direct IS_A parents
  - [x] GetChildren - Returns direct IS_A children
  - [x] IsDescendantOf - BFS-based subsumption check
- [x] **SearchService**
  - [x] Search - Case-insensitive term search with limit
- [x] **EclService**
  - [x] ExecuteEcl - Full ECL query execution
  - [x] MatchesEcl - Test if concept matches ECL
  - [x] GetDescendants - Optimized descendant traversal
  - [x] GetAncestors - Optimized ancestor traversal

### Planned Features

- [ ] Health checks (gRPC health protocol)
- [ ] Metrics (Prometheus)
- [ ] Configuration file support (TOML/YAML)
- [ ] Docker support
- [ ] REST gateway (tonic-web)
- [ ] Streaming responses for large result sets
- [ ] Pagination support
- [ ] MRCM validation endpoints

## Client Usage

### Rust Client

```rust
use snomed_service::proto::{
    concept_service_client::ConceptServiceClient,
    ecl_service_client::EclServiceClient,
    GetConceptRequest, ExecuteEclRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Concept queries
    let mut concept_client = ConceptServiceClient::connect("http://[::1]:50051").await?;

    let response = concept_client.get_concept(GetConceptRequest {
        id: 73211009,
    }).await?;

    if let Some(c) = response.into_inner().concept {
        println!("Concept: {} ({})", c.fsn, c.id);
    }

    // ECL queries
    let mut ecl_client = EclServiceClient::connect("http://[::1]:50051").await?;

    let response = ecl_client.execute_ecl(ExecuteEclRequest {
        ecl: "<< 73211009".to_string(),  // Descendants of Diabetes
        limit: 100,
        include_details: true,
    }).await?;

    let result = response.into_inner();
    println!("Found {} concepts in {}ms", result.total_count, result.execution_time_ms);

    Ok(())
}
```

### grpcurl (Command Line)

```bash
# Get a concept
grpcurl -plaintext -d '{"id": 73211009}' \
    localhost:50051 snomed.ConceptService/GetConcept

# Get parents of a concept
grpcurl -plaintext -d '{"id": 73211009}' \
    localhost:50051 snomed.ConceptService/GetParents

# Get children of a concept
grpcurl -plaintext -d '{"id": 64572001}' \
    localhost:50051 snomed.ConceptService/GetChildren

# Check subsumption (is 73211009 a descendant of 64572001?)
grpcurl -plaintext -d '{"concept_id": 73211009, "ancestor_id": 64572001}' \
    localhost:50051 snomed.ConceptService/IsDescendantOf

# Search for terms
grpcurl -plaintext -d '{"query": "diabetes", "limit": 10, "active_only": true}' \
    localhost:50051 snomed.SearchService/Search

# Execute ECL query - all descendants of Diabetes mellitus
grpcurl -plaintext -d '{"ecl": "<< 73211009", "limit": 100}' \
    localhost:50051 snomed.EclService/ExecuteEcl

# Check if concept matches ECL
grpcurl -plaintext -d '{"concept_id": 46635009, "ecl": "<< 73211009"}' \
    localhost:50051 snomed.EclService/MatchesEcl

# Get all descendants with details
grpcurl -plaintext -d '{"concept_id": 73211009, "limit": 50, "include_self": true}' \
    localhost:50051 snomed.EclService/GetDescendants

# Get all ancestors
grpcurl -plaintext -d '{"concept_id": 46635009, "include_self": false}' \
    localhost:50051 snomed.EclService/GetAncestors
```

## ECL Support

The service supports SNOMED CT Expression Constraint Language (ECL) via the `snomed-ecl-executor` crate. Supported operators:

- `*` - Wildcard (all concepts)
- `< id` - Descendants of
- `<< id` - Descendant or self
- `> id` - Ancestors of
- `>> id` - Ancestor or self
- `! id` - All except
- `expr AND expr` - Conjunction
- `expr OR expr` - Disjunction
- `expr MINUS expr` - Exclusion
- `{ attr = value }` - Attribute refinement
- `( expr )` - Grouping

Example ECL expressions:
- `<< 73211009` - Diabetes mellitus and all subtypes
- `< 404684003 AND < 123037004` - Clinical findings that are also body structures
- `<< 73211009 MINUS 46635009` - Diabetes excluding Type 1

## Architecture Notes

### Thread Safety

The `SnomedStore` is wrapped in `Arc<SnomedStore>` allowing safe concurrent access from multiple gRPC handlers. Since the store is read-only after initial loading, no additional synchronization is needed.

### Memory Considerations

A full SNOMED CT release requires approximately:
- Concepts: ~50MB
- Descriptions: ~300MB
- Relationships: ~400MB
- Total: ~750MB - 1GB

The service should be deployed with adequate memory (2GB+ recommended).

### Startup Time

Initial loading takes 5-15 seconds depending on hardware:
- With parallel feature: ~5s
- Without parallel: ~15s

Consider implementing health checks that distinguish between "starting" and "ready" states.
