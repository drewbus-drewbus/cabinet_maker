//! Toolpath visualization — annotated toolpaths with part/operation metadata
//! for frontend rendering.

use cm_cabinet::part::PartOperation;
use cm_cabinet::project::TaggedPart;
use cm_cam::ops::CamConfig;
use cm_cam::{Motion, Toolpath};
use cm_core::geometry::{Point2D, Rect};
use cm_core::tool::Tool;
use cm_nesting::packer::SheetLayout;
use serde::{Deserialize, Serialize};

use crate::generate::{generate_operation_toolpath, strip_nesting_id};
use cm_cam::ops::generate_profile_cut;

/// The type of CNC operation a toolpath represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    Profile,
    Dado,
    Rabbet,
    Drill,
    PocketHole,
    Dovetail,
    BoxJoint,
    Mortise,
    Tenon,
    Dowel,
}

/// A toolpath annotated with part label and operation type for visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedToolpath {
    /// The underlying toolpath with segments.
    pub toolpath: Toolpath,
    /// Label of the part this toolpath belongs to (e.g. "Left Side").
    pub part_label: String,
    /// The nesting placement ID (e.g. "Kitchen/Left Side_1").
    pub placement_id: String,
    /// What kind of operation this toolpath performs.
    pub operation_type: OperationType,
}

/// Visualization DTO returned by the API with pre-computed stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolpathVisualizationDto {
    /// All annotated toolpaths for this sheet.
    pub toolpaths: Vec<AnnotatedToolpath>,
    /// Sheet dimensions.
    pub sheet_width: f64,
    pub sheet_height: f64,
    /// Total number of toolpath segments.
    pub total_segments: usize,
    /// Total rapid travel distance (in project units).
    pub rapid_distance: f64,
    /// Total cutting distance (in project units).
    pub cut_distance: f64,
    /// Number of distinct parts on this sheet.
    pub part_count: usize,
    /// Estimated machining time in seconds (rough: cut_distance / feed_rate + rapids).
    pub estimated_time_s: f64,
    /// Bounding box of all toolpaths: [min_x, min_y, max_x, max_y].
    pub bounds: [f64; 4],
}

/// Classify a `PartOperation` into an `OperationType`.
fn classify_operation(op: &PartOperation) -> OperationType {
    match op {
        PartOperation::Dado(_) => OperationType::Dado,
        PartOperation::Rabbet(_) => OperationType::Rabbet,
        PartOperation::Drill(_) => OperationType::Drill,
        PartOperation::PocketHole(_) => OperationType::PocketHole,
        PartOperation::Dovetail(_) => OperationType::Dovetail,
        PartOperation::BoxJoint(_) => OperationType::BoxJoint,
        PartOperation::Mortise(_) => OperationType::Mortise,
        PartOperation::Tenon(_) => OperationType::Tenon,
        PartOperation::Dowel(_) => OperationType::Dowel,
    }
}

