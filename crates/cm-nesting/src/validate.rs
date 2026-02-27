//! Manual placement validation for interactive nesting.
//!
//! Validates user-placed parts for collisions, out-of-bounds, and
//! computes utilization metrics.

use cm_core::geometry::Rect;
use serde::{Deserialize, Serialize};

use crate::NestingConfig;

/// A manually placed part on a sheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualPlacement {
    /// Unique part identifier.
    pub id: String,
    /// X position on sheet.
    pub x: f64,
    /// Y position on sheet.
    pub y: f64,
    /// Part width (after rotation, if rotated).
    pub width: f64,
    /// Part height (after rotation, if rotated).
    pub height: f64,
    /// Whether this part was rotated 90 degrees.
    pub rotated: bool,
}

/// Collision between two parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionInfo {
    pub part_a: String,
    pub part_b: String,
    /// Overlap area in square inches.
    pub overlap_area: f64,
}

/// Result of validating a manual placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementValidation {
    /// Whether all placements are valid (no collisions, all in bounds).
    pub valid: bool,
    /// Part-pair collisions detected.
    pub collisions: Vec<CollisionInfo>,
    /// IDs of parts that extend beyond the sheet boundary.
    pub out_of_bounds: Vec<String>,
    /// Sheet utilization (0.0 to 1.0) — total part area / sheet area.
    pub utilization: f64,
}

/// Validate a set of manual placements against a sheet.
///
/// Checks for:
/// - AABB collisions between all part pairs (O(n^2), fine for 10-50 parts)
/// - Parts extending beyond sheet boundaries (accounting for kerf)
/// - Utilization calculation
pub fn validate_manual_placement(
    placements: &[ManualPlacement],
    config: &NestingConfig,
) -> PlacementValidation {
    let sheet_width = config.sheet_width;
    let sheet_length = config.sheet_length;
    let kerf = config.kerf;

    let mut collisions = Vec::new();
    let mut out_of_bounds = Vec::new();

    // Check bounds — inflate parts by half kerf on each side
    let half_kerf = kerf / 2.0;
    for p in placements {
        let x_min = p.x - half_kerf;
        let y_min = p.y - half_kerf;
        let x_max = p.x + p.width + half_kerf;
        let y_max = p.y + p.height + half_kerf;

        if x_min < -1e-6 || y_min < -1e-6 || x_max > sheet_width + 1e-6 || y_max > sheet_length + 1e-6 {
            out_of_bounds.push(p.id.clone());
        }
    }

    // Check collisions — AABB overlap with kerf gap
    for i in 0..placements.len() {
        for j in (i + 1)..placements.len() {
            let a = &placements[i];
            let b = &placements[j];

            // Inflated rectangles (each side gets half kerf)
            let a_left = a.x - half_kerf;
            let a_right = a.x + a.width + half_kerf;
            let a_bottom = a.y - half_kerf;
            let a_top = a.y + a.height + half_kerf;

            let b_left = b.x - half_kerf;
            let b_right = b.x + b.width + half_kerf;
            let b_bottom = b.y - half_kerf;
            let b_top = b.y + b.height + half_kerf;

            // AABB overlap
            let overlap_x = (a_right.min(b_right) - a_left.max(b_left)).max(0.0);
            let overlap_y = (a_top.min(b_top) - a_bottom.max(b_bottom)).max(0.0);
            let overlap_area = overlap_x * overlap_y;

            if overlap_area > 1e-6 {
                collisions.push(CollisionInfo {
                    part_a: a.id.clone(),
                    part_b: b.id.clone(),
                    overlap_area,
                });
            }
        }
    }

    // Utilization
    let sheet_area = sheet_width * sheet_length;
    let parts_area: f64 = placements.iter().map(|p| p.width * p.height).sum();
    let utilization = if sheet_area > 0.0 {
        (parts_area / sheet_area).min(1.0)
    } else {
        0.0
    };

    let valid = collisions.is_empty() && out_of_bounds.is_empty();

    PlacementValidation {
        valid,
        collisions,
        out_of_bounds,
        utilization,
    }
}

