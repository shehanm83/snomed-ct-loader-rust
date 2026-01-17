# Documentation Index

## Overview

This documentation explains the architecture and implementation of the SNOMED CT RF2 loader in Rust.

## Contents

| Document | Description |
|----------|-------------|
| [01-architecture-overview.md](01-architecture-overview.md) | Project structure and crate responsibilities |
| [02-snomed-ct-basics.md](02-snomed-ct-basics.md) | Introduction to SNOMED CT concepts |
| [03-snomed-types-crate.md](03-snomed-types-crate.md) | Detailed explanation of the types crate |
| [04-snomed-loader-crate.md](04-snomed-loader-crate.md) | Parser crate design (to be implemented) |
| [05-mrcm-explained.md](05-mrcm-explained.md) | Machine Readable Concept Model deep dive |

## Quick Start

### Reading Order

If you're new to SNOMED CT:
1. Start with [02-snomed-ct-basics.md](02-snomed-ct-basics.md)
2. Then read [01-architecture-overview.md](01-architecture-overview.md)
3. Explore [03-snomed-types-crate.md](03-snomed-types-crate.md)

If you're familiar with SNOMED CT:
1. Start with [01-architecture-overview.md](01-architecture-overview.md)
2. Dive into [03-snomed-types-crate.md](03-snomed-types-crate.md)
3. Check [05-mrcm-explained.md](05-mrcm-explained.md) for validation rules

## Key Concepts

### SNOMED CT Components

```
Concept (73211009 = "Diabetes mellitus")
    │
    ├── Descriptions (text)
    │   ├── FSN: "Diabetes mellitus (disorder)"
    │   └── Synonym: "Diabetes", "DM"
    │
    └── Relationships (connections)
        └── IS_A → Disorder of endocrine system
```

### Crate Structure

```
snomed-ct-loader-rust/
├── snomed-types/        # Pure data structures
│   └── Rf2Concept, Rf2Description, Rf2Relationship, MRCM types
│
└── snomed-loader/       # Parser & I/O
    └── File loading, CSV parsing, parallel processing
```

### Type Hierarchy

```
SctId (u64)
    │
    ├── Rf2Concept
    │   ├── id: SctId
    │   ├── active: bool
    │   └── definition_status_id: SctId → DefinitionStatus enum
    │
    ├── Rf2Description
    │   ├── id: SctId
    │   ├── concept_id: SctId
    │   ├── term: String
    │   └── type_id: SctId → DescriptionType enum
    │
    └── Rf2Relationship
        ├── source_id: SctId
        ├── destination_id: SctId
        ├── type_id: SctId
        └── characteristic_type_id: SctId → CharacteristicType enum
```

## Implementation Progress

- [x] Project structure setup
- [x] Cargo workspace configuration
- [x] `snomed-types` crate
  - [x] SctId type
  - [x] Enums (DefinitionStatus, DescriptionType, etc.)
  - [x] Well-known constants
  - [x] Rf2Concept
  - [x] Rf2Description
  - [x] Rf2Relationship
  - [x] MRCM types
- [ ] `snomed-loader` crate
  - [ ] Error types and configuration
  - [ ] Generic RF2 parser
  - [ ] File discovery
  - [ ] In-memory store
  - [ ] MRCM parsing

## References

- [SNOMED International](https://www.snomed.org/)
- [SNOMED CT Technical Implementation Guide](https://confluence.ihtsdotools.org/display/DOCTIG)
- [RF2 Guide](https://confluence.ihtsdotools.org/display/DOCRELFMT)
- [MRCM Specification](https://confluence.ihtsdotools.org/display/DOCMRCM)
