use cm_core::geometry::Point2D;
use serde::{Deserialize, Serialize};

/// A complete toolpath for one operation on one part.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toolpath {
    /// Tool number to use (references the tool library).
    pub tool_number: u32,

    /// Spindle RPM.
    pub rpm: f64,

    /// Feed rate for cutting moves (project units per minute).
    pub feed_rate: f64,

    /// Plunge feed rate (Z moves into material).
    pub plunge_rate: f64,

    /// The sequence of motions.
    pub segments: Vec<ToolpathSegment>,
}

/// A single segment of a toolpath.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolpathSegment {
    pub motion: Motion,
    pub endpoint: Point2D,
    /// Z height at the endpoint. Negative = into material (convention: Z=0 is material surface).
    pub z: f64,
}

/// Types of CNC motion.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Motion {
    /// G00: Rapid move (no cutting, max speed).
    Rapid,
    /// G01: Linear feed (cutting).
    Linear,
    /// G02: Clockwise arc.
    ArcCW {
        /// Arc center relative to start point (I, J values).
        i: f64,
        j: f64,
    },
    /// G03: Counter-clockwise arc.
    ArcCCW {
        /// Arc center relative to start point (I, J values).
        i: f64,
        j: f64,
    },
}
