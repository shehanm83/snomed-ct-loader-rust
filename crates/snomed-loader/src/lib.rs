//! # snomed-loader
//!
//! High-performance parallel parser for SNOMED CT RF2 distribution files.
//!
//! This crate provides a streaming parser for SNOMED CT Release Format 2 (RF2) files,
//! including concepts, descriptions, and relationships. Includes parallel parsing
//! support via rayon for maximum performance on multi-core systems.
//!
//! ## Features
//!
//! - `parallel` - Enables parallel parsing via rayon (default)
//! - `progress` - Enables progress bar support via indicatif (optional)
//!
//! ## Usage
//!
//! ### Basic Parsing
//!
//! ```ignore
//! use snomed_loader::{Rf2Parser, Rf2Config};
//! use snomed_types::Rf2Concept;
//!
//! let config = Rf2Config::default();
//! let parser = Rf2Parser::<_, Rf2Concept>::from_path("concepts.txt", config)?;
//!
//! for result in parser {
//!     match result {
//!         Ok(concept) => println!("Concept: {} (active: {})", concept.id, concept.active),
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```
//!
//! ### Using the Store
//!
//! ```ignore
//! use snomed_loader::{SnomedStore, discover_rf2_files};
//!
//! // Discover RF2 files in a release directory
//! let files = discover_rf2_files("/path/to/snomed/release")?;
//!
//! // Load into store
//! let mut store = SnomedStore::new();
//! store.load_all(&files)?;
//!
//! // Query the store
//! if let Some(concept) = store.get_concept(73211009) {
//!     println!("Found: {:?}", concept);
//! }
//!
//! // Get parents via IS_A relationships
//! let parents = store.get_parents(73211009);
//! println!("Parents: {:?}", parents);
//! ```
//!
//! ### Filtering
//!
//! ```ignore
//! use snomed_loader::{Rf2Parser, DescriptionConfig, DescriptionFilter};
//! use snomed_types::Rf2Description;
//!
//! // Parse only English FSN descriptions
//! let config = DescriptionConfig::fsn_only();
//! let parser = Rf2Parser::<_, Rf2Description>::from_path("descriptions.txt", config.base)?;
//!
//! let fsn_descriptions: Vec<_> = parser
//!     .filter_map(Result::ok)
//!     .filter(|d| d.passes_description_filter(&config))
//!     .collect();
//! ```

#![warn(missing_docs)]

mod concept;
mod description;
mod loader;
pub mod mrcm;
mod parser;
mod relationship;
mod store;
mod types;

// Re-export main types and functions
pub use loader::{discover_rf2_files, format_bytes};
pub use parser::{parse, Rf2Parser, Rf2Record};
pub use store::SnomedStore;
pub use types::{
    DescriptionConfig, ParseStats, RelationshipConfig, Rf2Config, Rf2Error, Rf2Files, Rf2Result,
};

// Re-export filter traits
pub use description::DescriptionFilter;
pub use relationship::RelationshipFilter;

// Re-export snomed-types for convenience
pub use snomed_types;
