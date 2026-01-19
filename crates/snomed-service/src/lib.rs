//! # snomed-service
//!
//! gRPC service for SNOMED CT terminology queries.
//!
//! This crate provides a gRPC server that exposes SNOMED CT data
//! loaded by the snomed-loader crate.
//!
//! ## ECL Support
//!
//! This crate includes ECL (Expression Constraint Language) support via the
//! `snomed-ecl-executor` crate. The [`EclQueryable`](snomed_loader::EclQueryable)
//! trait is implemented for [`SnomedStore`](snomed_loader::SnomedStore) in the
//! snomed-loader crate, enabling ECL queries to be executed against loaded SNOMED CT data.

#![warn(missing_docs)]

#[allow(missing_docs)]
pub mod proto {
    //! Generated protobuf types.
    tonic::include_proto!("snomed");
}

mod server;
mod services;

pub use server::SnomedServer;

// Re-export ECL types from snomed-loader for convenience
pub use snomed_loader::{EclExecutor, EclQueryable, ExecutorConfig};
