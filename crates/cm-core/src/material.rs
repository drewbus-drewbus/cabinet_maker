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

    /// Material density in lb/ft³ (optional — uses MaterialType default if not specified).
    #[serde(default)]
    pub density_lb_per_ft3: Option<f64>,
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
            density_lb_per_ft3: None,
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
            density_lb_per_ft3: None,
        }
    }

    /// Returns the density in lb/ft³, using the explicit value if set,
    /// otherwise a default based on MaterialType.
    pub fn effective_density(&self) -> f64 {
        self.density_lb_per_ft3.unwrap_or_else(|| self.material_type.default_density())
    }
}

impl MaterialType {
    /// Default density in lb/ft³ for each material type.
    pub fn default_density(self) -> f64 {
        match self {
            MaterialType::Plywood => 34.0,
            MaterialType::Mdf => 48.0,
            MaterialType::Hardwood => 44.0,
            MaterialType::Softwood => 28.0,
            MaterialType::Melamine => 45.0,
            MaterialType::Particleboard => 42.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_density_uses_explicit_value() {
        let mat = Material {
            name: "Custom".into(),
            thickness: 0.75,
            sheet_width: Some(48.0),
            sheet_length: Some(96.0),
            cost_per_unit: None,
            material_type: MaterialType::Plywood,
            density_lb_per_ft3: Some(50.0),
        };
        assert!((mat.effective_density() - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_effective_density_falls_back_to_default() {
        let mat = Material {
            name: "Standard Plywood".into(),
            thickness: 0.75,
            sheet_width: Some(48.0),
            sheet_length: Some(96.0),
            cost_per_unit: None,
            material_type: MaterialType::Plywood,
            density_lb_per_ft3: None,
        };
        assert!((mat.effective_density() - 34.0).abs() < 1e-10);
    }

    #[test]
    fn test_all_material_type_densities() {
        let expected = [
            (MaterialType::Plywood, 34.0),
            (MaterialType::Mdf, 48.0),
            (MaterialType::Hardwood, 44.0),
            (MaterialType::Softwood, 28.0),
            (MaterialType::Melamine, 45.0),
            (MaterialType::Particleboard, 42.0),
        ];
        for (mt, density) in expected {
            assert!(
                (mt.default_density() - density).abs() < 1e-10,
                "{:?} density should be {}, got {}",
                mt, density, mt.default_density()
            );
        }
    }

    #[test]
    fn test_effective_density_per_material_type_defaults() {
        for mt in [
            MaterialType::Plywood, MaterialType::Mdf, MaterialType::Hardwood,
            MaterialType::Softwood, MaterialType::Melamine, MaterialType::Particleboard,
        ] {
            let mat = Material {
                name: format!("{:?}", mt),
                thickness: 0.75,
                sheet_width: None,
                sheet_length: None,
                cost_per_unit: None,
                material_type: mt,
                density_lb_per_ft3: None,
            };
            assert!(
                (mat.effective_density() - mt.default_density()).abs() < 1e-10,
                "effective_density() for {:?} should match default_density()", mt
            );
        }
    }

    #[test]
    fn test_material_serde_with_density() {
        let mat = Material {
            name: "Heavy Plywood".into(),
            thickness: 0.75,
            sheet_width: Some(48.0),
            sheet_length: Some(96.0),
            cost_per_unit: Some(55.0),
            material_type: MaterialType::Plywood,
            density_lb_per_ft3: Some(38.0),
        };
        let json = serde_json::to_string(&mat).unwrap();
        let mat2: Material = serde_json::from_str(&json).unwrap();
        assert_eq!(mat2.density_lb_per_ft3, Some(38.0));
        assert!((mat2.effective_density() - 38.0).abs() < 1e-10);
    }

    #[test]
    fn test_material_serde_without_density() {
        let mat = Material::plywood_3_4();
        let json = serde_json::to_string(&mat).unwrap();
        let mat2: Material = serde_json::from_str(&json).unwrap();
        assert_eq!(mat2.density_lb_per_ft3, None);
        // Should fall back to plywood default
        assert!((mat2.effective_density() - 34.0).abs() < 1e-10);
    }

    #[test]
    fn test_plywood_3_4_constructor() {
        let mat = Material::plywood_3_4();
        assert!((mat.thickness - 0.75).abs() < 1e-10);
        assert_eq!(mat.sheet_width, Some(48.0));
        assert_eq!(mat.sheet_length, Some(96.0));
        assert_eq!(mat.material_type, MaterialType::Plywood);
        assert_eq!(mat.density_lb_per_ft3, None);
    }

    #[test]
    fn test_plywood_1_4_constructor() {
        let mat = Material::plywood_1_4();
        assert!((mat.thickness - 0.25).abs() < 1e-10);
        assert_eq!(mat.material_type, MaterialType::Plywood);
    }

    #[test]
    fn test_material_type_default_is_plywood() {
        assert_eq!(MaterialType::default(), MaterialType::Plywood);
    }

    #[test]
    fn test_material_toml_round_trip() {
        let mat = Material {
            name: "MDF".into(),
            thickness: 0.75,
            sheet_width: Some(48.0),
            sheet_length: Some(96.0),
            cost_per_unit: Some(32.0),
            material_type: MaterialType::Mdf,
            density_lb_per_ft3: Some(50.0),
        };
        let toml_str = toml::to_string_pretty(&mat).unwrap();
        let mat2: Material = toml::from_str(&toml_str).unwrap();
        assert_eq!(mat2.name, "MDF");
        assert_eq!(mat2.material_type, MaterialType::Mdf);
        assert_eq!(mat2.density_lb_per_ft3, Some(50.0));
    }

    #[test]
    fn test_weight_calculation_math() {
        // Verify: a 1ft x 1ft x 1" plywood panel should weigh:
        // volume = 1ft² × (1/12)ft = 1/12 ft³
        // weight = (1/12) × 34 lb/ft³ = 2.833 lb
        let mat = Material::plywood_3_4();
        let density = mat.effective_density(); // 34.0
        let area_sqft = 1.0; // 1 ft²
        let thickness_ft = 1.0 / 12.0;
        let volume_cuft = area_sqft * thickness_ft;
        let weight = volume_cuft * density;
        assert!((weight - 34.0 / 12.0).abs() < 1e-10);
    }
}
