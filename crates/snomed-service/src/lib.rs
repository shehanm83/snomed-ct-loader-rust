//! # snomed-service
//!
//! gRPC service for SNOMED CT terminology queries.
//!
//! This crate provides a gRPC server that exposes SNOMED CT data
//! loaded by the snomed-loader crate.

#![warn(missing_docs)]

#[allow(missing_docs)]
pub mod proto {
    //! Generated protobuf types.
    tonic::include_proto!("snomed");
}

mod server;
mod services;

pub use server::SnomedServer;
