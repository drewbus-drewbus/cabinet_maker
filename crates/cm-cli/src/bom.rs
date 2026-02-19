//! Comprehensive Bill of Materials (BOM) with per-cabinet breakdown,
//! hardware costing, edge banding estimation, and weight calculation.

use cm_cabinet::project::{Project, TaggedPart};
use cm_hardware::auto_assign_hardware;
use serde::Serialize;

/// Top-level BOM for an entire project.
#[derive(Debug, Clone, Serialize)]
pub struct Bom {
    pub project_name: String,
    pub cabinets: Vec<CabinetBom>,
    pub totals: BomTotals,
}

/// Per-cabinet BOM breakdown.
#[derive(Debug, Clone, Serialize)]
pub struct CabinetBom {
    pub cabinet_name: String,
    pub cabinet_type: String,
    pub sheet_parts: Vec<SheetPartEntry>,
    pub hardware: Vec<HardwareEntry>,
    pub edge_banding: Vec<EdgeBandEntry>,
    pub cost: CostSummary,
    pub weight_lb: f64,
}

/// A sheet-good part in the BOM.
#[derive(Debug, Clone, Serialize)]
pub struct SheetPartEntry {
    pub label: String,
    pub width: f64,
    pub height: f64,
    pub thickness: f64,
    pub quantity: u32,
    pub area_sqft: f64,
    pub weight_lb: f64,
    pub material: String,
}

/// A hardware item in the BOM.
#[derive(Debug, Clone, Serialize)]
pub struct HardwareEntry {
    pub id: String,
    pub description: String,
    pub quantity: u32,
    pub unit_price: Option<f64>,
    pub line_total: Option<f64>,
}

/// Edge banding in the BOM.
#[derive(Debug, Clone, Serialize)]
pub struct EdgeBandEntry {
    pub material: String,
    pub linear_feet: f64,
    pub cost_per_foot: Option<f64>,
    pub line_total: Option<f64>,
}

/// Cost summary for a cabinet.
#[derive(Debug, Clone, Serialize)]
pub struct CostSummary {
    pub sheet_cost: Option<f64>,
    pub hardware_cost: Option<f64>,
    pub edge_banding_cost: Option<f64>,
    pub total: Option<f64>,
}

/// Aggregated totals across all cabinets.
#[derive(Debug, Clone, Serialize)]
pub struct BomTotals {
    pub total_sheet_parts: u32,
    pub total_sheets_required: u32,
    pub total_hardware_items: u32,
    pub total_edge_banding_lf: f64,
    pub total_weight_lb: f64,
    pub total_cost: Option<f64>,
}

