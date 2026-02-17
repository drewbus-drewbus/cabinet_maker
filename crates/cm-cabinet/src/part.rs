use cm_core::geometry::Rect;
use serde::{Deserialize, Serialize};

/// A single rectangular panel to be cut from sheet goods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    /// Unique label for this part (e.g., "left_side", "shelf_1").
    pub label: String,

    /// The 2D rectangle representing this part (width x height in project units).
    /// Width is along the grain direction.
    pub rect: Rect,

    /// Material thickness (used to determine cut depth for profile cuts).
    pub thickness: f64,

    /// Grain direction matters for nesting on sheet goods.
    #[serde(default)]
    pub grain_direction: GrainDirection,

    /// Operations to perform on this part (dados, rabbets, holes) before profile cutting.
    #[serde(default)]
    pub operations: Vec<PartOperation>,

    /// Quantity needed.
    #[serde(default = "default_quantity")]
    pub quantity: u32,
}

fn default_quantity() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GrainDirection {
    /// Grain runs along the width (X axis) of the part.
    #[default]
    LengthWise,
    /// Grain runs along the height (Y axis) of the part.
    WidthWise,
}

/// An operation to perform on a part beyond the profile cut.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PartOperation {
    /// A dado (groove across the grain or along it).
    Dado(DadoOp),
    /// A rabbet (groove along an edge).
    Rabbet(RabbetOp),
    /// A drill hole.
    Drill(DrillOp),
}

/// A dado cut: a rectangular groove in the face of a panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DadoOp {
    /// Position along the part's height (Y) where the dado center is.
    pub position: f64,
    /// Width of the dado (typically matches mating part thickness).
    pub width: f64,
    /// Depth of the dado (typically half the panel thickness).
    pub depth: f64,
    /// Orientation: does the dado run along the width (horizontal) or height (vertical)?
    #[serde(default)]
    pub orientation: DadoOrientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DadoOrientation {
    /// Dado runs across the width of the part (most common: shelf dados in side panels).
    #[default]
    Horizontal,
    /// Dado runs along the height of the part.
    Vertical,
}

/// A rabbet cut along an edge of the panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RabbetOp {
    /// Which edge of the panel.
    pub edge: Edge,
    /// Width of the rabbet.
    pub width: f64,
    /// Depth of the rabbet.
    pub depth: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

/// A drill operation (single hole).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrillOp {
    /// X position of hole center relative to part origin.
    pub x: f64,
    /// Y position of hole center relative to part origin.
    pub y: f64,
    /// Hole diameter.
    pub diameter: f64,
    /// Hole depth.
    pub depth: f64,
}