/// Generate annotated toolpaths for all parts on a single sheet.
///
/// This is the visualization counterpart of `generate_sheet_toolpaths` —
/// each toolpath is tagged with metadata for frontend rendering.
pub fn generate_annotated_toolpaths(
    sheet: &SheetLayout,
    parts: &[&TaggedPart],
    tool: &Tool,
    rpm: f64,
    cam_config: &CamConfig,
) -> ToolpathVisualizationDto {
    let mut annotated = Vec::new();

    for placed in &sheet.parts {
        let base_label = strip_nesting_id(&placed.id);

        let tp_match = parts.iter()
            .find(|tp| tp.part.label == base_label || placed.id.ends_with(&tp.part.label));

        let Some(tp_match) = tp_match else {
            continue;
        };

        let positioned_rect = Rect::new(
            placed.rect.origin,
            placed.rect.width,
            placed.rect.height,
        );

        // Generate annotated toolpaths for each operation
        for op in &tp_match.part.operations {
            if let Some(toolpath) = generate_operation_toolpath(op, &positioned_rect, tool, rpm, cam_config) {
                annotated.push(AnnotatedToolpath {
                    toolpath,
                    part_label: tp_match.part.label.clone(),
                    placement_id: placed.id.clone(),
                    operation_type: classify_operation(op),
                });
            }
        }

        // Profile cut (always last)
        let profile = generate_profile_cut(&positioned_rect, tp_match.part.thickness, tool, rpm, cam_config);
        annotated.push(AnnotatedToolpath {
            toolpath: profile,
            part_label: tp_match.part.label.clone(),
            placement_id: placed.id.clone(),
            operation_type: OperationType::Profile,
        });
    }

    // Compute stats
    let total_segments: usize = annotated.iter().map(|a| a.toolpath.segments.len()).sum();
    let part_count = sheet.parts.len();

    let mut rapid_distance = 0.0_f64;
    let mut cut_distance = 0.0_f64;
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut avg_feed_rate = 0.0_f64;
    let mut feed_count = 0u32;

    for at in &annotated {
        let tp = &at.toolpath;
        if tp.feed_rate > 0.0 {
            avg_feed_rate += tp.feed_rate;
            feed_count += 1;
        }

        let mut prev = Point2D::origin();
        for seg in &tp.segments {
            let dx = seg.endpoint.x - prev.x;
            let dy = seg.endpoint.y - prev.y;
            let dist = (dx * dx + dy * dy).sqrt();

            match seg.motion {
                Motion::Rapid => rapid_distance += dist,
                _ => cut_distance += dist,
            }

            // Update bounds
            update_bounds(seg.endpoint, &mut min_x, &mut min_y, &mut max_x, &mut max_y);
            prev = seg.endpoint;
        }
    }

    if avg_feed_rate > 0.0 && feed_count > 0 {
        avg_feed_rate /= feed_count as f64;
    }

    // Rough time estimate: cutting at feed rate + rapids at ~200 ipm
    let rapid_speed = 200.0; // inches per minute assumed
    let estimated_time_s = if avg_feed_rate > 0.0 {
        (cut_distance / avg_feed_rate + rapid_distance / rapid_speed) * 60.0
    } else {
        0.0
    };

    let bounds = if min_x <= max_x {
        [min_x, min_y, max_x, max_y]
    } else {
        [0.0, 0.0, sheet.sheet_rect.width, sheet.sheet_rect.height]
    };

    ToolpathVisualizationDto {
        toolpaths: annotated,
        sheet_width: sheet.sheet_rect.width,
        sheet_height: sheet.sheet_rect.height,
        total_segments,
        rapid_distance,
        cut_distance,
        part_count,
        estimated_time_s,
        bounds,
    }
}

