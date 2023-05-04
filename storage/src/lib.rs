//! # The Spark Storage Library
//!
//! The Spark Storage Library provides two main components:
//! - The relation database
//! - The sparse merkle tree database

pub mod relation_db;
pub mod smt;

mod error;
mod traits;
mod types;
