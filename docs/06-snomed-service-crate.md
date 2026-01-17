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

// Core data types
message Concept {
    uint64 id = 1;
    uint32 effective_time = 2;
    bool active = 3;
    uint64 module_id = 4;
    uint64 definition_status_id = 5;
}

message Description {
    uint64 id = 1;
    uint32 effective_time = 2;
    bool active = 3;
    uint64 module_id = 4;
    uint64 concept_id = 5;
    string language_code = 6;
    uint64 type_id = 7;
    string term = 8;
    uint64 case_significance_id = 9;
}

message Relationship {
    uint64 id = 1;
    uint32 effective_time = 2;
    bool active = 3;
    uint64 module_id = 4;
    uint64 source_id = 5;
    uint64 destination_id = 6;
    uint32 relationship_group = 7;
    uint64 type_id = 8;
    uint64 characteristic_type_id = 9;
    uint64 modifier_id = 10;
}

// Request/Response messages
message GetConceptRequest {
    uint64 id = 1;
}

message GetConceptResponse {
    Concept concept = 1;
    repeated Description descriptions = 2;
    repeated Relationship relationships = 3;
}

message SearchRequest {
    string query = 1;
    int32 limit = 2;
    bool active_only = 3;
}

message SearchResponse {
    repeated SearchResult results = 1;
}

message SearchResult {
    uint64 concept_id = 1;
    string term = 2;
    string fsn = 3;
    bool active = 4;
}

// Services
service ConceptService {
    rpc GetConcept(GetConceptRequest) returns (GetConceptResponse);
    rpc GetParents(GetConceptRequest) returns (GetConceptsResponse);
    rpc GetChildren(GetConceptRequest) returns (GetConceptsResponse);
}

service SearchService {
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
- [ ] ConceptService implementation
  - [ ] GetConcept
  - [ ] GetParents
  - [ ] GetChildren
- [ ] SearchService implementation
  - [ ] Basic term search

### Phase 2: Enhanced Features
- [ ] Hierarchy navigation (ancestors, descendants)
- [ ] ECL (Expression Constraint Language) support
- [ ] Subsumption testing
- [ ] MRCM validation endpoints

### Phase 3: Production Ready
- [ ] Health checks
- [ ] Metrics (Prometheus)
- [ ] Configuration (TOML/YAML)
- [ ] Docker support
- [ ] REST gateway (grpc-gateway)

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

# Search for terms
grpcurl -plaintext -d '{"query": "diabetes", "limit": 10}' \
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
