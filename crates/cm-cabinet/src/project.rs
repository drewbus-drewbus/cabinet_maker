use cm_core::material::Material;
use cm_core::tool::Tool;
use cm_core::units::Unit;
use serde::{Deserialize, Serialize};

use crate::cabinet::Cabinet;

/// A complete project definition, deserialized from a TOML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Project metadata.
    pub project: ProjectMeta,

    /// Primary carcass material.
    pub material: Material,

    /// Back panel material (optional, defaults to 1/4" ply).
    pub back_material: Option<Material>,

    /// Cabinet definition(s).
    pub cabinet: Cabinet,

    /// Available tools.
    #[serde(default)]
    pub tools: Vec<Tool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(default = "default_units")]
    pub units: Unit,
}

fn default_units() -> Unit {
    Unit::Inches
}

impl Project {
    /// Load a project from a TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// Serialize the project to a TOML string.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}
