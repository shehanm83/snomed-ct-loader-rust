# Architecture Overview

## Project Structure

This project is a Rust workspace containing three crates for loading and working with SNOMED CT RF2 files.

```
snomed-ct-loader-rust/
├── Cargo.toml                          # Workspace root
├── docs/                               # Documentation
├── data/                               # SNOMED CT data files
│   └── SnomedCT_InternationalRF2_PRODUCTION_20251201T120000Z/
│
└── crates/
    ├── snomed-types/                   # Data structures (no I/O)
    │   └── src/
    │       ├── lib.rs
    │       ├── sctid.rs
    │       ├── enums.rs
    │       ├── well_known.rs
    │       ├── concept.rs
    │       ├── description.rs
    │       ├── relationship.rs
    │       └── mrcm.rs
    │
    ├── snomed-loader/                  # Parser & Loader (I/O)
    │   └── src/
    │       ├── lib.rs
    │       ├── types.rs
    │       ├── parser.rs
    │       ├── loader.rs
    │       ├── concept.rs
    │       ├── description.rs
    │       ├── relationship.rs
    │       ├── store.rs
    │       └── mrcm/
    │           ├── mod.rs
    │           ├── domain.rs
    │           ├── attribute_domain.rs
    │           ├── attribute_range.rs
    │           └── store.rs
    │
    └── snomed-service/                 # gRPC Service Layer
        ├── proto/
        │   └── snomed.proto
        ├── build.rs
        └── src/
            ├── lib.rs
            ├── main.rs
            ├── server.rs
            └── services/
                └── mod.rs
```

## Crate Responsibilities

### snomed-types

**Purpose**: Pure data structures with no I/O dependencies.

- Type definitions for SNOMED CT components
- Enums for coded values (DefinitionStatus, DescriptionType, CharacteristicType, etc.)
- Well-known SNOMED CT constants
- MRCM types (Cardinality, MrcmDomain, MrcmAttributeDomain, MrcmAttributeRange)
- Can be used without any file parsing

**Dependencies**: Only `serde` (optional)

### snomed-loader

**Purpose**: High-performance RF2 file parsing and loading.

- CSV parsing with streaming (`Rf2Parser`)
- Parallel processing with rayon (optional feature)
- File discovery (`discover_rf2_files`)
- In-memory storage (`SnomedStore`)
- Filter traits (`DescriptionFilter`, `RelationshipFilter`)
- MRCM parsing (`MrcmStore`)

**Dependencies**: `snomed-types`, `csv`, `thiserror`, `rayon` (optional)

### snomed-service

**Purpose**: gRPC-based API service for querying SNOMED CT data.

- Protocol buffer definitions for SNOMED CT APIs
- Concept lookup and search services
- Hierarchy navigation endpoints
- Built on tonic/prost for high-performance gRPC

**Dependencies**: `snomed-types`, `snomed-loader`, `tonic`, `prost`, `tokio`

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         RF2 Files                               │
│  sct2_Concept_*.txt  sct2_Description_*.txt  sct2_Relationship_*.txt
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      snomed-loader                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │   loader    │  │   parser    │  │   store     │              │
│  │ (discover)  │─▶│  (parse)    │─▶│  (store)    │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      snomed-types                               │
│  ┌────────────┐  ┌────────────────┐  ┌─────────────────┐        │
│  │ Rf2Concept │  │ Rf2Description │  │ Rf2Relationship │        │
│  └────────────┘  └────────────────┘  └─────────────────┘        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      snomed-service                             │
│  ┌─────────────────┐  ┌─────────────────┐                       │
│  │ ConceptService  │  │  SearchService  │                       │
│  │  (gRPC API)     │  │   (gRPC API)    │                       │
│  └─────────────────┘  └─────────────────┘                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │   Clients       │
                    │ (gRPC/REST)     │
                    └─────────────────┘
```

## Why Three Crates?

1. **Separation of concerns**: Types vs I/O vs Service
2. **Flexibility**: Use types without parsing, use loader without service
3. **Testing**: Pure types are easier to test
4. **Compilation**: Changes to service don't recompile parser
5. **Deployment**: Service can be deployed independently
