use cm_core::material::Material;
use cm_core::tool::Tool;
use cm_core::units::Unit;
use serde::{Deserialize, Serialize};

use crate::cabinet::Cabinet;
use crate::part::Part;

/// A complete project definition, deserialized from a TOML file.
///
/// Supports both single-cabinet (legacy) and multi-cabinet formats:
///
/// **Legacy format** (still fully supported):
/// ```toml
/// [material]
/// name = "3/4\" Plywood"
/// ...
/// [cabinet]
/// name = "bookshelf"
/// ...
/// ```
///
/// **Multi-cabinet format**:
/// ```toml
/// [[materials]]
/// name = "3/4\" Maple Plywood"
/// ...
/// [[cabinets]]
/// name = "sink_base"
/// material_ref = "3/4\" Maple Plywood"
/// ...
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Project metadata.
    pub project: ProjectMeta,

    /// Single material (legacy format).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<Material>,

    /// Back panel material (legacy format).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub back_material: Option<Material>,

    /// Single cabinet (legacy format).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cabinet: Option<Cabinet>,

    /// Multiple materials (multi-cabinet format).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub materials: Vec<Material>,

    /// Multiple cabinets with material references (multi-cabinet format).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cabinets: Vec<CabinetEntry>,

    /// Available tools.
    #[serde(default)]
    pub tools: Vec<Tool>,
}

/// A cabinet entry in a multi-cabinet project, wrapping a Cabinet with
/// material references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CabinetEntry {
    /// The cabinet definition (all fields flattened in).
    #[serde(flatten)]
    pub cabinet: Cabinet,

    /// Reference to a material name in the `materials` array.
    /// If omitted, uses the first material.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_ref: Option<String>,

    /// Reference to a back panel material name.
    /// If omitted, uses a default 1/4" plywood.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub back_material_ref: Option<String>,
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

/// A part tagged with its source cabinet and material info, for multi-cabinet nesting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedPart {
    /// The part itself.
    pub part: Part,
    /// Which cabinet this part belongs to (cabinet name).
    pub cabinet_name: String,
    /// Material name for this part.
    pub material_name: String,
    /// Material details.
    pub material: Material,
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

    /// Get all materials in this project (merging legacy + multi-cabinet).
    pub fn all_materials(&self) -> Vec<&Material> {
        let mut mats: Vec<&Material> = Vec::new();
        if let Some(ref m) = self.material {
            mats.push(m);
        }
        if let Some(ref m) = self.back_material {
            mats.push(m);
        }
        for m in &self.materials {
            mats.push(m);
        }
        mats
    }

    /// Get the primary material (first material available).
    pub fn primary_material(&self) -> Option<&Material> {
        self.material.as_ref().or_else(|| self.materials.first())
    }

    /// Get all cabinets in this project (merging legacy + multi-cabinet).
    pub fn all_cabinets(&self) -> Vec<&Cabinet> {
        let mut cabs: Vec<&Cabinet> = Vec::new();
        if let Some(ref c) = self.cabinet {
            cabs.push(c);
        }
        for entry in &self.cabinets {
            cabs.push(&entry.cabinet);
        }
        cabs
    }

    /// Resolve the material for a given cabinet entry.
    /// Falls back to the first available material.
    pub fn resolve_material(&self, entry: &CabinetEntry) -> Material {
        if let Some(ref name) = entry.material_ref
            && let Some(m) = self.materials.iter().find(|m| m.name == *name)
        {
            return m.clone();
        }
        // Fall back to first material in list, or legacy material
        self.materials
            .first()
            .or(self.material.as_ref())
            .cloned()
            .unwrap_or_else(Material::plywood_3_4)
    }

    /// Resolve the back material for a given cabinet entry.
    pub fn resolve_back_material(&self, entry: &CabinetEntry) -> Material {
        if let Some(ref name) = entry.back_material_ref
            && let Some(m) = self.materials.iter().find(|m| m.name == *name)
        {
            return m.clone();
        }
        // Fall back to legacy back_material or default
        self.back_material
            .clone()
            .unwrap_or_else(Material::plywood_1_4)
    }

    /// Generate all parts for all cabinets, tagged with cabinet name and material.
    /// Parts are grouped by material for efficient nesting.
    pub fn generate_all_parts(&self) -> Vec<TaggedPart> {
        let mut tagged = Vec::new();

        // Legacy single-cabinet
        if let Some(ref cab) = self.cabinet {
            let mat = self.material.clone().unwrap_or_else(Material::plywood_3_4);
            let back_mat = self.back_material.clone().unwrap_or_else(Material::plywood_1_4);
            let parts = cab.generate_parts();
            for part in parts {
                let (mat_name, material) = if part.label == "back" {
                    (back_mat.name.clone(), back_mat.clone())
                } else {
                    (mat.name.clone(), mat.clone())
                };
                tagged.push(TaggedPart {
                    part,
                    cabinet_name: cab.name.clone(),
                    material_name: mat_name,
                    material,
                });
            }
        }

        // Multi-cabinet entries
        for entry in &self.cabinets {
            let mat = self.resolve_material(entry);
            let back_mat = self.resolve_back_material(entry);
            let parts = entry.cabinet.generate_parts();
            for part in parts {
                let (mat_name, material) = if part.label == "back" {
                    (back_mat.name.clone(), back_mat.clone())
                } else {
                    (mat.name.clone(), mat.clone())
                };
                tagged.push(TaggedPart {
                    part,
                    cabinet_name: entry.cabinet.name.clone(),
                    material_name: mat_name,
                    material,
                });
            }
        }

        tagged
    }

    /// Get unique material groups from tagged parts. Returns (material_name, material, parts) tuples.
    pub fn group_parts_by_material(tagged: &[TaggedPart]) -> Vec<MaterialGroup<'_>> {
        let mut groups: Vec<MaterialGroup> = Vec::new();
        for tp in tagged {
            if let Some(group) = groups.iter_mut().find(|g| g.material_name == tp.material_name) {
                group.parts.push(tp);
            } else {
                groups.push(MaterialGroup {
                    material_name: tp.material_name.clone(),
                    material: tp.material.clone(),
                    parts: vec![tp],
                });
            }
        }
        groups
    }
}

