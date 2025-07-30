//! Cabinet storage library for FoundationDB.
//!
//! This crate provides a high-level interface for storing and retrieving data in FoundationDB
//! with support for tenant isolation and transaction management.

pub use toolbox::foundationdb;

pub mod errors;
pub mod item;