/// Convert validated manual placements to a SheetLayout.
pub fn sheet_layout_from_manual(
    placements: &[ManualPlacement],
    sheet_index: usize,
    config: &NestingConfig,
) -> crate::SheetLayout {
    use cm_core::geometry::Point2D;

    let sheet_rect = Rect::new(
        Point2D::new(0.0, 0.0),
        config.sheet_width,
        config.sheet_length,
    );

    let sheet_area = config.sheet_width * config.sheet_length;
    let parts_area: f64 = placements.iter().map(|p| p.width * p.height).sum();

    let parts = placements
        .iter()
        .map(|p| crate::PlacedPart {
            id: p.id.clone(),
            rect: Rect::new(Point2D::new(p.x, p.y), p.width, p.height),
            rotated: p.rotated,
        })
        .collect();

    let utilization = if sheet_area > 0.0 {
        parts_area / sheet_area * 100.0
    } else {
        0.0
    };

    crate::SheetLayout {
        sheet_index,
        sheet_rect,
        parts,
        waste_area: (sheet_area - parts_area).max(0.0),
        utilization,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> NestingConfig {
        NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.125,
            edge_margin: 0.0,
            allow_rotation: true,
            guillotine_compatible: false,
        }
    }

    #[test]
    fn test_valid_placement() {
        let config = test_config();
        let kerf = config.kerf;
        // Position parts with kerf margin from edges and between them
        let placements = vec![
            ManualPlacement { id: "a".into(), x: kerf, y: kerf, width: 12.0, height: 30.0, rotated: false },
            ManualPlacement { id: "b".into(), x: 12.0 + 2.0 * kerf, y: kerf, width: 12.0, height: 30.0, rotated: false },
        ];
        let result = validate_manual_placement(&placements, &config);
        assert!(result.valid, "collisions: {:?}, oob: {:?}", result.collisions, result.out_of_bounds);
        assert!(result.collisions.is_empty());
        assert!(result.out_of_bounds.is_empty());
        assert!(result.utilization > 0.0);
    }

    #[test]
    fn test_collision_detected() {
        let placements = vec![
            ManualPlacement { id: "a".into(), x: 0.0, y: 0.0, width: 20.0, height: 30.0, rotated: false },
            ManualPlacement { id: "b".into(), x: 10.0, y: 0.0, width: 20.0, height: 30.0, rotated: false },
        ];
        let result = validate_manual_placement(&placements, &test_config());
        assert!(!result.valid);
        assert_eq!(result.collisions.len(), 1);
        assert_eq!(result.collisions[0].part_a, "a");
        assert_eq!(result.collisions[0].part_b, "b");
        assert!(result.collisions[0].overlap_area > 0.0);
    }

    #[test]
    fn test_out_of_bounds() {
        let placements = vec![
            ManualPlacement { id: "a".into(), x: 40.0, y: 0.0, width: 20.0, height: 30.0, rotated: false },
        ];
        let result = validate_manual_placement(&placements, &test_config());
        assert!(!result.valid);
        assert!(result.out_of_bounds.contains(&"a".to_string()));
    }

    #[test]
    fn test_touching_parts_no_collision_without_kerf() {
        let config = NestingConfig {
            kerf: 0.0, ..test_config()
        };
        let placements = vec![
            ManualPlacement { id: "a".into(), x: 0.0, y: 0.0, width: 12.0, height: 30.0, rotated: false },
            ManualPlacement { id: "b".into(), x: 12.0, y: 0.0, width: 12.0, height: 30.0, rotated: false },
        ];
        let result = validate_manual_placement(&placements, &config);
        assert!(result.valid, "touching but not overlapping parts should be valid");
    }

    #[test]
    fn test_touching_parts_collision_with_kerf() {
        // With kerf = 0.125, parts touching at 12.0 are too close
        let placements = vec![
            ManualPlacement { id: "a".into(), x: 0.0, y: 0.0, width: 12.0, height: 30.0, rotated: false },
            ManualPlacement { id: "b".into(), x: 12.0, y: 0.0, width: 12.0, height: 30.0, rotated: false },
        ];
        let result = validate_manual_placement(&placements, &test_config());
        assert!(!result.valid, "touching parts with kerf should collide");
        assert_eq!(result.collisions.len(), 1);
    }

    #[test]
    fn test_kerf_respected_gap() {
        // Parts separated by exactly kerf width should be valid (if within bounds)
        let config = test_config();
        let kerf = config.kerf;
        let placements = vec![
            ManualPlacement { id: "a".into(), x: kerf, y: kerf, width: 12.0, height: 30.0, rotated: false },
            ManualPlacement { id: "b".into(), x: 12.0 + 2.0 * kerf, y: kerf, width: 12.0, height: 30.0, rotated: false },
        ];
        let result = validate_manual_placement(&placements, &config);
        assert!(result.valid, "parts separated by kerf should be valid: collisions={:?} oob={:?}", result.collisions, result.out_of_bounds);
    }

    #[test]
    fn test_utilization_calculation() {
        let config = NestingConfig {
            sheet_width: 100.0,
            sheet_length: 100.0,
            kerf: 0.0,
            edge_margin: 0.0,
            allow_rotation: false,
            guillotine_compatible: false,
        };
        let placements = vec![
            ManualPlacement { id: "a".into(), x: 0.0, y: 0.0, width: 50.0, height: 50.0, rotated: false },
        ];
        let result = validate_manual_placement(&placements, &config);
        assert!((result.utilization - 0.25).abs() < 1e-6, "50x50 on 100x100 = 25%");
    }

    #[test]
    fn test_empty_placements() {
        let result = validate_manual_placement(&[], &test_config());
        assert!(result.valid);
        assert_eq!(result.utilization, 0.0);
    }

    #[test]
    fn test_negative_position_out_of_bounds() {
        let placements = vec![
            ManualPlacement { id: "a".into(), x: -1.0, y: 0.0, width: 12.0, height: 30.0, rotated: false },
        ];
        let result = validate_manual_placement(&placements, &test_config());
        assert!(result.out_of_bounds.contains(&"a".to_string()));
    }

    #[test]
    fn test_sheet_layout_from_manual() {
        let config = test_config();
        let placements = vec![
            ManualPlacement { id: "a".into(), x: 0.0, y: 0.0, width: 12.0, height: 30.0, rotated: false },
            ManualPlacement { id: "b".into(), x: 15.0, y: 0.0, width: 12.0, height: 30.0, rotated: true },
        ];
        let layout = sheet_layout_from_manual(&placements, 0, &config);

        assert_eq!(layout.sheet_index, 0);
        assert_eq!(layout.parts.len(), 2);
        assert_eq!(layout.parts[0].id, "a");
        assert!(!layout.parts[0].rotated);
        assert!(layout.parts[1].rotated);
        assert!(layout.waste_area > 0.0);
    }
}