/// Generate a comprehensive BOM from a project, its parts, and nesting results.
pub fn generate_bom(
    project: &Project,
    tagged_parts: &[TaggedPart],
    total_sheets: u32,
    overall_cost_per_sheet: Option<f64>,
) -> Bom {
    let all_cabs = project.all_cabinets();

    let mut cabinet_boms = Vec::new();

    for cab in &all_cabs {
        // Parts belonging to this cabinet
        let cab_parts: Vec<&TaggedPart> = tagged_parts.iter()
            .filter(|tp| tp.cabinet_name == cab.name)
            .collect();

        // Sheet parts
        let sheet_parts: Vec<SheetPartEntry> = cab_parts.iter().map(|tp| {
            let area_sqft = (tp.part.rect.width * tp.part.rect.height * tp.part.quantity as f64) / 144.0;
            let volume_cuft = area_sqft * tp.part.thickness / 12.0;
            let density = tp.material.effective_density();
            let weight = volume_cuft * density;
            SheetPartEntry {
                label: tp.part.label.clone(),
                width: tp.part.rect.width,
                height: tp.part.rect.height,
                thickness: tp.part.thickness,
                quantity: tp.part.quantity,
                area_sqft,
                weight_lb: weight,
                material: tp.material_name.clone(),
            }
        }).collect();

        // Hardware assignments
        let hw_assignments = auto_assign_hardware(cab);
        let hardware: Vec<HardwareEntry> = hw_assignments.iter().map(|a| {
            let line_total = a.hardware.unit_price.map(|p| p * a.quantity as f64);
            HardwareEntry {
                id: a.hardware.id.clone(),
                description: a.description.clone(),
                quantity: a.quantity,
                unit_price: a.hardware.unit_price,
                line_total,
            }
        }).collect();

        // Edge banding estimation
        let edge_banding = calculate_edge_banding(&cab_parts, cab.material_thickness);

        // Weight
        let sheet_weight: f64 = sheet_parts.iter().map(|p| p.weight_lb).sum();
        let hw_weight: f64 = hw_assignments.iter()
            .map(|a| {
                a.hardware.unit_weight_oz.unwrap_or(0.0) * a.quantity as f64 / 16.0
            })
            .sum();
        let total_weight = sheet_weight + hw_weight;

        // Cost summary
        let hardware_cost = sum_optional(hardware.iter().map(|h| h.line_total));
        let edge_cost = sum_optional(edge_banding.iter().map(|e| e.line_total));

        let cost = CostSummary {
            sheet_cost: None, // set at project level from nesting
            hardware_cost,
            edge_banding_cost: edge_cost,
            total: None, // computed below
        };

        cabinet_boms.push(CabinetBom {
            cabinet_name: cab.name.clone(),
            cabinet_type: format!("{:?}", cab.cabinet_type),
            sheet_parts,
            hardware,
            edge_banding,
            cost,
            weight_lb: total_weight,
        });
    }

    // Project-level sheet cost from nesting
    let sheet_cost = overall_cost_per_sheet.map(|c| c * total_sheets as f64);

    // Compute per-cabinet totals (spread sheet cost proportionally)
    if let Some(sc) = sheet_cost {
        let total_area: f64 = cabinet_boms.iter()
            .flat_map(|cb| cb.sheet_parts.iter())
            .map(|p| p.area_sqft)
            .sum();
        if total_area > 0.0 {
            for cb in &mut cabinet_boms {
                let cab_area: f64 = cb.sheet_parts.iter().map(|p| p.area_sqft).sum();
                let cab_sheet_cost = sc * cab_area / total_area;
                cb.cost.sheet_cost = Some(cab_sheet_cost);
                cb.cost.total = Some(
                    cab_sheet_cost
                    + cb.cost.hardware_cost.unwrap_or(0.0)
                    + cb.cost.edge_banding_cost.unwrap_or(0.0)
                );
            }
        }
    } else {
        // No sheet cost — total is hardware + edge banding only
        for cb in &mut cabinet_boms {
            let hw = cb.cost.hardware_cost.unwrap_or(0.0);
            let eb = cb.cost.edge_banding_cost.unwrap_or(0.0);
            if cb.cost.hardware_cost.is_some() || cb.cost.edge_banding_cost.is_some() {
                cb.cost.total = Some(hw + eb);
            }
        }
    }

    // Totals
    let total_sheet_parts: u32 = cabinet_boms.iter()
        .flat_map(|cb| cb.sheet_parts.iter())
        .map(|p| p.quantity)
        .sum();
    let total_hardware_items: u32 = cabinet_boms.iter()
        .flat_map(|cb| cb.hardware.iter())
        .map(|h| h.quantity)
        .sum();
    let total_edge_banding_lf: f64 = cabinet_boms.iter()
        .flat_map(|cb| cb.edge_banding.iter())
        .map(|e| e.linear_feet)
        .sum();
    let total_weight_lb: f64 = cabinet_boms.iter()
        .map(|cb| cb.weight_lb)
        .sum();
    let total_cost = sum_optional(cabinet_boms.iter().map(|cb| cb.cost.total));

    Bom {
        project_name: project.project.name.clone(),
        cabinets: cabinet_boms,
        totals: BomTotals {
            total_sheet_parts,
            total_sheets_required: total_sheets,
            total_hardware_items,
            total_edge_banding_lf,
            total_weight_lb,
            total_cost,
        },
    }
}

/// Calculate edge banding linear footage by part label convention.
///
/// Front edges get banded:
/// - `left_side`, `right_side` → front edge = panel height
/// - `shelf`, `top`, `bottom` → front edge = panel width (minus 2× material thickness for dadoed shelves)
/// - `divider` → front edge = panel height
/// - `stretcher` → front edge = panel width
/// - `back` → no edge banding
fn calculate_edge_banding(parts: &[&TaggedPart], _material_thickness: f64) -> Vec<EdgeBandEntry> {
    let mut total_linear_inches = 0.0;

    for tp in parts {
        let label = &tp.part.label;
        let qty = tp.part.quantity as f64;

        let edge_length = if label == "left_side" || label == "right_side" {
            // Front edge = height of panel
            tp.part.rect.height
        } else if label.starts_with("shelf") {
            // Front edge = width, accounting for dado inset on both sides
            tp.part.rect.width
        } else if label == "top" || label == "bottom" {
            // Front edge = width
            tp.part.rect.width
        } else if label.starts_with("divider") {
            tp.part.rect.height
        } else if label.starts_with("stretcher") {
            tp.part.rect.width
        } else {
            // back, stile, rail, etc. — no edge banding
            0.0
        };

        total_linear_inches += edge_length * qty;
    }

    if total_linear_inches <= 0.0 {
        return Vec::new();
    }

    let linear_feet = total_linear_inches / 12.0;
    let cost_per_foot = 0.50; // default PVC edge banding cost

    vec![EdgeBandEntry {
        material: "PVC edge banding".to_string(),
        linear_feet,
        cost_per_foot: Some(cost_per_foot),
        line_total: Some(linear_feet * cost_per_foot),
    }]
}

