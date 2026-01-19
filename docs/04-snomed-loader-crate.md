# snomed-loader Crate

## Overview

The `snomed-loader` crate provides high-performance RF2 file parsing with optional parallel processing.

## Module Structure

```
snomed-loader/src/
├── lib.rs              # Public API exports
├── types.rs            # Parser-specific types (errors, configs)
├── parser.rs           # Generic RF2 parser with Rf2Record trait
├── loader.rs           # File discovery utilities
├── concept.rs          # Rf2Record impl for Rf2Concept
├── description.rs      # Rf2Record impl + DescriptionFilter trait
├── relationship.rs     # Rf2Record impl + RelationshipFilter trait
├── store.rs            # In-memory data store with parallel loading
├── ecl.rs              # EclQueryable trait implementation for SnomedStore
└── mrcm/
    ├── mod.rs          # MRCM module exports
    ├── domain.rs       # MrcmDomain parser
    ├── attribute_domain.rs  # MrcmAttributeDomain parser
    ├── attribute_range.rs   # MrcmAttributeRange parser
    └── store.rs        # MrcmStore for MRCM data
```

## types.rs

### Error Types

```rust
#[derive(Error, Debug)]
pub enum Rf2Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Invalid SCTID format: {value}")]
    InvalidSctId { value: String },

    #[error("Invalid boolean value: {value}")]
    InvalidBoolean { value: String },

    #[error("Invalid date format: {value}")]
    InvalidDate { value: String },

    #[error("Invalid integer: {value}")]
    InvalidInteger { value: String },

    #[error("Missing required column: {column}")]
    MissingColumn { column: String },

    #[error("Invalid header: expected {expected} columns, found {found}")]
    InvalidHeader { expected: usize, found: usize },

    #[error("Unexpected column at position {position}: expected {expected}, found {found}")]
    UnexpectedColumn { position: usize, expected: String, found: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: String },

    #[error("Required file missing: {file_type} in {directory}")]
    RequiredFileMissing { file_type: String, directory: String },
}

pub type Rf2Result<T> = Result<T, Rf2Error>;
```

### Configuration

```rust
/// Base configuration for RF2 parsing.
#[derive(Debug, Clone, Default)]
pub struct Rf2Config {
    pub active_only: bool,    // Filter to active records only (default: true)
    pub batch_size: usize,    // Processing batch size (default: 10000)
}

/// Configuration for parsing descriptions.
#[derive(Debug, Clone, Default)]
pub struct DescriptionConfig {
    pub base: Rf2Config,
    pub language_codes: Vec<String>,  // Filter by language (e.g., ["en"])
    pub type_ids: Vec<SctId>,         // Filter by type (FSN, Synonym)
}

impl DescriptionConfig {
    pub fn english_terms() -> Self;   // English FSN + Synonyms
    pub fn fsn_only() -> Self;        // Only Fully Specified Names
}

/// Configuration for parsing relationships.
#[derive(Debug, Clone, Default)]
pub struct RelationshipConfig {
    pub base: Rf2Config,
    pub type_ids: Vec<SctId>,                // Filter by relationship type
    pub characteristic_type_ids: Vec<SctId>, // Stated vs Inferred
}

impl RelationshipConfig {
    pub fn inferred_only() -> Self;   // Only inferred relationships
    pub fn stated_only() -> Self;     // Only stated relationships
    pub fn is_a_only() -> Self;       // Only IS_A relationships
}
```

### File Discovery

```rust
/// Discovered RF2 file paths.
#[derive(Debug, Default)]
pub struct Rf2Files {
    pub concept_file: Option<PathBuf>,
    pub description_file: Option<PathBuf>,
    pub relationship_file: Option<PathBuf>,
    pub stated_relationship_file: Option<PathBuf>,
    pub text_definition_file: Option<PathBuf>,
    pub mrcm_domain: Option<PathBuf>,
    pub mrcm_attribute_domain: Option<PathBuf>,
    pub mrcm_attribute_range: Option<PathBuf>,
    pub release_date: Option<String>,
}

impl Rf2Files {
    pub fn has_required_files(&self) -> bool;
    pub fn missing_files(&self) -> Vec<&str>;
}
```

## parser.rs

Generic streaming parser for RF2 files:

