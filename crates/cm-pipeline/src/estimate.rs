//! Quick design-time cost estimation without nesting.
//!
//! Provides instant feedback as the user edits a project, estimating
//! sheet count, material cost, hardware cost, edge banding, and weight
//! without running the full nesting pipeline.

use cm_cabinet::project::{Project, TaggedPart};
use cm_hardware::auto_assign_hardware;
use serde::Serialize;

/// Utilization factor for area-based sheet estimation (no actual nesting).
const ESTIMATED_UTILIZATION: f64 = 0.85;

/// Quick project estimate — fast enough for real-time UI updates.
#[derive(Debug, Clone, Serialize)]
pub struct QuickEstimate {
    pub total_parts: u32,
    pub total_area_sqft: f64,
    pub estimated_sheets: u32,
    pub material_cost: Option<f64>,
    pub hardware_cost: Option<f64>,
    pub edge_banding_cost: Option<f64>,
    pub total_estimated_cost: Option<f64>,
    pub total_weight_lb: f64,
    pub per_cabinet: Vec<CabinetEstimate>,
}

/// Per-cabinet cost estimate.
#[derive(Debug, Clone, Serialize)]
pub struct CabinetEstimate {
    pub name: String,
    pub cabinet_type: String,
    pub part_count: u32,
    pub area_sqft: f64,
    pub weight_lb: f64,
    pub hardware_cost: Option<f64>,
    pub edge_banding_lf: f64,
}

/// Compute a quick estimate for the entire project.
///
/// This generates parts and computes area/weight/hardware without running
/// the nesting algorithm, making it suitable for real-time feedback.
pub fn quick_estimate(project: &Project) -> QuickEstimate {
    let tagged_parts = project.generate_all_parts();
    let all_cabs = project.all_cabinets();

    let mut per_cabinet = Vec::with_capacity(all_cabs.len());
    let mut total_area_sqft = 0.0;
    let mut total_weight = 0.0;
    let mut total_parts = 0u32;
    let mut total_hw_cost: Option<f64> = None;
    let mut total_eb_cost = 0.0;
    let mut _total_eb_lf = 0.0;

    // Group by material for sheet estimation
    let mut area_by_material: std::collections::HashMap<String, (f64, Option<f64>, Option<f64>)> =
        std::collections::HashMap::new(); // name -> (area, sheet_area, cost_per_sheet)

    for cab in &all_cabs {
        let cab_parts: Vec<&TaggedPart> = tagged_parts
            .iter()
            .filter(|tp| tp.cabinet_name == cab.name)
            .collect();

        let mut cab_area = 0.0;
        let mut cab_weight = 0.0;
        let mut cab_part_count = 0u32;

        for tp in &cab_parts {
            let area = (tp.part.rect.width * tp.part.rect.height * tp.part.quantity as f64) / 144.0;
            let volume_cuft = area * tp.part.thickness / 12.0;
            let density = tp.material.effective_density();
            let weight = volume_cuft * density;

            cab_area += area;
            cab_weight += weight;
            cab_part_count += tp.part.quantity;

            // Accumulate area by material for sheet estimation
            let entry = area_by_material
                .entry(tp.material_name.clone())
                .or_insert((0.0, None, None));
            entry.0 += area;
            if entry.1.is_none() {
                let sheet_w = tp.material.sheet_width.unwrap_or(48.0);
                let sheet_l = tp.material.sheet_length.unwrap_or(96.0);
                entry.1 = Some(sheet_w * sheet_l / 144.0);
                entry.2 = tp.material.cost_per_unit;
            }
        }

        // Hardware
        let hw = auto_assign_hardware(cab);
        let cab_hw_cost: Option<f64> = {
            let costs: Vec<f64> = hw
                .iter()
                .filter_map(|a| a.hardware.unit_price.map(|p| p * a.quantity as f64))
                .collect();
            if costs.is_empty() {
                None
            } else {
                Some(costs.iter().sum())
            }
        };
        let hw_weight: f64 = hw
            .iter()
            .map(|a| a.hardware.unit_weight_oz.unwrap_or(0.0) * a.quantity as f64 / 16.0)
            .sum();

        cab_weight += hw_weight;

        if let Some(c) = cab_hw_cost {
            *total_hw_cost.get_or_insert(0.0) += c;
        }

        // Edge banding
        let eb_lf = estimate_edge_banding(&cab_parts);
        let eb_cost = eb_lf * 0.50; // $0.50/ft default
        _total_eb_lf += eb_lf;
        total_eb_cost += eb_cost;

        total_area_sqft += cab_area;
        total_weight += cab_weight;
        total_parts += cab_part_count;

        per_cabinet.push(CabinetEstimate {
            name: cab.name.clone(),
            cabinet_type: format!("{:?}", cab.cabinet_type),
            part_count: cab_part_count,
            area_sqft: cab_area,
            weight_lb: cab_weight,
            hardware_cost: cab_hw_cost,
            edge_banding_lf: eb_lf,
        });
    }

    // Estimate sheets from area (no nesting)
    let mut estimated_sheets = 0u32;
    let mut material_cost: Option<f64> = None;
    for (area, sheet_area, cost_per_sheet) in area_by_material.values() {
        if let Some(sa) = sheet_area {
            let usable = sa * ESTIMATED_UTILIZATION;
            let sheets = (area / usable).ceil() as u32;
            estimated_sheets += sheets;
            if let Some(c) = cost_per_sheet {
                *material_cost.get_or_insert(0.0) += c * sheets as f64;
            }
        }
    }

    let eb_cost_opt = if total_eb_cost > 0.0 {
        Some(total_eb_cost)
    } else {
        None
    };

    let total_estimated_cost = {
        let mut total = 0.0;
        let mut any = false;
        if let Some(m) = material_cost {
            total += m;
            any = true;
        }
        if let Some(h) = total_hw_cost {
            total += h;
            any = true;
        }
        if let Some(e) = eb_cost_opt {
            total += e;
            any = true;
        }
        if any { Some(total) } else { None }
    };

    QuickEstimate {
        total_parts,
        total_area_sqft,
        estimated_sheets,
        material_cost,
        hardware_cost: total_hw_cost,
        edge_banding_cost: eb_cost_opt,
        total_estimated_cost,
        total_weight_lb: total_weight,
        per_cabinet,
    }
}