/// A group of parts sharing the same material, for per-material nesting.
pub struct MaterialGroup<'a> {
    pub material_name: String,
    pub material: Material,
    pub parts: Vec<&'a TaggedPart>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEGACY_TOML: &str = r#"
[project]
name = "Test Bookshelf"
units = "inches"

[material]
name = "3/4\" Birch Plywood"
thickness = 0.75
sheet_width = 48.0
sheet_length = 96.0
material_type = "plywood"

[cabinet]
name = "bookshelf"
cabinet_type = "basic_box"
width = 36.0
height = 30.0
depth = 12.0
material_thickness = 0.75
back_thickness = 0.25
shelf_count = 2
shelf_joinery = "dado"
dado_depth_fraction = 0.5
has_back = true
back_joinery = "rabbet"

[[tools]]
number = 1
tool_type = "endmill"
diameter = 0.25
flutes = 2
cutting_length = 1.0
description = "1/4\" 2-flute upcut endmill"
"#;

    const MULTI_TOML: &str = r#"
[project]
name = "Kitchen Cabinets"
units = "inches"

[[materials]]
name = "3/4\" Maple Plywood"
thickness = 0.75
sheet_width = 48.0
sheet_length = 96.0
material_type = "plywood"

[[materials]]
name = "1/4\" Maple Plywood"
thickness = 0.25
sheet_width = 48.0
sheet_length = 96.0
material_type = "plywood"

[[cabinets]]
name = "sink_base"
cabinet_type = "sink_base"
width = 36.0
height = 34.5
depth = 24.0
material_thickness = 0.75
back_thickness = 0.25
has_back = true
material_ref = "3/4\" Maple Plywood"
back_material_ref = "1/4\" Maple Plywood"
toe_kick = { height = 4.0, setback = 3.0 }
stretchers = { front_width = 4.0, has_rear = true }

[[cabinets]]
name = "wall_unit"
cabinet_type = "wall_cabinet"
width = 30.0
height = 30.0
depth = 12.0
material_thickness = 0.75
back_thickness = 0.25
shelf_count = 2
has_back = true
material_ref = "3/4\" Maple Plywood"
back_material_ref = "1/4\" Maple Plywood"

