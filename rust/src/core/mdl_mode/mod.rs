//! MDL Read Mode (F8) — structural descriptions for compact code representation.
//!
//! Generates minimum-length structural fingerprints of source files using the
//! Minimum Description Length principle.

pub mod bounds;
pub mod structural;

pub use structural::StructuralDescription;
pub use structural::generate_structural_description;
