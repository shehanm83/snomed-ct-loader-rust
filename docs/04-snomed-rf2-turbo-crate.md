# snomed-rf2-turbo Crate

## Overview

The `snomed-rf2-turbo` crate provides high-performance RF2 file parsing with optional parallel processing.

> **Status**: To be implemented in next phase

## Planned Module Structure

```
snomed-rf2-turbo/src/
├── lib.rs           # Public API exports
├── types.rs         # Parser-specific types (errors, configs)
├── parser.rs        # Generic RF2 parser
├── loader.rs        # File discovery
├── store.rs         # In-memory data store
├── concept.rs       # Concept parsing trait impl
├── description.rs   # Description parsing trait impl
├── relationship.rs  # Relationship parsing trait impl
└── mrcm/
    ├── mod.rs
    ├── domain.rs
    ├── attribute_domain.rs
    ├── attribute_range.rs
    └── store.rs
```

## types.rs

### Error Types

```rust
#[derive(Error, Debug)]
pub enum Rf2Error {
    #[error("IO error reading RF2 file: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Invalid SCTID format: {value}")]
    InvalidSctId { value: String },

    #[error("Missing required column: {column}")]
    MissingColumn { column: String },

    // ... more variants
}

pub type Rf2Result<T> = Result<T, Rf2Error>;
```

### Configuration

```rust
pub struct Rf2Config {
    pub active_only: bool,    // Filter to active records
    pub batch_size: usize,    // Processing batch size
}

pub struct DescriptionConfig {
    pub base: Rf2Config,
    pub language_codes: Vec<String>,  // ["en"]
    pub type_ids: Vec<u64>,           // FSN, Synonym IDs
}

pub struct RelationshipConfig {
    pub base: Rf2Config,
    pub type_ids: Vec<u64>,           // Filter by type
    pub characteristic_type_ids: Vec<u64>, // Stated/Inferred
}
```

### File Discovery

```rust
pub struct Rf2Files {
    pub concept_file: Option<PathBuf>,
    pub description_file: Option<PathBuf>,
    pub relationship_file: Option<PathBuf>,
    pub stated_relationship_file: Option<PathBuf>,
    pub mrcm_domain: Option<PathBuf>,
    pub mrcm_attribute_domain: Option<PathBuf>,
    pub mrcm_attribute_range: Option<PathBuf>,
    pub release_date: Option<String>,
}
```

## parser.rs

Generic streaming parser for RF2 files:

```rust
pub trait Rf2Record: Sized {
    fn from_record(record: &csv::StringRecord) -> Rf2Result<Self>;
}

pub struct Rf2Parser<R, T> {
    reader: csv::Reader<R>,
    config: Rf2Config,
    _phantom: PhantomData<T>,
}

impl<R: Read, T: Rf2Record> Iterator for Rf2Parser<R, T> {
    type Item = Rf2Result<T>;
    // ...
}
```

### Usage Pattern

```rust
// Parse concepts from file
let parser = Rf2Parser::<_, Rf2Concept>::from_path("concepts.txt", config)?;

for result in parser {
    match result {
        Ok(concept) => println!("{}: active={}", concept.id, concept.active),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## loader.rs

File discovery and batch loading:

```rust
/// Discover RF2 files in a release directory
pub fn discover_rf2_files(path: &Path) -> Rf2Result<Rf2Files> {
    // Searches for sct2_Concept_*, sct2_Description_*, etc.
}

/// Format byte size for display
pub fn format_bytes(bytes: u64) -> String {
    // "1.5 GB", "250 MB", etc.
}
```

## store.rs

In-memory storage with fast lookups:

```rust
pub struct SnomedStore {
    concepts: HashMap<SctId, Rf2Concept>,
    descriptions: HashMap<SctId, Vec<Rf2Description>>,
    relationships: HashMap<SctId, Vec<Rf2Relationship>>,
    parents: HashMap<SctId, Vec<SctId>>,    // IS_A cache
    children: HashMap<SctId, Vec<SctId>>,   // Reverse IS_A
}

impl SnomedStore {
    pub fn new() -> Self;
    pub fn load_all(&mut self, files: &Rf2Files) -> Rf2Result<()>;

    // Lookups
    pub fn get_concept(&self, id: SctId) -> Option<&Rf2Concept>;
    pub fn get_descriptions(&self, concept_id: SctId) -> &[Rf2Description];
    pub fn get_fsn(&self, concept_id: SctId) -> Option<&str>;

    // Hierarchy navigation
    pub fn get_parents(&self, id: SctId) -> &[SctId];
    pub fn get_children(&self, id: SctId) -> &[SctId];
    pub fn get_ancestors(&self, id: SctId) -> HashSet<SctId>;
    pub fn is_descendant_of(&self, id: SctId, ancestor: SctId) -> bool;
}
```

## Feature Flags

```toml
[features]
default = ["parallel"]
parallel = ["rayon"]      # Parallel parsing
progress = ["indicatif"]  # Progress bars
```

## Performance

With parallel parsing enabled:

| File | Records | Sequential | Parallel (8 cores) |
|------|---------|------------|-------------------|
| Concepts | ~500K | ~2s | ~0.5s |
| Descriptions | ~1.5M | ~8s | ~1.5s |
| Relationships | ~3M | ~15s | ~3s |

## Planned Usage

```rust
use snomed_rf2_turbo::{discover_rf2_files, SnomedStore};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Discover files
    let files = discover_rf2_files("path/to/snomed/release")?;

    // Load into store
    let mut store = SnomedStore::new();
    store.load_all(&files)?;

    // Query
    if let Some(concept) = store.get_concept(73211009) {
        println!("Found: {} (active={})", concept.id, concept.active);

        if let Some(fsn) = store.get_fsn(concept.id) {
            println!("FSN: {}", fsn);
        }

        let parents = store.get_parents(concept.id);
        println!("Parents: {:?}", parents);
    }

    Ok(())
}
```
