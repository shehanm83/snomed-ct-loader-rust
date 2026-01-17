# snomed-types Crate

## Overview

The `snomed-types` crate provides pure Rust type definitions for SNOMED CT data structures. It has no I/O operations and minimal dependencies.

## Module Structure

```
snomed-types/src/
├── lib.rs           # Public API exports
├── sctid.rs         # SNOMED CT ID type
├── enums.rs         # Enumeration types
├── well_known.rs    # Common SCTID constants
├── concept.rs       # Rf2Concept struct
├── description.rs   # Rf2Description struct
├── relationship.rs  # Rf2Relationship struct
└── mrcm.rs          # MRCM constraint types
```

## sctid.rs

The simplest module - just a type alias:

```rust
pub type SctId = u64;
```

**Design decision**: Using a type alias instead of a newtype wrapper because:
- Zero overhead
- Easy CSV parsing (just parse as u64)
- Trade-off: Less type safety, but more ergonomic

## enums.rs

Type-safe enums that map SNOMED coded values to Rust types.

### Pattern Used

Each enum follows this pattern:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefinitionStatus {
    Primitive,
    FullyDefined,
}

impl DefinitionStatus {
    // Associated constants for SCTID values
    pub const PRIMITIVE_ID: SctId = 900000000000074008;
    pub const FULLY_DEFINED_ID: SctId = 900000000000073002;

    // Convert from SCTID
    pub fn from_id(id: SctId) -> Option<Self> { ... }

    // Convert to SCTID
    pub fn to_id(self) -> SctId { ... }
}
```

### Enums Defined

| Enum | Purpose | Values |
|------|---------|--------|
| `DefinitionStatus` | Concept definition type | Primitive, FullyDefined |
| `DescriptionType` | Description category | Fsn, Synonym, Definition |
| `CaseSignificance` | Case handling | CaseInsensitive, EntireTermCaseSensitive, InitialCharacterCaseSensitive |
| `CharacteristicType` | Relationship source | Stated, Inferred, Additional |
| `ModifierType` | Logic modifier | Existential, Universal |

## well_known.rs

Constants for frequently-used SNOMED CT identifiers:

```rust
// Root
pub const SNOMED_CT_ROOT: SctId = 138875005;

// Top-level hierarchies
pub const CLINICAL_FINDING: SctId = 404684003;
pub const PROCEDURE: SctId = 71388002;
pub const BODY_STRUCTURE: SctId = 123037004;

// Relationship types
pub const IS_A: SctId = 116680003;
pub const FINDING_SITE: SctId = 363698007;

// Modules
pub const SNOMED_CT_CORE_MODULE: SctId = 900000000000207008;
```

### Categories

1. **Root Concepts** - The top of the hierarchy
2. **Top-Level Hierarchies** - Main branches (Clinical finding, Procedure, etc.)
3. **Relationship Types** - IS_A, Finding site, etc.
4. **Modules** - Content ownership
5. **Qualifiers** - Mild, Moderate, Severe, Left, Right
6. **MRCM Reference Sets** - Validation rule identifiers

## concept.rs

The `Rf2Concept` struct maps directly to RF2 concept file rows:

```rust
pub struct Rf2Concept {
    pub id: SctId,                    // Unique identifier
    pub effective_time: u32,          // YYYYMMDD as integer
    pub active: bool,                 // Is this concept current?
    pub module_id: SctId,             // Owning module
    pub definition_status_id: SctId,  // Primitive or FullyDefined
}
```

### Helper Methods

```rust
impl Rf2Concept {
    pub fn is_primitive(&self) -> bool { ... }
    pub fn is_fully_defined(&self) -> bool { ... }
    pub fn definition_status(&self) -> Option<DefinitionStatus> { ... }
}
```

## description.rs

The `Rf2Description` struct for human-readable terms:

```rust
pub struct Rf2Description {
    pub id: SctId,
    pub effective_time: u32,
    pub active: bool,
    pub module_id: SctId,
    pub concept_id: SctId,           // Parent concept
    pub language_code: String,        // "en", "es", etc.
    pub type_id: SctId,              // FSN, Synonym, Definition
    pub term: String,                 // The actual text
    pub case_significance_id: SctId,
}
```

### Helper Methods

```rust
impl Rf2Description {
    pub fn is_fsn(&self) -> bool { ... }
    pub fn is_synonym(&self) -> bool { ... }
    pub fn is_definition(&self) -> bool { ... }
    pub fn description_type(&self) -> Option<DescriptionType> { ... }
}
```

## relationship.rs

The `Rf2Relationship` struct connects concepts:

```rust
pub struct Rf2Relationship {
    pub id: SctId,
    pub effective_time: u32,
    pub active: bool,
    pub module_id: SctId,
    pub source_id: SctId,            // From concept
    pub destination_id: SctId,       // To concept
    pub relationship_group: u16,     // Role group (0 = ungrouped)
    pub type_id: SctId,              // IS_A, Finding site, etc.
    pub characteristic_type_id: SctId, // Stated or Inferred
    pub modifier_id: SctId,          // Existential or Universal
}
```

### Stated vs Inferred

- **Stated**: Authored by human editors
- **Inferred**: Computed by Description Logic classifier

### Role Groups

Role groups bundle related attributes:

```
Pneumonia:
  Group 0 (ungrouped):
    IS_A → Lung disease
  Group 1:
    Finding site → Lung structure
    Causative agent → Bacteria
  Group 2:
    Finding site → Lung structure
    Causative agent → Virus
```

## mrcm.rs

MRCM (Machine Readable Concept Model) types for validation:

### Cardinality

```rust
pub struct Cardinality {
    pub min: u32,
    pub max: Option<u32>,  // None = unbounded (*)
}

// Examples:
// "0..*" = zero or more
// "0..1" = optional
// "1..1" = required exactly one
// "1..*" = one or more
```

### MrcmDomain

Defines semantic domains (e.g., "Clinical finding domain")

### MrcmAttributeDomain

Defines which attributes are valid in which domains with cardinality constraints

### MrcmAttributeRange

Defines valid value ranges for attributes using ECL expressions

## Feature Flags

```toml
[features]
default = ["serde"]
serde = ["dep:serde"]  # Optional serialization
```

Disable serde for zero-dependency usage:

```toml
snomed-types = { version = "0.1", default-features = false }
```
