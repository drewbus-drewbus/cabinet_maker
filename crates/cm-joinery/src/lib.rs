//! Joinery Rules Engine for Cabinet Maker.
//!
//! Decouples joinery decisions from cabinet geometry generation.
//! Instead of hardcoding joinery operations in cabinet generators, this crate
//! provides a rules engine that maps joint declarations to concrete operations
//! (dados, rabbets, pocket holes, etc.) based on material and user preferences.
//!
//! # Architecture
//!
//! 1. Cabinet generators produce parts with declared **joints** (connections between parts).
//! 2. `JoineryRules::apply()` transforms joints into `PartOperation`s.
//! 3. Parts with operations flow through the normal nesting → CAM → G-code pipeline.
//!
//! # Example
//!
//! ```rust,ignore
//! let rules = JoineryRuleset::default();
//! let joints = vec![
//!     Joint::shelf_to_side("shelf", "left_side", JointPosition::Horizontal(10.0)),
//! ];
//! let operations = rules.resolve(&joints, material_thickness, &material_type);
//! ```

pub mod rules;
pub mod error;

pub use rules::{
    Joint, JointKind, JointPosition, JoineryRuleset, JoineryRule,
    JoineryMethod, ResolvedOperation,
};
pub use error::JoineryError;
