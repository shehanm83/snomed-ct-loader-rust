# Architecture Overview

## Project Structure

This project is a Rust workspace containing two crates for loading and working with SNOMED CT RF2 files.

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
    └── snomed-loader/                  # Parser & Loader (I/O)
        └── src/
            ├── lib.rs
            ├── types.rs
            ├── parser.rs
            ├── loader.rs
            ├── store.rs
            └── mrcm/
```

## Crate Responsibilities

### snomed-types

**Purpose**: Pure data structures with no I/O dependencies.

- Type definitions for SNOMED CT components
- Enums for coded values
- Well-known SNOMED CT constants
- Can be used without any file parsing

**Dependencies**: Only `serde` (optional)

### snomed-loader

**Purpose**: High-performance RF2 file parsing and loading.

- CSV parsing with streaming
- Parallel processing with rayon
- File discovery
- In-memory storage

**Dependencies**: `snomed-types`, `csv`, `thiserror`, `rayon`

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
```

## Why Two Crates?

1. **Separation of concerns**: Types vs I/O
2. **Flexibility**: Use types without parsing (e.g., receiving SNOMED data via API)
3. **Testing**: Pure types are easier to test
4. **Compilation**: Changes to parser don't recompile type definitions