[[tools]]
number = 1
tool_type = "endmill"
diameter = 0.25
flutes = 2
cutting_length = 1.0
description = "1/4\" 2-flute upcut endmill"
"#;

    #[test]
    fn test_legacy_format_loads() {
        let project = Project::from_toml(LEGACY_TOML).unwrap();
        assert_eq!(project.project.name, "Test Bookshelf");
        assert!(project.cabinet.is_some());
        assert!(project.material.is_some());
        assert!(project.cabinets.is_empty());
        assert!(project.materials.is_empty());
    }

    #[test]
    fn test_legacy_all_cabinets() {
        let project = Project::from_toml(LEGACY_TOML).unwrap();
        let cabs = project.all_cabinets();
        assert_eq!(cabs.len(), 1);
        assert_eq!(cabs[0].name, "bookshelf");
    }

    #[test]
    fn test_multi_format_loads() {
        let project = Project::from_toml(MULTI_TOML).unwrap();
        assert_eq!(project.project.name, "Kitchen Cabinets");
        assert!(project.cabinet.is_none());
        assert!(project.material.is_none());
        assert_eq!(project.cabinets.len(), 2);
        assert_eq!(project.materials.len(), 2);
    }

    #[test]
    fn test_multi_all_cabinets() {
        let project = Project::from_toml(MULTI_TOML).unwrap();
        let cabs = project.all_cabinets();
        assert_eq!(cabs.len(), 2);
        assert_eq!(cabs[0].name, "sink_base");
        assert_eq!(cabs[1].name, "wall_unit");
    }

    #[test]
    fn test_multi_material_resolution() {
        let project = Project::from_toml(MULTI_TOML).unwrap();
        let entry = &project.cabinets[0];
        let mat = project.resolve_material(entry);
        assert_eq!(mat.name, "3/4\" Maple Plywood");
        assert!((mat.thickness - 0.75).abs() < 1e-10);

        let back_mat = project.resolve_back_material(entry);
        assert_eq!(back_mat.name, "1/4\" Maple Plywood");
        assert!((back_mat.thickness - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_generate_all_parts_legacy() {
        let project = Project::from_toml(LEGACY_TOML).unwrap();
        let tagged = project.generate_all_parts();
        // bookshelf: 2 sides + top + bottom + shelf(qty1, count=2) + back = 6
        assert_eq!(tagged.len(), 6);
        assert!(tagged.iter().all(|t| t.cabinet_name == "bookshelf"));
    }

    #[test]
    fn test_generate_all_parts_multi() {
        let project = Project::from_toml(MULTI_TOML).unwrap();
        let tagged = project.generate_all_parts();
        // sink_base: 2 sides + front_stretcher + rear_stretcher + back = 5
        // wall_unit: 2 sides + top + bottom + shelf(qty1, count=2) + back = 6
        // Total = 11
        assert_eq!(tagged.len(), 11);

        let sink_parts: Vec<_> = tagged.iter().filter(|t| t.cabinet_name == "sink_base").collect();
        assert_eq!(sink_parts.len(), 5);

        let wall_parts: Vec<_> = tagged.iter().filter(|t| t.cabinet_name == "wall_unit").collect();
        assert_eq!(wall_parts.len(), 6);
    }

    #[test]
    fn test_group_parts_by_material() {
        let project = Project::from_toml(MULTI_TOML).unwrap();
        let tagged = project.generate_all_parts();
        let groups = Project::group_parts_by_material(&tagged);

        // Should have 2 groups: 3/4" and 1/4"
        assert_eq!(groups.len(), 2);

        // All back panels should be in the 1/4" group
        let thin_group = groups.iter().find(|g| g.material_name.contains("1/4")).unwrap();
        assert!(thin_group.parts.iter().all(|p| p.part.label == "back"));

        // All non-back panels should be in the 3/4" group
        let thick_group = groups.iter().find(|g| g.material_name.contains("3/4")).unwrap();
        assert!(thick_group.parts.iter().all(|p| p.part.label != "back"));
    }

    #[test]
    fn test_legacy_round_trip() {
        let project = Project::from_toml(LEGACY_TOML).unwrap();
        let toml_str = project.to_toml().unwrap();
        let project2 = Project::from_toml(&toml_str).unwrap();
        assert_eq!(project.project.name, project2.project.name);
        let cab1 = project.cabinet.as_ref().unwrap();
        let cab2 = project2.cabinet.as_ref().unwrap();
        assert_eq!(cab1.width, cab2.width);
        assert_eq!(cab1.height, cab2.height);
    }

    #[test]
    fn test_multi_cabinet_tagged_part_labels() {
        let project = Project::from_toml(MULTI_TOML).unwrap();
        let tagged = project.generate_all_parts();

        // Verify cabinet names are correctly assigned
        let sink_lefts: Vec<_> = tagged.iter()
            .filter(|t| t.cabinet_name == "sink_base" && t.part.label == "left_side")
            .collect();
        assert_eq!(sink_lefts.len(), 1);
    }
}
