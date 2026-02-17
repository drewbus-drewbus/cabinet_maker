use serde::{Deserialize, Serialize};

/// A material that parts can be made from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    /// Display name (e.g., "3/4 Birch Plywood")
    pub name: String,

    /// Actual thickness in project units (e.g., 0.75 for 3/4" plywood,
    /// which is actually ~0.703" but nominal 0.75" is standard for joinery math).
    pub thickness: f64,

    /// Sheet width for sheet goods (e.g., 48.0 for 4' plywood).
    /// None for solid wood.
    pub sheet_width: Option<f64>,

    /// Sheet length for sheet goods (e.g., 96.0 for 8' plywood).
    /// None for solid wood.
    pub sheet_length: Option<f64>,

    /// Cost per sheet (for sheet goods) or per board-foot (for solid wood).
    pub cost_per_unit: Option<f64>,

    /// Material type affects feed/speed recommendations.
    #[serde(default)]
    pub material_type: MaterialType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MaterialType {
    #[default]
    Plywood,
    Mdf,
    Hardwood,
    Softwood,
    Melamine,
    Particleboard,
}

impl Material {
    /// Create a standard 3/4" plywood material.
    pub fn plywood_3_4() -> Self {
        Self {
            name: "3/4\" Plywood".into(),
            thickness: 0.75,
            sheet_width: Some(48.0),
            sheet_length: Some(96.0),
            cost_per_unit: None,
            material_type: MaterialType::Plywood,
        }
    }

    /// Create a standard 1/4" plywood material (for backs).
    pub fn plywood_1_4() -> Self {
        Self {
            name: "1/4\" Plywood".into(),
            thickness: 0.25,
            sheet_width: Some(48.0),
            sheet_length: Some(96.0),
            cost_per_unit: None,
            material_type: MaterialType::Plywood,
        }
    }
}
