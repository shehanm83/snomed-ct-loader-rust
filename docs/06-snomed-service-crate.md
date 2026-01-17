# snomed-service Crate

## Overview

The `snomed-service` crate provides a gRPC-based API service for querying SNOMED CT data. It builds on top of `snomed-loader` to provide network-accessible terminology services.

> **Status**: Skeleton implemented, service logic pending

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
    ├── server.rs         # SnomedServer implementation
    └── services/
        └── mod.rs        # Service implementations
```

## Protocol Buffer Definitions

The service is defined using Protocol Buffers (proto3):

```protobuf
syntax = "proto3";

package snomed;

// SNOMED CT Concept
message Concept {
  uint64 id = 1;
  uint32 effective_time = 2;
  bool active = 3;
  uint64 module_id = 4;
  uint64 definition_status_id = 5;
  string fsn = 6;  // Fully Specified Name
}

// SNOMED CT Description
message Description {
  uint64 id = 1;
  uint64 concept_id = 2;
  string language_code = 3;
  uint64 type_id = 4;
  string term = 5;
  bool active = 6;
}

// SNOMED CT Relationship
message Relationship {
  uint64 id = 1;
  uint64 source_id = 2;
  uint64 destination_id = 3;
  uint64 type_id = 4;
  uint32 relationship_group = 5;
  bool active = 6;
}

// Request/Response messages
message GetConceptRequest {
  uint64 id = 1;
}

message GetConceptResponse {
  Concept concept = 1;
  repeated Description descriptions = 2;
}

message GetParentsRequest {
  uint64 id = 1;
}

message GetParentsResponse {
  repeated Concept parents = 1;
}

message GetChildrenRequest {
  uint64 id = 1;
}

message GetChildrenResponse {
  repeated Concept children = 1;
}

message SearchRequest {
  string query = 1;
  int32 limit = 2;
  bool active_only = 3;
}

message SearchResponse {
  repeated Concept concepts = 1;
}

message IsDescendantOfRequest {
  uint64 concept_id = 1;
  uint64 ancestor_id = 2;
}

message IsDescendantOfResponse {
  bool is_descendant = 1;
}

// Service definitions
service ConceptService {
  // Get a concept by ID with descriptions
  rpc GetConcept(GetConceptRequest) returns (GetConceptResponse);

  // Get parent concepts (via IS_A relationships)
  rpc GetParents(GetParentsRequest) returns (GetParentsResponse);

  // Get child concepts (reverse IS_A)
  rpc GetChildren(GetChildrenRequest) returns (GetChildrenResponse);

  // Check if concept is descendant of another (subsumption)
  rpc IsDescendantOf(IsDescendantOfRequest) returns (IsDescendantOfResponse);
}

service SearchService {
  // Search concepts by term (case-insensitive substring match)
  rpc Search(SearchRequest) returns (SearchResponse);
}
```

## Dependencies

```toml
[dependencies]
snomed-types = { workspace = true }
snomed-loader = { workspace = true }
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

```rust
use snomed_loader::SnomedStore;
use std::sync::Arc;

/// Main server holding the SNOMED CT data store.
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

    /// Returns a reference to the underlying store.
    pub fn store(&self) -> &SnomedStore {
        &self.store
    }
}
```

### Running the Server

```rust
use snomed_loader::{discover_rf2_files, SnomedStore};
use snomed_service::SnomedServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load SNOMED CT data
    let files = discover_rf2_files("path/to/snomed/release")?;
    let mut store = SnomedStore::new();
    store.load_all(&files)?;

    // Create server
    let server = SnomedServer::new(store);

    // Start gRPC server
    let addr = "[::1]:50051".parse()?;
    tracing::info!("Starting SNOMED CT server on {}", addr);

    Server::builder()
        .add_service(ConceptServiceServer::new(server.clone()))
        .add_service(SearchServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
```

## Planned Features

### Phase 1: Core Services
- [x] Protocol buffer definitions
- [x] Server skeleton
- [x] ConceptService implementation
  - [x] GetConcept - Returns concept with FSN and descriptions
  - [x] GetParents - Returns direct IS_A parents
  - [x] GetChildren - Returns direct IS_A children
  - [x] IsDescendantOf - BFS-based subsumption check
- [x] SearchService implementation
  - [x] Basic term search (case-insensitive substring match)

### Phase 2: Enhanced Features
- [x] Hierarchy navigation (ancestors via IsDescendantOf)
- [ ] Full ancestors/descendants traversal endpoints
- [ ] ECL (Expression Constraint Language) support
  - [ ] Integration with snomed-ecl-executor
  - [ ] ExecuteEcl RPC endpoint
- [ ] MRCM validation endpoints
  - [ ] ValidateExpression RPC
  - [ ] GetAllowedAttributes RPC

### Phase 3: Production Ready
- [ ] Health checks (gRPC health protocol)
- [ ] Metrics (Prometheus)
- [ ] Configuration (TOML/YAML)
- [ ] Docker support
- [ ] REST gateway (grpc-gateway or tonic-web)
- [ ] Streaming responses for large result sets
- [ ] Pagination support

## Client Usage

### Rust Client

```rust
use snomed_service::proto::{
    concept_service_client::ConceptServiceClient,
    GetConceptRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ConceptServiceClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(GetConceptRequest {
        id: 73211009, // Diabetes mellitus
    });

    let response = client.get_concept(request).await?;
    let concept = response.into_inner();

    if let Some(c) = concept.concept {
        println!("Concept ID: {}", c.id);
        println!("Active: {}", c.active);
    }

    for desc in &concept.descriptions {
        println!("Description: {}", desc.term);
    }

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

# Check if concept is descendant of another (subsumption)
grpcurl -plaintext -d '{"concept_id": 73211009, "ancestor_id": 64572001}' \
    localhost:50051 snomed.ConceptService/IsDescendantOf

# Search for terms
grpcurl -plaintext -d '{"query": "diabetes", "limit": 10, "active_only": true}' \
    localhost:50051 snomed.SearchService/Search
```

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