```rust
/// Trait for types that can be parsed from RF2 records.
pub trait Rf2Record: Sized {
    /// Expected column names for validation.
    const EXPECTED_COLUMNS: &'static [&'static str];

    /// Parse a record from a CSV StringRecord.
    fn from_record(record: &StringRecord) -> Rf2Result<Self>;

    /// Returns true if this record passes the filter.
    fn passes_filter(&self, config: &Rf2Config) -> bool;
}

/// Streaming parser for RF2 files.
pub struct Rf2Parser<R: Read, T: Rf2Record> {
    reader: csv::Reader<R>,
    config: Rf2Config,
    records_read: usize,
}

impl<T: Rf2Record> Rf2Parser<BufReader<File>, T> {
    /// Creates a parser from a file path.
    pub fn from_path<P: AsRef<Path>>(path: P, config: Rf2Config) -> Rf2Result<Self>;

    /// Counts total lines in file (for progress reporting).
    pub fn count_lines<P: AsRef<Path>>(path: P) -> Rf2Result<usize>;
}

impl<R: Read, T: Rf2Record> Rf2Parser<R, T> {
    /// Creates a parser from any reader.
    pub fn from_reader(reader: R, config: Rf2Config) -> Rf2Result<Self>;

    /// Parses all records into a Vec.
    pub fn parse_all(self) -> Rf2Result<Vec<T>>;

    /// Parses records in batches with a callback.
    pub fn parse_batched<F>(self, callback: F) -> Rf2Result<usize>
    where
        F: FnMut(Vec<T>) -> Rf2Result<()>;
}

impl<R: Read, T: Rf2Record> Iterator for Rf2Parser<R, T> {
    type Item = Rf2Result<T>;
}
```

### Usage Pattern

```rust
use snomed_loader::{Rf2Parser, Rf2Config};
use snomed_types::Rf2Concept;

// Parse concepts from file
let config = Rf2Config::default();
let parser = Rf2Parser::<_, Rf2Concept>::from_path("concepts.txt", config)?;

for result in parser {
    match result {
        Ok(concept) => println!("{}: active={}", concept.id, concept.active),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## loader.rs

File discovery and utilities:

```rust
/// Discover RF2 files in a release directory.
/// Searches for Snapshot/Terminology and Refset/Metadata directories.
pub fn discover_rf2_files<P: AsRef<Path>>(path: P) -> Rf2Result<Rf2Files>;

/// Format byte size for display (e.g., "1.50 GB", "250.00 MB").
pub fn format_bytes(bytes: usize) -> String;
```

## store.rs

In-memory storage with fast lookups and parallel loading:

```rust
/// In-memory store for SNOMED CT data.
#[derive(Debug, Default)]
pub struct SnomedStore {
    concepts: HashMap<SctId, Rf2Concept>,
    descriptions_by_concept: HashMap<SctId, Vec<Rf2Description>>,
    relationships_by_source: HashMap<SctId, Vec<Rf2Relationship>>,
    relationships_by_destination: HashMap<SctId, Vec<Rf2Relationship>>,
    mrcm: Option<MrcmStore>,
}

impl SnomedStore {
    pub fn new() -> Self;
    pub fn with_capacity(concepts: usize, descriptions: usize, relationships: usize) -> Self;

    // Sequential loading
    pub fn load_concepts<P: AsRef<Path>>(&mut self, path: P, config: Rf2Config) -> Rf2Result<usize>;
    pub fn load_descriptions<P: AsRef<Path>>(&mut self, path: P, config: DescriptionConfig) -> Rf2Result<usize>;
    pub fn load_relationships<P: AsRef<Path>>(&mut self, path: P, config: RelationshipConfig) -> Rf2Result<usize>;
    pub fn load_all(&mut self, files: &Rf2Files) -> Rf2Result<()>;

    // Parallel loading (requires "parallel" feature)
    #[cfg(feature = "parallel")]
    pub fn load_concepts_parallel<P: AsRef<Path>>(&mut self, path: P, config: Rf2Config) -> Rf2Result<usize>;
    #[cfg(feature = "parallel")]
    pub fn load_descriptions_parallel<P: AsRef<Path>>(&mut self, path: P, config: DescriptionConfig) -> Rf2Result<usize>;
    #[cfg(feature = "parallel")]
    pub fn load_relationships_parallel<P: AsRef<Path>>(&mut self, path: P, config: RelationshipConfig) -> Rf2Result<usize>;
    #[cfg(feature = "parallel")]
    pub fn load_all_parallel(&mut self, files: &Rf2Files) -> Rf2Result<(usize, usize, usize)>;

    // MRCM loading
    pub fn load_mrcm(&mut self, files: &Rf2Files) -> Rf2Result<()>;
    pub fn get_mrcm(&self) -> Option<&MrcmStore>;
    pub fn has_mrcm(&self) -> bool;

    // Bulk inserts
    pub fn insert_concepts(&mut self, concepts: impl IntoIterator<Item = Rf2Concept>);
    pub fn insert_descriptions(&mut self, descriptions: impl IntoIterator<Item = Rf2Description>);
    pub fn insert_relationships(&mut self, relationships: impl IntoIterator<Item = Rf2Relationship>);

    // Query methods
    pub fn get_concept(&self, id: SctId) -> Option<&Rf2Concept>;
    pub fn has_concept(&self, id: SctId) -> bool;
    pub fn get_descriptions(&self, concept_id: SctId) -> Option<&Vec<Rf2Description>>;
    pub fn get_fsn(&self, concept_id: SctId) -> Option<&Rf2Description>;
    pub fn get_preferred_term(&self, concept_id: SctId) -> Option<&str>;
    pub fn get_outgoing_relationships(&self, source_id: SctId) -> Option<&Vec<Rf2Relationship>>;
    pub fn get_incoming_relationships(&self, dest_id: SctId) -> Option<&Vec<Rf2Relationship>>;

