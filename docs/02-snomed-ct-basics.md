# SNOMED CT Basics

## What is SNOMED CT?

SNOMED CT (Systematized Nomenclature of Medicine - Clinical Terms) is a comprehensive clinical terminology used in healthcare for encoding clinical information.

## Core Components

SNOMED CT has three core component types:

### 1. Concepts

A **Concept** is a clinical meaning. Every concept has a unique identifier (SCTID).

Examples:
- `73211009` = Diabetes mellitus
- `404684003` = Clinical finding
- `123037004` = Body structure

### 2. Descriptions

A **Description** is human-readable text for a concept. Each concept has multiple descriptions:

- **FSN** (Fully Specified Name): Unambiguous name with semantic tag
  - Example: "Diabetes mellitus (disorder)"
- **Synonym**: Alternative acceptable term
  - Example: "Diabetes", "DM"

### 3. Relationships

A **Relationship** connects two concepts:

```
Source Concept ──[type]──▶ Destination Concept

Example:
Diabetes mellitus ──[IS_A]──▶ Disorder of endocrine system
     73211009                      362969004
```

## The IS_A Hierarchy

The `IS_A` relationship (SCTID: `116680003`) creates a taxonomy:

```
SNOMED CT Root (138875005)
├── Clinical finding (404684003)
│   ├── Disease (64572001)
│   │   ├── Disorder of endocrine system (362969004)
│   │   │   └── Diabetes mellitus (73211009)
│   │   │       ├── Type 1 diabetes mellitus (46635009)
│   │   │       └── Type 2 diabetes mellitus (44054006)
│   ...
├── Procedure (71388002)
├── Body structure (123037004)
├── Organism (410607006)
└── ...
```

## SCTID Structure

SNOMED CT Identifiers (SCTIDs) are 64-bit integers with embedded metadata:

```
Example: 73211009
         ├───────┤
         │       └─ Check digit
         └─ Namespace + Item ID
```

In this codebase, we use `u64` for simplicity:

```rust
pub type SctId = u64;
```

## RF2 Format

RF2 (Release Format 2) is the distribution format for SNOMED CT. Files are tab-delimited text with headers.

### Concept File (sct2_Concept_*.txt)

| Column | Description |
|--------|-------------|
| id | Concept SCTID |
| effectiveTime | Release date (YYYYMMDD) |
| active | 1 = active, 0 = inactive |
| moduleId | Module SCTID |
| definitionStatusId | Primitive or Fully Defined |

### Description File (sct2_Description_*.txt)

| Column | Description |
|--------|-------------|
| id | Description SCTID |
| effectiveTime | Release date |
| active | 1 = active, 0 = inactive |
| moduleId | Module SCTID |
| conceptId | Parent concept SCTID |
| languageCode | ISO code (e.g., "en") |
| typeId | FSN, Synonym, or Definition |
| term | The actual text |
| caseSignificanceId | Case handling rules |

### Relationship File (sct2_Relationship_*.txt)

| Column | Description |
|--------|-------------|
| id | Relationship SCTID |
| effectiveTime | Release date |
| active | 1 = active, 0 = inactive |
| moduleId | Module SCTID |
| sourceId | Source concept SCTID |
| destinationId | Target concept SCTID |
| relationshipGroup | Role group (0 = ungrouped) |
| typeId | Relationship type (e.g., IS_A) |
| characteristicTypeId | Stated or Inferred |
| modifierId | Existential or Universal |
