//! MRCM (Machine Readable Concept Model) reference set parsers.
//!
//! This module provides parsers for SNOMED CT MRCM reference sets:
//!
//! - **Domain** - Semantic domains where attributes can be applied
//! - **Attribute Domain** - Which attributes are valid in which domains
//! - **Attribute Range** - Valid value ranges for attributes
//!
//! # Usage
//!
//! ```ignore
//! use snomed_loader::mrcm::MrcmStore;
//!
//! // Load MRCM data from RF2 files
//! let store = MrcmStore::load("/path/to/snomed/release")?;
//!
//! // Look up attribute constraints
//! let finding_site = 363698007;
//! if let Some(domains) = store.get_attribute_domains(finding_site) {
//!     for domain in domains {
//!         println!("Valid in domain: {}", domain.domain_id);
//!         println!("Grouped: {}", domain.grouped);
//!     }
//! }
//!
//! // Get range constraints
//! if let Some(ranges) = store.get_attribute_range(finding_site) {
//!     for range in ranges {
//!         println!("Range constraint: {}", range.range_constraint);
//!     }
//! }
//! ```
//!
//! # RF2 File Locations
//!
//! MRCM files are located in the `Refset/Metadata/` directory:
//!
//! ```text
//! Snapshot/
//! └── Refset/
//!     └── Metadata/
//!         ├── der2_cRefset_MRCMDomainSnapshot_*.txt
//!         ├── der2_cRefset_MRCMAttributeDomainSnapshot_*.txt
//!         └── der2_cRefset_MRCMAttributeRangeSnapshot_*.txt
//! ```

mod attribute_domain;
mod attribute_range;
mod domain;
mod store;

pub use attribute_domain::parse_attribute_domain_file;
pub use attribute_range::parse_attribute_range_file;
pub use domain::parse_domain_file;
pub use store::MrcmStore;