    // Hierarchy navigation
    pub fn get_parents(&self, concept_id: SctId) -> Vec<SctId>;
    pub fn get_children(&self, concept_id: SctId) -> Vec<SctId>;

    // Statistics
    pub fn concept_count(&self) -> usize;
    pub fn description_count(&self) -> usize;
    pub fn relationship_count(&self) -> usize;
    pub fn concepts(&self) -> impl Iterator<Item = &Rf2Concept>;
    pub fn concept_ids(&self) -> impl Iterator<Item = &SctId>;
    pub fn estimated_memory_bytes(&self) -> usize;
}
```

## Filter Traits

### DescriptionFilter

```rust
pub trait DescriptionFilter {
    fn passes_description_filter(&self, config: &DescriptionConfig) -> bool;
}

impl DescriptionFilter for Rf2Description {
    // Filters by language_codes and type_ids
}
```

### RelationshipFilter

```rust
pub trait RelationshipFilter {
    fn passes_relationship_filter(&self, config: &RelationshipConfig) -> bool;
}

impl RelationshipFilter for Rf2Relationship {
    // Filters by type_ids and characteristic_type_ids
}
```

## MRCM Module

The `mrcm` submodule provides parsing for MRCM reference sets:

```rust
pub use mrcm::MrcmStore;
pub use mrcm::parse_domain_file;
pub use mrcm::parse_attribute_domain_file;
pub use mrcm::parse_attribute_range_file;
```

See [05-mrcm-explained.md](05-mrcm-explained.md) for details.

## ECL (Expression Constraint Language) Support

The `snomed-loader` crate implements the `EclQueryable` trait from `snomed-ecl-executor`, enabling ECL queries to be executed against `SnomedStore`.

### Dependencies

ECL support is provided via the `snomed-ecl` and `snomed-ecl-executor` crates from the [snomed-ecl-rust](https://github.com/shehanm83/snomed-ecl-rust) repository.

### Re-exports

```rust
// ECL types re-exported for convenience
pub use snomed_ecl;
pub use snomed_ecl_executor::{EclExecutor, EclQueryable, ExecutorConfig, QueryResult};
```

### Usage

```rust
use snomed_loader::{discover_rf2_files, SnomedStore, EclExecutor};

// Load SNOMED CT data
let files = discover_rf2_files("path/to/release")?;
let mut store = SnomedStore::new();
store.load_all(&files)?;

// Create ECL executor
let executor = EclExecutor::new(&store);

// Execute ECL query - descendants of Diabetes mellitus
let result = executor.execute("<< 73211009")?;
println!("Found {} concepts", result.count());

for concept_id in result.iter().take(10) {
    println!("  {}", concept_id);
}

// Check if a concept matches an ECL expression
let matches = executor.matches(46635009, "<< 73211009")?;
println!("Type 1 diabetes is a diabetes: {}", matches);

// Get all ancestors/descendants
let ancestors = executor.get_ancestors(46635009);
let descendants = executor.get_descendants(73211009);
```

### Supported ECL Operations

- `*` - Wildcard (all concepts)
- `< id` - Descendants of
- `<< id` - Descendant or self
- `> id` - Ancestors of
- `>> id` - Ancestor or self
- `! id` - All except
- `AND`, `OR`, `MINUS` - Set operations
- `{ attr = value }` - Attribute refinement
- `( expr )` - Grouping

## Feature Flags

```toml
[features]
default = ["parallel"]
parallel = ["rayon"]      # Parallel parsing with rayon
```

## Complete Usage Example

```rust
use snomed_loader::{discover_rf2_files, SnomedStore, Rf2Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Discover files in release directory
    let files = discover_rf2_files("path/to/SnomedCT_Release")?;
    println!("Found release: {:?}", files.release_date);

    // Load into store (uses parallel loading if feature enabled)
    let mut store = SnomedStore::new();

    #[cfg(feature = "parallel")]
    {
        let (concepts, descriptions, relationships) = store.load_all_parallel(&files)?;
        println!("Loaded {} concepts, {} descriptions, {} relationships",
            concepts, descriptions, relationships);
    }

    #[cfg(not(feature = "parallel"))]
    {
        store.load_all(&files)?;
        println!("Loaded {} concepts", store.concept_count());
    }

    // Load MRCM data
    store.load_mrcm(&files)?;

    // Query the store
    let diabetes_id = 73211009;
    if let Some(concept) = store.get_concept(diabetes_id) {
        println!("Found: {} (active={})", concept.id, concept.active);

        if let Some(fsn) = store.get_fsn(concept.id) {
            println!("FSN: {}", fsn.term);
        }

        if let Some(term) = store.get_preferred_term(concept.id) {
            println!("Preferred term: {}", term);
        }

        let parents = store.get_parents(concept.id);
        println!("Parents: {:?}", parents);

        let children = store.get_children(concept.id);
        println!("Children count: {}", children.len());
    }

    // Memory usage
    println!("Estimated memory: {}",
        snomed_loader::format_bytes(store.estimated_memory_bytes()));

    Ok(())
}
```