/// Estimate edge banding linear footage from parts (same logic as BOM).
fn estimate_edge_banding(parts: &[&TaggedPart]) -> f64 {
    let mut total_inches = 0.0;

    for tp in parts {
        let label = &tp.part.label;
        let qty = tp.part.quantity as f64;

        let edge_length = if label == "left_side" || label == "right_side" {
            tp.part.rect.height
        } else if label.starts_with("shelf") || label == "top" || label == "bottom" {
            tp.part.rect.width
        } else if label.starts_with("divider") {
            tp.part.rect.height
        } else if label.starts_with("stretcher") {
            tp.part.rect.width
        } else {
            0.0
        };

        total_inches += edge_length * qty;
    }

    total_inches / 12.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use cm_cabinet::cabinet::*;
    use cm_cabinet::project::ProjectMeta;
    use cm_core::material::{Material, MaterialType};
    use cm_core::units::Unit;

    fn bookshelf_project() -> Project {
        Project {
            project: ProjectMeta {
                name: "Test Bookshelf".to_string(),
                units: Unit::Inches,
            },
            material: Some(Material {
                name: "3/4\" Plywood".to_string(),
                thickness: 0.75,
                sheet_width: Some(48.0),
                sheet_length: Some(96.0),
                cost_per_unit: Some(45.00),
                material_type: MaterialType::Plywood,
                density_lb_per_ft3: None,
            }),
            back_material: None,
            cabinet: Some(Cabinet {
                name: "bookshelf".to_string(),
                cabinet_type: CabinetType::BasicBox,
                width: 36.0,
                height: 30.0,
                depth: 12.0,
                material_thickness: 0.75,
                back_thickness: 0.25,
                shelf_count: 2,
                shelf_joinery: ShelfJoinery::Dado,
                dado_depth_fraction: 0.5,
                has_back: true,
                back_joinery: BackJoinery::Rabbet,
                toe_kick: None,
                drawers: None,
                stretchers: None,
                construction: ConstructionMethod::Frameless,
                face_frame: None,
                corner_type: None,
                plumbing_cutout: None,
            }),
            materials: vec![],
            cabinets: vec![],
            tools: vec![],
        }
    }

    #[test]
    fn test_basic_estimate() {
        let estimate = quick_estimate(&bookshelf_project());

        assert!(estimate.total_parts > 0, "should have parts");
        assert!(estimate.total_area_sqft > 0.0, "should have area");
        assert!(estimate.estimated_sheets >= 1, "should need at least 1 sheet");
        assert!(estimate.total_weight_lb > 5.0, "should have reasonable weight");
        assert!(estimate.per_cabinet.len() == 1);
        assert_eq!(estimate.per_cabinet[0].name, "bookshelf");
    }

    #[test]
    fn test_estimate_vs_bom_weight_match() {
        let project = bookshelf_project();
        let estimate = quick_estimate(&project);

        // BOM weight should match estimate weight (same calculation)
        let tagged_parts = project.generate_all_parts();
        let bom = crate::generate_bom(&project, &tagged_parts, 1, None);

        let weight_diff = (estimate.total_weight_lb - bom.totals.total_weight_lb).abs();
        assert!(
            weight_diff < 0.01,
            "estimate weight ({:.3}) should match BOM weight ({:.3})",
            estimate.total_weight_lb,
            bom.totals.total_weight_lb
        );
    }

    #[test]
    fn test_estimate_vs_bom_hardware_match() {
        let project = bookshelf_project();
        let estimate = quick_estimate(&project);
        let tagged_parts = project.generate_all_parts();
        let bom = crate::generate_bom(&project, &tagged_parts, 1, None);

        // Hardware cost should match
        assert_eq!(
            estimate.hardware_cost.is_some(),
            bom.cabinets[0].cost.hardware_cost.is_some()
        );
        if let (Some(est), Some(bom_hw)) = (estimate.hardware_cost, bom.cabinets[0].cost.hardware_cost) {
            assert!(
                (est - bom_hw).abs() < 0.01,
                "hardware cost should match: est={:.2}, bom={:.2}",
                est,
                bom_hw
            );
        }
    }

    #[test]
    fn test_estimate_sheet_count_reasonable() {
        let project = bookshelf_project();
        let estimate = quick_estimate(&project);

        // A single bookshelf (36x30x12) should need 1 sheet of 4x8 plywood
        assert!(
            estimate.estimated_sheets <= 2,
            "bookshelf should need 1-2 sheets, got {}",
            estimate.estimated_sheets
        );
    }

    #[test]
    fn test_estimate_material_cost() {
        let project = bookshelf_project();
        let estimate = quick_estimate(&project);

        assert!(estimate.material_cost.is_some(), "should have material cost");
        let cost = estimate.material_cost.unwrap();
        assert!(cost >= 45.0, "should cost at least 1 sheet ($45): ${:.2}", cost);
    }

    #[test]
    fn test_estimate_edge_banding() {
        let project = bookshelf_project();
        let estimate = quick_estimate(&project);

        assert!(
            estimate.edge_banding_cost.is_some(),
            "should have edge banding cost"
        );
        assert!(
            estimate.per_cabinet[0].edge_banding_lf > 0.0,
            "should have edge banding footage"
        );
    }

    #[test]
    fn test_estimate_multi_cabinet() {
        let toml = r#"
[project]
name = "Kitchen"
units = "inches"

[[materials]]
name = "3/4\" Plywood"
thickness = 0.75
sheet_width = 48.0
sheet_length = 96.0
material_type = "plywood"
cost_per_unit = 45.0

[[cabinets]]
name = "base"
cabinet_type = "base_cabinet"
width = 24.0
height = 34.5
depth = 24.0
material_thickness = 0.75
shelf_count = 1
has_back = true
back_joinery = "rabbet"
material_ref = "3/4\" Plywood"
toe_kick = { height = 4.0, setback = 3.0 }

[[cabinets]]
name = "wall"
cabinet_type = "wall_cabinet"
width = 30.0
height = 30.0
depth = 12.0
material_thickness = 0.75
shelf_count = 2
has_back = true
back_joinery = "rabbet"
material_ref = "3/4\" Plywood"
"#;
        let project = Project::from_toml(toml).unwrap();
        let estimate = quick_estimate(&project);

        assert_eq!(estimate.per_cabinet.len(), 2);
        assert_eq!(estimate.per_cabinet[0].name, "base");
        assert_eq!(estimate.per_cabinet[1].name, "wall");

        // Total should be sum of per-cabinet
        let sum_area: f64 = estimate.per_cabinet.iter().map(|c| c.area_sqft).sum();
        assert!(
            (estimate.total_area_sqft - sum_area).abs() < 0.01,
            "total area should match sum of cabinets"
        );

        assert!(estimate.estimated_sheets >= 2, "kitchen should need multiple sheets");
        assert!(estimate.total_estimated_cost.is_some());
    }

    #[test]
    fn test_estimate_empty_project() {
        let project = Project {
            project: ProjectMeta {
                name: "Empty".to_string(),
                units: Unit::Inches,
            },
            material: None,
            back_material: None,
            cabinet: None,
            materials: vec![],
            cabinets: vec![],
            tools: vec![],
        };
        let estimate = quick_estimate(&project);

        assert_eq!(estimate.total_parts, 0);
        assert_eq!(estimate.estimated_sheets, 0);
        assert!(estimate.total_estimated_cost.is_none());
        assert!(estimate.per_cabinet.is_empty());
    }
}
