//! Faithful Rust translation scaffold for MUSCLE.
//!
//! The original C++ sources live under `muscle/src`. This crate starts as an
//! audit-friendly one-to-one scaffold: each original function has a Rust stub
//! in [`generated`], and `ccc_mapping.toml` pins those stubs to original paths
//! and line numbers for code-complexity-comparator workflows.

#![allow(dead_code)]

pub mod generated;

pub use generated::*;