/// Sum Option<f64> values — returns None if all are None, Some(sum) otherwise.
fn sum_optional(values: impl Iterator<Item = Option<f64>>) -> Option<f64> {
    let mut any_some = false;
    let mut total = 0.0;
    for v in values {
        if let Some(val) = v {
            any_some = true;
            total += val;
        }
    }
    if any_some { Some(total) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cm_cabinet::cabinet::*;
    use cm_cabinet::project::ProjectMeta;
    use cm_core::material::{Material, MaterialType};
    use cm_core::units::Unit;

    fn test_project() -> Project {
        Project {
            project: ProjectMeta {
                name: "Test BOM".to_string(),
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
                name: "test_box".to_string(),
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
    fn test_weight_calculation() {
        let project = test_project();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, None);

        assert_eq!(bom.cabinets.len(), 1);
        assert!(bom.totals.total_weight_lb > 0.0, "weight should be positive");

        // Verify weight is reasonable for a 36x30x12 bookshelf in plywood
        // ~6 panels, density 34 lb/ft³ ≈ 15-25 lbs total
        assert!(bom.totals.total_weight_lb > 5.0, "too light: {:.2} lb", bom.totals.total_weight_lb);
        assert!(bom.totals.total_weight_lb < 50.0, "too heavy: {:.2} lb", bom.totals.total_weight_lb);
    }

    #[test]
    fn test_edge_banding_linear_footage() {
        let project = test_project();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, None);

        let total_eb: f64 = bom.cabinets[0].edge_banding.iter()
            .map(|e| e.linear_feet)
            .sum();
        // Sides: 2 × 30" = 60", top+bottom: 2 × ~34.5" = 69", shelves: 2 × ~34.5" = 69"
        // Total ≈ 198" = 16.5 linear feet
        assert!(total_eb > 10.0, "edge banding too short: {:.2} lf", total_eb);
        assert!(total_eb < 30.0, "edge banding too long: {:.2} lf", total_eb);
    }

    #[test]
    fn test_hardware_cost_rollup() {
        let project = test_project();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, None);

        // BasicBox with shelves → shelf pins only
        let hw_cost = bom.cabinets[0].cost.hardware_cost;
        assert!(hw_cost.is_some(), "should have hardware cost");
        assert!(hw_cost.unwrap() > 0.0, "hardware cost should be positive");
    }

    #[test]
    fn test_hardware_cost_none_handling() {
        // A cabinet with no hardware should have None for hardware_cost
        let mut project = test_project();
        if let Some(ref mut cab) = project.cabinet {
            cab.shelf_count = 0;
            cab.cabinet_type = CabinetType::BasicBox;
        }
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, None);

        // BasicBox with 0 shelves → no hardware
        assert!(bom.cabinets[0].hardware.is_empty());
        assert!(bom.cabinets[0].cost.hardware_cost.is_none());
    }

    #[test]
    fn test_bom_json_structure() {
        let project = test_project();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, Some(45.00));

        let json = serde_json::to_string_pretty(&bom).expect("BOM should serialize to JSON");

        // Verify key fields exist in JSON
        assert!(json.contains("project_name"));
        assert!(json.contains("cabinets"));
        assert!(json.contains("totals"));
        assert!(json.contains("sheet_parts"));
        assert!(json.contains("hardware"));
        assert!(json.contains("edge_banding"));
        assert!(json.contains("total_weight_lb"));
        assert!(json.contains("total_cost"));
    }

    #[test]
    fn test_bom_with_sheet_cost() {
        let project = test_project();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 2, Some(45.00));

        // With sheet cost: $45 × 2 sheets = $90 total sheet cost
        let cab = &bom.cabinets[0];
        assert!(cab.cost.sheet_cost.is_some(), "should have sheet cost");
        assert!(cab.cost.total.is_some(), "should have total cost");

        let total = cab.cost.total.unwrap();
        assert!(total > 45.0, "total should include sheet + hardware + edge banding");
    }

    #[test]
    fn test_bom_totals_aggregation() {
        let project = test_project();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, None);

        assert!(bom.totals.total_sheet_parts > 0);
        assert_eq!(bom.totals.total_sheets_required, 1);
        assert!(bom.totals.total_weight_lb > 0.0);
        assert!(bom.totals.total_edge_banding_lf > 0.0);
    }

    // --- Phase 16d: Additional comprehensive BOM tests ---

    #[test]
    fn test_multi_cabinet_bom() {
        let toml = r#"
[project]
name = "Kitchen Set"
units = "inches"

[[materials]]
name = "3/4\" Plywood"
thickness = 0.75
sheet_width = 48.0
sheet_length = 96.0
material_type = "plywood"
cost_per_unit = 45.0

[[cabinets]]
name = "base_left"
cabinet_type = "base_cabinet"
width = 24.0
height = 34.5
depth = 24.0
material_thickness = 0.75
shelf_count = 1
has_back = false
material_ref = "3/4\" Plywood"
toe_kick = { height = 4.0, setback = 3.0 }

[[cabinets]]
name = "wall_above"
cabinet_type = "wall_cabinet"
width = 24.0
height = 30.0
depth = 12.0
material_thickness = 0.75
shelf_count = 2
has_back = false
material_ref = "3/4\" Plywood"
"#;
        let project = Project::from_toml(toml).unwrap();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 2, Some(45.00));

        // Should have 2 cabinet BOMs
        assert_eq!(bom.cabinets.len(), 2);
        assert_eq!(bom.cabinets[0].cabinet_name, "base_left");
        assert_eq!(bom.cabinets[1].cabinet_name, "wall_above");

        // Each cabinet should have sheet parts
        assert!(!bom.cabinets[0].sheet_parts.is_empty());
        assert!(!bom.cabinets[1].sheet_parts.is_empty());

        // Total parts should be the sum
        let cab0_parts: u32 = bom.cabinets[0].sheet_parts.iter().map(|p| p.quantity).sum();
        let cab1_parts: u32 = bom.cabinets[1].sheet_parts.iter().map(|p| p.quantity).sum();
        assert_eq!(bom.totals.total_sheet_parts, cab0_parts + cab1_parts);

        // Both cabinets have hardware
        assert!(!bom.cabinets[0].hardware.is_empty(), "base cabinet should have hardware");
        assert!(!bom.cabinets[1].hardware.is_empty(), "wall cabinet should have hardware");

        // Total weight = sum of cabinet weights
        let sum_weight: f64 = bom.cabinets.iter().map(|c| c.weight_lb).sum();
        assert!((bom.totals.total_weight_lb - sum_weight).abs() < 1e-10);

        // Sheet cost distributed proportionally
        for cab in &bom.cabinets {
            assert!(cab.cost.sheet_cost.is_some(), "{} should have sheet cost", cab.cabinet_name);
            assert!(cab.cost.total.is_some(), "{} should have total cost", cab.cabinet_name);
        }
    }

    #[test]
    fn test_bom_custom_density_affects_weight() {
        let mut project = test_project();
        // Set a very high density to see the effect
        if let Some(ref mut mat) = project.material {
            mat.density_lb_per_ft3 = Some(100.0); // much heavier than default plywood (34)
        }
        let tagged_parts = project.generate_all_parts();
        let heavy_bom = generate_bom(&project, &tagged_parts, 1, None);

        // Compare with default density
        let mut project2 = test_project();
        if let Some(ref mut mat) = project2.material {
            mat.density_lb_per_ft3 = None; // default 34 lb/ft³
        }
        let tagged_parts2 = project2.generate_all_parts();
        let light_bom = generate_bom(&project2, &tagged_parts2, 1, None);

        assert!(heavy_bom.totals.total_weight_lb > light_bom.totals.total_weight_lb * 2.0,
            "100 lb/ft³ should be much heavier than 34 lb/ft³: {} vs {}",
            heavy_bom.totals.total_weight_lb, light_bom.totals.total_weight_lb);
    }

    #[test]
    fn test_bom_no_shelves_no_edge_banding_on_shelves() {
        let mut project = test_project();
        if let Some(ref mut cab) = project.cabinet {
            cab.shelf_count = 0;
        }
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, None);

        // Should still have edge banding for sides, top, bottom
        let total_eb: f64 = bom.cabinets[0].edge_banding.iter()
            .map(|e| e.linear_feet)
            .sum();
        assert!(total_eb > 0.0, "should still have edge banding for top/bottom/sides");

        // But less than with shelves
        let project2 = test_project();
        let tagged_parts2 = project2.generate_all_parts();
        let bom2 = generate_bom(&project2, &tagged_parts2, 1, None);
        let total_eb2: f64 = bom2.cabinets[0].edge_banding.iter()
            .map(|e| e.linear_feet)
            .sum();
        assert!(total_eb < total_eb2, "no shelves = less edge banding: {} < {}", total_eb, total_eb2);
    }

    #[test]
    fn test_bom_sheet_parts_have_correct_material() {
        let project = test_project();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, None);

        for part in &bom.cabinets[0].sheet_parts {
            assert!(!part.material.is_empty(), "material name should not be empty");
            // Most parts should use main material; back uses back material
            if part.label == "back" {
                assert!(part.thickness < 0.5, "back should be thin: {}", part.thickness);
            } else {
                assert!((part.thickness - 0.75).abs() < 1e-10,
                    "{} should be 0.75\" thick", part.label);
            }
        }
    }

    #[test]
    fn test_bom_area_calculation() {
        let project = test_project();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, None);

        // Check area for left_side: 12" × 30" = 360 sq in = 2.5 sq ft
        let left = bom.cabinets[0].sheet_parts.iter()
            .find(|p| p.label == "left_side")
            .unwrap();
        assert!((left.area_sqft - 2.5).abs() < 1e-10,
            "left_side area should be 2.5 sqft, got {}", left.area_sqft);
    }

    #[test]
    fn test_bom_shelf_quantity_correct() {
        let project = test_project();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, None);

        let shelf = bom.cabinets[0].sheet_parts.iter()
            .find(|p| p.label == "shelf")
            .unwrap();
        assert_eq!(shelf.quantity, 2, "shelf quantity should be 2");
    }

    #[test]
    fn test_bom_total_cost_without_sheet_cost() {
        let project = test_project();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, None);

        // Without sheet cost, total = hardware + edge banding only
        let cab = &bom.cabinets[0];
        assert!(cab.cost.sheet_cost.is_none());
        let expected = cab.cost.hardware_cost.unwrap_or(0.0) + cab.cost.edge_banding_cost.unwrap_or(0.0);
        assert!((cab.cost.total.unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_bom_edge_banding_cost() {
        let project = test_project();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, None);

        let cab = &bom.cabinets[0];
        assert!(cab.cost.edge_banding_cost.is_some());
        // Edge banding at $0.50/ft, typically 10-20 lf = $5-10
        let eb_cost = cab.cost.edge_banding_cost.unwrap();
        assert!(eb_cost > 2.0, "edge banding cost too low: {:.2}", eb_cost);
        assert!(eb_cost < 20.0, "edge banding cost too high: {:.2}", eb_cost);
    }

    #[test]
    fn test_bom_hardware_entries_have_ids() {
        let project = test_project();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, None);

        for entry in &bom.cabinets[0].hardware {
            assert!(!entry.id.is_empty(), "hardware id should not be empty");
            assert!(!entry.description.is_empty(), "hardware description should not be empty");
            assert!(entry.quantity > 0, "hardware quantity should be positive");
        }
    }

    #[test]
    fn test_bom_wall_cabinet_hardware_cost() {
        // Wall cabinet has hinges, pulls, and shelf pins
        let toml = r#"
[project]
name = "Wall Test"
units = "inches"

[material]
name = "3/4\" Plywood"
thickness = 0.75
sheet_width = 48.0
sheet_length = 96.0
material_type = "plywood"
cost_per_unit = 45.0

[cabinet]
name = "wall"
cabinet_type = "wall_cabinet"
width = 30.0
height = 30.0
depth = 12.0
material_thickness = 0.75
shelf_count = 2
has_back = true
back_joinery = "rabbet"
"#;
        let project = Project::from_toml(toml).unwrap();
        let tagged_parts = project.generate_all_parts();
        let bom = generate_bom(&project, &tagged_parts, 1, None);

        // Should have hinge, pull, and shelf pin hardware entries
        assert!(bom.cabinets[0].hardware.len() >= 2, "wall cabinet should have multiple hardware types");

        // Total hardware cost should include hinges ($3.50 ea) + pulls ($4 ea) + pins ($0.15 ea)
        let total_hw_cost = bom.cabinets[0].cost.hardware_cost.unwrap();
        assert!(total_hw_cost > 10.0, "wall cabinet hardware should cost >$10, got ${:.2}", total_hw_cost);
    }
}