fn update_bounds(pt: Point2D, min_x: &mut f64, min_y: &mut f64, max_x: &mut f64, max_y: &mut f64) {
    if pt.x < *min_x { *min_x = pt.x; }
    if pt.y < *min_y { *min_y = pt.y; }
    if pt.x > *max_x { *max_x = pt.x; }
    if pt.y > *max_y { *max_y = pt.y; }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cm_cabinet::part::{DadoOp, DadoOrientation, DrillOp, Part, GrainDirection};
    use cm_core::geometry::Rect;
    use cm_core::material::Material;

    fn test_tool() -> Tool {
        Tool::quarter_inch_endmill()
    }

    fn test_cam_config() -> CamConfig {
        CamConfig::default()
    }

    fn make_tagged_part(label: &str, width: f64, height: f64, ops: Vec<PartOperation>) -> TaggedPart {
        TaggedPart {
            cabinet_name: "test".into(),
            part: Part {
                label: label.into(),
                rect: Rect::new(Point2D::new(0.0, 0.0), width, height),
                thickness: 0.75,
                quantity: 1,
                grain_direction: GrainDirection::LengthWise,
                operations: ops,
            },
            material_name: "Plywood".into(),
            material: Material {
                name: "Plywood".into(),
                thickness: 0.75,
                sheet_width: Some(48.0),
                sheet_length: Some(96.0),
                material_type: Default::default(),
                density_lb_per_ft3: None,
                cost_per_unit: None,
            },
        }
    }

    fn make_sheet(parts: &[(&str, f64, f64, f64, f64)]) -> SheetLayout {
        use cm_nesting::PlacedPart;
        SheetLayout {
            sheet_index: 0,
            sheet_rect: Rect::new(Point2D::new(0.0, 0.0), 48.0, 96.0),
            parts: parts.iter().map(|(id, x, y, w, h)| PlacedPart {
                id: id.to_string(),
                rect: Rect::new(Point2D::new(*x, *y), *w, *h),
                rotated: false,
            }).collect(),
            waste_area: 0.0,
            utilization: 0.0,
        }
    }

    #[test]
    fn test_annotated_toolpaths_profile_only() {
        let tool = test_tool();
        let config = test_cam_config();
        let tp = make_tagged_part("side", 12.0, 30.0, vec![]);
        let parts: Vec<&TaggedPart> = vec![&tp];
        let sheet = make_sheet(&[("side", 0.0, 0.0, 12.0, 30.0)]);

        let result = generate_annotated_toolpaths(&sheet, &parts, &tool, 5000.0, &config);

        assert_eq!(result.toolpaths.len(), 1);
        assert_eq!(result.toolpaths[0].operation_type, OperationType::Profile);
        assert_eq!(result.toolpaths[0].part_label, "side");
        assert_eq!(result.part_count, 1);
        assert!(result.total_segments > 0);
        assert!(result.cut_distance > 0.0);
    }

    #[test]
    fn test_annotated_toolpaths_with_dado() {
        let tool = test_tool();
        let config = test_cam_config();
        let tp = make_tagged_part("side", 12.0, 30.0, vec![
            PartOperation::Dado(DadoOp {
                position: 6.0,
                width: 0.75,
                depth: 0.375,
                orientation: DadoOrientation::Horizontal,
            }),
        ]);
        let parts: Vec<&TaggedPart> = vec![&tp];
        let sheet = make_sheet(&[("side", 0.0, 0.0, 12.0, 30.0)]);

        let result = generate_annotated_toolpaths(&sheet, &parts, &tool, 5000.0, &config);

        assert_eq!(result.toolpaths.len(), 2); // dado + profile
        assert_eq!(result.toolpaths[0].operation_type, OperationType::Dado);
        assert_eq!(result.toolpaths[1].operation_type, OperationType::Profile);
    }

    #[test]
    fn test_annotated_toolpaths_multiple_parts() {
        let tool = test_tool();
        let config = test_cam_config();
        let tp1 = make_tagged_part("side", 12.0, 30.0, vec![]);
        let tp2 = make_tagged_part("shelf", 11.25, 11.25, vec![]);
        let parts: Vec<&TaggedPart> = vec![&tp1, &tp2];
        let sheet = make_sheet(&[
            ("side", 0.0, 0.0, 12.0, 30.0),
            ("shelf", 13.0, 0.0, 11.25, 11.25),
        ]);

        let result = generate_annotated_toolpaths(&sheet, &parts, &tool, 5000.0, &config);

        assert_eq!(result.toolpaths.len(), 2); // 1 profile each
        assert_eq!(result.part_count, 2);
        assert_eq!(result.toolpaths[0].part_label, "side");
        assert_eq!(result.toolpaths[1].part_label, "shelf");
    }

    #[test]
    fn test_stats_calculation() {
        let tool = test_tool();
        let config = test_cam_config();
        let tp = make_tagged_part("side", 12.0, 30.0, vec![]);
        let parts: Vec<&TaggedPart> = vec![&tp];
        let sheet = make_sheet(&[("side", 0.0, 0.0, 12.0, 30.0)]);

        let result = generate_annotated_toolpaths(&sheet, &parts, &tool, 5000.0, &config);

        assert!(result.total_segments > 0, "should have segments");
        assert!(result.rapid_distance >= 0.0, "rapid distance should be non-negative");
        assert!(result.cut_distance > 0.0, "cut distance should be positive");
        assert!(result.estimated_time_s > 0.0, "estimated time should be positive");
        assert_eq!(result.sheet_width, 48.0);
        assert_eq!(result.sheet_height, 96.0);
    }

    #[test]
    fn test_bounds_calculation() {
        let tool = test_tool();
        let config = test_cam_config();
        let tp = make_tagged_part("side", 12.0, 30.0, vec![]);
        let parts: Vec<&TaggedPart> = vec![&tp];
        let sheet = make_sheet(&[("side", 5.0, 5.0, 12.0, 30.0)]);

        let result = generate_annotated_toolpaths(&sheet, &parts, &tool, 5000.0, &config);

        // Bounds should be at least around the part origin
        assert!(result.bounds[0] >= 0.0, "min_x should be reasonable");
        assert!(result.bounds[1] >= 0.0, "min_y should be reasonable");
        assert!(result.bounds[2] > result.bounds[0], "max_x > min_x");
        assert!(result.bounds[3] > result.bounds[1], "max_y > min_y");
    }

    #[test]
    fn test_empty_sheet() {
        let tool = test_tool();
        let config = test_cam_config();
        let parts: Vec<&TaggedPart> = vec![];
        let sheet = SheetLayout {
            sheet_index: 0,
            sheet_rect: Rect::new(Point2D::new(0.0, 0.0), 48.0, 96.0),
            parts: vec![],
            waste_area: 48.0 * 96.0,
            utilization: 0.0,
        };

        let result = generate_annotated_toolpaths(&sheet, &parts, &tool, 5000.0, &config);

        assert!(result.toolpaths.is_empty());
        assert_eq!(result.total_segments, 0);
        assert_eq!(result.part_count, 0);
    }

    #[test]
    fn test_operation_type_classification() {
        assert_eq!(classify_operation(&PartOperation::Dado(DadoOp {
            position: 0.0, width: 0.75, depth: 0.375, orientation: DadoOrientation::Horizontal,
        })), OperationType::Dado);

        assert_eq!(classify_operation(&PartOperation::Drill(DrillOp {
            x: 1.0, y: 1.0, diameter: 0.25, depth: 0.5,
        })), OperationType::Drill);
    }
}
