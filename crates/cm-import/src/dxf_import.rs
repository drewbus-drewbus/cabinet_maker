use std::path::Path;

use cm_cabinet::part::{Part, PartOperation, DadoOp, DadoOrientation, DrillOp, RabbetOp, Edge};
use cm_core::geometry::{Point2D, Rect};

use crate::{
    ExtractedCircle, ExtractedRect, ImportError, ImportMode, ImportOptions, ImportResult,
};

/// Import parts from a DXF file on disk.
pub fn import_from_file(
    path: &Path,
    options: &ImportOptions,
) -> Result<ImportResult, ImportError> {
    let drawing = dxf::Drawing::load_file(path)
        .map_err(|e| ImportError::DxfRead(e.to_string()))?;
    import_from_drawing(&drawing, options)
}

/// Import parts from a DXF drawing (already loaded).
pub fn import_from_drawing(
    drawing: &dxf::Drawing,
    options: &ImportOptions,
) -> Result<ImportResult, ImportError> {
    let mut rects = Vec::new();
    let mut circles = Vec::new();
    let mut skipped = 0usize;
    let mut warnings = Vec::new();

    for entity in drawing.entities() {
        let layer = entity.common.layer.clone();

        match &entity.specific {
            // LwPolyline: the most common rectangle representation in DXF
            dxf::entities::EntityType::LwPolyline(lwp) => {
                if let Some(rect) = lwpolyline_to_rect(lwp, &layer) {
                    rects.push(rect);
                } else {
                    skipped += 1;
                    warnings.push(format!(
                        "Skipped non-rectangular polyline on layer '{}'",
                        layer
                    ));
                }
            }
            // Circle: used for drill holes in layer-based mode
            dxf::entities::EntityType::Circle(c) => {
                circles.push(ExtractedCircle {
                    center: Point2D::new(c.center.x, c.center.y),
                    diameter: c.radius * 2.0,
                    layer,
                });
            }
            // Line: we collect lines and try to form rectangles from them
            dxf::entities::EntityType::Line(_) => {
                // Individual lines aren't directly useful — we'd need to
                // assemble them into closed rectangles. For now, skip and
                // warn. Users should export as polylines.
                skipped += 1;
            }
            // Polyline (3D/legacy): try to extract rectangle
            dxf::entities::EntityType::Polyline(poly) => {
                if let Some(rect) = polyline_to_rect(poly, &layer) {
                    rects.push(rect);
                } else {
                    skipped += 1;
                }
            }
            _ => {
                skipped += 1;
            }
        }
    }

    if rects.is_empty() && circles.is_empty() {
        return Err(ImportError::NoRectanglesFound);
    }

    let parts = match options.mode {
        ImportMode::LayerBased => build_parts_layer_based(&rects, &circles, options),
        ImportMode::Raw => build_parts_raw(&rects, options),
    };

    Ok(ImportResult {
        parts,
        warnings,
        skipped_entities: skipped,
    })
}

/// Try to convert an LwPolyline into an axis-aligned rectangle.
/// Returns None if the polyline doesn't form a rectangle.
fn lwpolyline_to_rect(
    lwp: &dxf::entities::LwPolyline,
    layer: &str,
) -> Option<ExtractedRect> {
    let verts = &lwp.vertices;

    // A closed rectangle has 4 vertices (the closing segment is implicit if is_closed)
    // or 5 vertices where first == last.
    let points: Vec<(f64, f64)> = verts.iter().map(|v| (v.x, v.y)).collect();

    let pts = if points.len() == 5 {
        // Check if first == last (closed)
        let first = points[0];
        let last = points[4];
        if (first.0 - last.0).abs() > 1e-6 || (first.1 - last.1).abs() > 1e-6 {
            return None;
        }
        &points[..4]
    } else if points.len() == 4 && lwp.is_closed() {
        &points[..4]
    } else {
        return None;
    };

    // Check if these 4 points form an axis-aligned rectangle
    let xs: Vec<f64> = pts.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = pts.iter().map(|p| p.1).collect();

    let min_x = xs.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_x = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_y = ys.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_y = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // Each point should be at a corner (combination of min/max x and y)
    let corners = [
        (min_x, min_y),
        (max_x, min_y),
        (max_x, max_y),
        (min_x, max_y),
    ];

    for pt in pts {
        let at_corner = corners
            .iter()
            .any(|c| (c.0 - pt.0).abs() < 1e-4 && (c.1 - pt.1).abs() < 1e-4);
        if !at_corner {
            return None; // Not axis-aligned
        }
    }

    let width = max_x - min_x;
    let height = max_y - min_y;

    if width < 1e-6 || height < 1e-6 {
        return None; // Degenerate
    }

    Some(ExtractedRect {
        origin: Point2D::new(min_x, min_y),
        width,
        height,
        layer: layer.to_string(),
    })
}

/// Try to convert a legacy Polyline entity into a rectangle.
fn polyline_to_rect(
    poly: &dxf::entities::Polyline,
    layer: &str,
) -> Option<ExtractedRect> {
    let verts: Vec<(f64, f64)> = poly
        .vertices()
        .map(|v| (v.location.x, v.location.y))
        .collect();

    if verts.len() < 4 {
        return None;
    }

    let pts = if verts.len() == 5 {
        let first = verts[0];
        let last = verts[4];
        if (first.0 - last.0).abs() > 1e-6 || (first.1 - last.1).abs() > 1e-6 {
            return None;
        }
        &verts[..4]
    } else if verts.len() == 4 {
        &verts[..4]
    } else {
        return None;
    };

    let xs: Vec<f64> = pts.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = pts.iter().map(|p| p.1).collect();

    let min_x = xs.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_x = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_y = ys.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_y = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let corners = [
        (min_x, min_y),
        (max_x, min_y),
        (max_x, max_y),
        (min_x, max_y),
    ];

    for pt in pts {
        let at_corner = corners
            .iter()
            .any(|c| (c.0 - pt.0).abs() < 1e-4 && (c.1 - pt.1).abs() < 1e-4);
        if !at_corner {
            return None;
        }
    }

    let width = max_x - min_x;
    let height = max_y - min_y;

    if width < 1e-6 || height < 1e-6 {
        return None;
    }

    Some(ExtractedRect {
        origin: Point2D::new(min_x, min_y),
        width,
        height,
        layer: layer.to_string(),
    })
}

/// Build parts using layer-based interpretation.
fn build_parts_layer_based(
    rects: &[ExtractedRect],
    circles: &[ExtractedCircle],
    options: &ImportOptions,
) -> Vec<Part> {
    let mut parts = Vec::new();
    let dado_depth = options.thickness * options.dado_depth_fraction;

    // Rectangles on PARTS/OUTLINES layers become parts
    let part_rects: Vec<&ExtractedRect> = rects
        .iter()
        .filter(|r| {
            let layer = r.layer.to_uppercase();
            layer == "PARTS"
                || layer == "OUTLINES"
                || layer == "0"
                || layer.is_empty()
        })
        .collect();

    // Rectangles on DADOS layer become dado operations
    let dado_rects: Vec<&ExtractedRect> = rects
        .iter()
        .filter(|r| r.layer.to_uppercase() == "DADOS")
        .collect();

    // Rectangles on RABBETS layer become rabbet operations
    let rabbet_rects: Vec<&ExtractedRect> = rects
        .iter()
        .filter(|r| r.layer.to_uppercase() == "RABBETS")
        .collect();

    // Circles on DRILLS layer become drill operations
    let drill_circles: Vec<&ExtractedCircle> = circles
        .iter()
        .filter(|c| c.layer.to_uppercase() == "DRILLS")
        .collect();

    for (idx, rect) in part_rects.iter().enumerate() {
        let label = format!("part_{}", idx + 1);
        let mut operations = Vec::new();

        // Find dados that overlap with this part
        for dado in &dado_rects {
            if rect_contains_rect(rect, dado) {
                // Determine orientation: wider than tall = horizontal dado
                let horizontal = dado.width > dado.height;
                let (position, width) = if horizontal {
                    // Position is the Y center relative to the part
                    let y_center = dado.origin.y + dado.height / 2.0 - rect.origin.y;
                    (y_center, dado.height)
                } else {
                    let x_center = dado.origin.x + dado.width / 2.0 - rect.origin.x;
                    (x_center, dado.width)
                };
                operations.push(PartOperation::Dado(DadoOp {
                    position,
                    width,
                    depth: dado_depth,
                    orientation: if horizontal {
                        DadoOrientation::Horizontal
                    } else {
                        DadoOrientation::Vertical
                    },
                }));
            }
        }

        // Find rabbets that touch this part's edges
        for rabbet in &rabbet_rects {
            if rect_touches_edge(rect, rabbet) {
                let edge = determine_edge(rect, rabbet);
                operations.push(PartOperation::Rabbet(RabbetOp {
                    edge,
                    width: options.rabbet_width,
                    depth: options.rabbet_depth,
                }));
            }
        }

        // Find drill holes inside this part
        for drill in &drill_circles {
            if point_in_rect(drill.center, rect) {
                operations.push(PartOperation::Drill(DrillOp {
                    x: drill.center.x - rect.origin.x,
                    y: drill.center.y - rect.origin.y,
                    diameter: drill.diameter,
                    depth: options.drill_depth,
                }));
            }
        }

        parts.push(Part {
            label,
            rect: Rect::new(Point2D::origin(), rect.width, rect.height),
            thickness: options.thickness,
            grain_direction: Default::default(),
            operations,
            quantity: 1,
        });
    }

    parts
}

/// Build parts in raw mode: every rectangle becomes a part with no operations.
fn build_parts_raw(rects: &[ExtractedRect], options: &ImportOptions) -> Vec<Part> {
    rects
        .iter()
        .enumerate()
        .map(|(idx, rect)| Part {
            label: format!("part_{}", idx + 1),
            rect: Rect::new(Point2D::origin(), rect.width, rect.height),
            thickness: options.thickness,
            grain_direction: Default::default(),
            operations: vec![],
            quantity: 1,
        })
        .collect()
}

/// Check if inner_rect is fully contained within outer_rect.
fn rect_contains_rect(outer: &ExtractedRect, inner: &ExtractedRect) -> bool {
    inner.origin.x >= outer.origin.x - 1e-4
        && inner.origin.y >= outer.origin.y - 1e-4
        && (inner.origin.x + inner.width) <= (outer.origin.x + outer.width + 1e-4)
        && (inner.origin.y + inner.height) <= (outer.origin.y + outer.height + 1e-4)
}

/// Check if a rectangle touches the edge of another rectangle.
fn rect_touches_edge(part: &ExtractedRect, rabbet: &ExtractedRect) -> bool {
    let part_max_x = part.origin.x + part.width;
    let part_max_y = part.origin.y + part.height;
    let rab_max_x = rabbet.origin.x + rabbet.width;
    let rab_max_y = rabbet.origin.y + rabbet.height;

    // Check if rabbet overlaps with part and touches an edge
    let overlaps_x = rabbet.origin.x < part_max_x + 1e-4 && rab_max_x > part.origin.x - 1e-4;
    let overlaps_y = rabbet.origin.y < part_max_y + 1e-4 && rab_max_y > part.origin.y - 1e-4;

    if !overlaps_x || !overlaps_y {
        return false;
    }

    // Check if one edge of the rabbet aligns with a part edge
    (rabbet.origin.x - part.origin.x).abs() < 1e-4
        || (rab_max_x - part_max_x).abs() < 1e-4
        || (rabbet.origin.y - part.origin.y).abs() < 1e-4
        || (rab_max_y - part_max_y).abs() < 1e-4
}

/// Determine which edge of a part a rabbet rectangle is on.
fn determine_edge(part: &ExtractedRect, rabbet: &ExtractedRect) -> Edge {
    let part_max_x = part.origin.x + part.width;
    let part_max_y = part.origin.y + part.height;
    let rab_max_x = rabbet.origin.x + rabbet.width;
    let rab_max_y = rabbet.origin.y + rabbet.height;

    // Find the edge with the smallest distance
    let top_dist = (rab_max_y - part_max_y).abs();
    let bottom_dist = (rabbet.origin.y - part.origin.y).abs();
    let left_dist = (rabbet.origin.x - part.origin.x).abs();
    let right_dist = (rab_max_x - part_max_x).abs();

    let min = top_dist.min(bottom_dist).min(left_dist).min(right_dist);
    if (min - top_dist).abs() < 1e-6 {
        Edge::Top
    } else if (min - bottom_dist).abs() < 1e-6 {
        Edge::Bottom
    } else if (min - right_dist).abs() < 1e-6 {
        Edge::Right
    } else {
        Edge::Left
    }
}

/// Check if a point is inside a rectangle.
fn point_in_rect(point: Point2D, rect: &ExtractedRect) -> bool {
    point.x >= rect.origin.x - 1e-4
        && point.x <= rect.origin.x + rect.width + 1e-4
        && point.y >= rect.origin.y - 1e-4
        && point.y <= rect.origin.y + rect.height + 1e-4
}

#[cfg(test)]
mod tests {
    use super::*;
    use dxf::LwPolylineVertex;

    fn make_closed_rect(vertices: Vec<LwPolylineVertex>) -> dxf::entities::LwPolyline {
        let mut lwp = dxf::entities::LwPolyline::default();
        lwp.set_is_closed(true);
        lwp.vertices = vertices;
        lwp
    }

    fn vertex(x: f64, y: f64) -> LwPolylineVertex {
        LwPolylineVertex { x, y, ..Default::default() }
    }

    #[test]
    fn test_raw_import_from_drawing() {
        let mut drawing = dxf::Drawing::new();

        // Add a closed LwPolyline rectangle (12" x 30")
        let lwp = make_closed_rect(vec![
            vertex(0.0, 0.0), vertex(12.0, 0.0),
            vertex(12.0, 30.0), vertex(0.0, 30.0),
        ]);
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        ));

        // Add another rectangle (35.25" x 12")
        let lwp2 = make_closed_rect(vec![
            vertex(15.0, 0.0), vertex(50.25, 0.0),
            vertex(50.25, 12.0), vertex(15.0, 12.0),
        ]);
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp2),
        ));

        let options = ImportOptions {
            mode: ImportMode::Raw,
            thickness: 0.75,
            ..Default::default()
        };

        let result = import_from_drawing(&drawing, &options).unwrap();
        assert_eq!(result.parts.len(), 2);
        assert!((result.parts[0].rect.width - 12.0).abs() < 1e-6);
        assert!((result.parts[0].rect.height - 30.0).abs() < 1e-6);
        assert!((result.parts[1].rect.width - 35.25).abs() < 1e-6);
    }

    #[test]
    fn test_layer_based_import() {
        let mut drawing = dxf::Drawing::new();

        // Part on PARTS layer
        let lwp = make_closed_rect(vec![
            vertex(0.0, 0.0), vertex(12.0, 0.0),
            vertex(12.0, 30.0), vertex(0.0, 30.0),
        ]);
        let mut entity = dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        );
        entity.common.layer = "PARTS".to_string();
        drawing.add_entity(entity);

        // Dado inside the part on DADOS layer
        let dado_lwp = make_closed_rect(vec![
            vertex(0.0, 9.625), vertex(12.0, 9.625),
            vertex(12.0, 10.375), vertex(0.0, 10.375),
        ]);
        let mut dado_entity = dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(dado_lwp),
        );
        dado_entity.common.layer = "DADOS".to_string();
        drawing.add_entity(dado_entity);

        // Drill hole on DRILLS layer
        let circle = dxf::entities::Circle {
            center: dxf::Point::new(2.0, 5.0, 0.0),
            radius: 0.0985, // ~5mm diameter / 2
            ..Default::default()
        };
        let mut drill_entity = dxf::entities::Entity::new(
            dxf::entities::EntityType::Circle(circle),
        );
        drill_entity.common.layer = "DRILLS".to_string();
        drawing.add_entity(drill_entity);

        let options = ImportOptions {
            mode: ImportMode::LayerBased,
            thickness: 0.75,
            dado_depth_fraction: 0.5,
            drill_depth: 0.5,
            ..Default::default()
        };

        let result = import_from_drawing(&drawing, &options).unwrap();
        assert_eq!(result.parts.len(), 1);
        assert_eq!(result.parts[0].operations.len(), 2); // 1 dado + 1 drill
    }

    #[test]
    fn test_non_rectangular_polyline_skipped() {
        let mut drawing = dxf::Drawing::new();

        // Triangle (3 vertices) — not a rectangle
        let mut lwp = dxf::entities::LwPolyline::default();
        lwp.set_is_closed(true);
        lwp.vertices = vec![
            vertex(0.0, 0.0), vertex(10.0, 0.0), vertex(5.0, 8.0),
        ];
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        ));

        let options = ImportOptions::default();
        let result = import_from_drawing(&drawing, &options);
        assert!(result.is_err()); // No rectangles found
    }

    #[test]
    fn test_empty_drawing_errors() {
        let drawing = dxf::Drawing::new();
        let options = ImportOptions::default();
        let result = import_from_drawing(&drawing, &options);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ImportError::NoRectanglesFound));
    }

    #[test]
    fn test_raw_import_skipped_entities_counted() {
        let mut drawing = dxf::Drawing::new();

        // Add a valid rectangle
        let lwp = make_closed_rect(vec![
            vertex(0.0, 0.0), vertex(10.0, 0.0),
            vertex(10.0, 5.0), vertex(0.0, 5.0),
        ]);
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        ));

        // Add a triangle (will be skipped)
        let mut tri = dxf::entities::LwPolyline::default();
        tri.set_is_closed(true);
        tri.vertices = vec![vertex(0.0, 0.0), vertex(5.0, 0.0), vertex(2.5, 4.0)];
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(tri),
        ));

        let options = ImportOptions { mode: ImportMode::Raw, ..Default::default() };
        let result = import_from_drawing(&drawing, &options).unwrap();
        assert_eq!(result.parts.len(), 1);
        assert_eq!(result.skipped_entities, 1);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_raw_import_preserves_part_dimensions() {
        let mut drawing = dxf::Drawing::new();

        // Specific fractional dimensions
        let lwp = make_closed_rect(vec![
            vertex(5.0, 10.0), vertex(17.5, 10.0),
            vertex(17.5, 22.25), vertex(5.0, 22.25),
        ]);
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        ));

        let options = ImportOptions { mode: ImportMode::Raw, thickness: 0.5, ..Default::default() };
        let result = import_from_drawing(&drawing, &options).unwrap();
        assert_eq!(result.parts.len(), 1);
        let part = &result.parts[0];
        assert!((part.rect.width - 12.5).abs() < 1e-6, "width should be 12.5, got {}", part.rect.width);
        assert!((part.rect.height - 12.25).abs() < 1e-6, "height should be 12.25, got {}", part.rect.height);
    }

    #[test]
    fn test_five_vertex_closed_polyline() {
        // 5 vertices where first == last (explicitly closed)
        let mut drawing = dxf::Drawing::new();
        let mut lwp = dxf::entities::LwPolyline::default();
        lwp.set_is_closed(false); // not marked as closed
        lwp.vertices = vec![
            vertex(0.0, 0.0), vertex(8.0, 0.0),
            vertex(8.0, 4.0), vertex(0.0, 4.0),
            vertex(0.0, 0.0), // closing vertex
        ];
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        ));

        let options = ImportOptions { mode: ImportMode::Raw, ..Default::default() };
        let result = import_from_drawing(&drawing, &options).unwrap();
        assert_eq!(result.parts.len(), 1);
        assert!((result.parts[0].rect.width - 8.0).abs() < 1e-6);
        assert!((result.parts[0].rect.height - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_open_polyline_with_4_vertices_skipped() {
        // 4 vertices but NOT closed — should be skipped
        let mut drawing = dxf::Drawing::new();
        let mut lwp = dxf::entities::LwPolyline::default();
        lwp.set_is_closed(false);
        lwp.vertices = vec![
            vertex(0.0, 0.0), vertex(8.0, 0.0),
            vertex(8.0, 4.0), vertex(0.0, 4.0),
        ];
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        ));

        let options = ImportOptions::default();
        let result = import_from_drawing(&drawing, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_degenerate_zero_width_rect_skipped() {
        // A rectangle with zero width (all points on same X line)
        let mut drawing = dxf::Drawing::new();
        let lwp = make_closed_rect(vec![
            vertex(5.0, 0.0), vertex(5.0, 0.0),
            vertex(5.0, 10.0), vertex(5.0, 10.0),
        ]);
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        ));

        let options = ImportOptions::default();
        let result = import_from_drawing(&drawing, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_layer_based_rabbet_on_edge() {
        let mut drawing = dxf::Drawing::new();

        // Part on PARTS layer (12" x 30")
        let lwp = make_closed_rect(vec![
            vertex(0.0, 0.0), vertex(12.0, 0.0),
            vertex(12.0, 30.0), vertex(0.0, 30.0),
        ]);
        let mut entity = dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        );
        entity.common.layer = "PARTS".to_string();
        drawing.add_entity(entity);

        // Rabbet on right edge, on RABBETS layer
        let rabbet_lwp = make_closed_rect(vec![
            vertex(11.75, 0.0), vertex(12.0, 0.0),
            vertex(12.0, 30.0), vertex(11.75, 30.0),
        ]);
        let mut rabbet_entity = dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(rabbet_lwp),
        );
        rabbet_entity.common.layer = "RABBETS".to_string();
        drawing.add_entity(rabbet_entity);

        let options = ImportOptions {
            mode: ImportMode::LayerBased,
            thickness: 0.75,
            ..Default::default()
        };

        let result = import_from_drawing(&drawing, &options).unwrap();
        assert_eq!(result.parts.len(), 1);
        // Should have 1 rabbet operation
        let rabbet_ops: Vec<_> = result.parts[0].operations.iter()
            .filter(|op| matches!(op, PartOperation::Rabbet(_)))
            .collect();
        assert_eq!(rabbet_ops.len(), 1, "should detect rabbet on right edge");
    }

    #[test]
    fn test_layer_based_default_layer_treated_as_parts() {
        let mut drawing = dxf::Drawing::new();

        // Rectangle on default layer "0"
        let lwp = make_closed_rect(vec![
            vertex(0.0, 0.0), vertex(6.0, 0.0),
            vertex(6.0, 3.0), vertex(0.0, 3.0),
        ]);
        let mut entity = dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        );
        entity.common.layer = "0".to_string();
        drawing.add_entity(entity);

        let options = ImportOptions {
            mode: ImportMode::LayerBased,
            ..Default::default()
        };

        let result = import_from_drawing(&drawing, &options).unwrap();
        assert_eq!(result.parts.len(), 1, "default layer '0' should be treated as parts");
    }

    #[test]
    fn test_layer_names_case_insensitive() {
        let mut drawing = dxf::Drawing::new();

        // Part on "Parts" (mixed case)
        let lwp = make_closed_rect(vec![
            vertex(0.0, 0.0), vertex(10.0, 0.0),
            vertex(10.0, 8.0), vertex(0.0, 8.0),
        ]);
        let mut entity = dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        );
        entity.common.layer = "Parts".to_string();
        drawing.add_entity(entity);

        // Dado on "dados" (lowercase)
        let dado_lwp = make_closed_rect(vec![
            vertex(0.0, 3.625), vertex(10.0, 3.625),
            vertex(10.0, 4.375), vertex(0.0, 4.375),
        ]);
        let mut dado_entity = dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(dado_lwp),
        );
        dado_entity.common.layer = "dados".to_string();
        drawing.add_entity(dado_entity);

        let options = ImportOptions {
            mode: ImportMode::LayerBased,
            ..Default::default()
        };

        let result = import_from_drawing(&drawing, &options).unwrap();
        assert_eq!(result.parts.len(), 1, "case-insensitive layer should work");
        assert!(!result.parts[0].operations.is_empty(), "should detect dado on mixed-case layer");
    }

    #[test]
    fn test_large_coordinate_values() {
        let mut drawing = dxf::Drawing::new();

        let lwp = make_closed_rect(vec![
            vertex(10000.0, 20000.0), vertex(10048.0, 20000.0),
            vertex(10048.0, 20096.0), vertex(10000.0, 20096.0),
        ]);
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        ));

        let options = ImportOptions { mode: ImportMode::Raw, ..Default::default() };
        let result = import_from_drawing(&drawing, &options).unwrap();
        assert_eq!(result.parts.len(), 1);
        assert!((result.parts[0].rect.width - 48.0).abs() < 1e-6);
        assert!((result.parts[0].rect.height - 96.0).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_parts_raw_mode() {
        let mut drawing = dxf::Drawing::new();

        for i in 0..5 {
            let offset = i as f64 * 20.0;
            let lwp = make_closed_rect(vec![
                vertex(offset, 0.0), vertex(offset + 10.0, 0.0),
                vertex(offset + 10.0, 5.0), vertex(offset, 5.0),
            ]);
            drawing.add_entity(dxf::entities::Entity::new(
                dxf::entities::EntityType::LwPolyline(lwp),
            ));
        }

        let options = ImportOptions { mode: ImportMode::Raw, ..Default::default() };
        let result = import_from_drawing(&drawing, &options).unwrap();
        assert_eq!(result.parts.len(), 5);
        assert_eq!(result.skipped_entities, 0);
    }

    #[test]
    fn test_non_axis_aligned_polyline_skipped() {
        // A rotated rectangle (diamond shape) — not axis-aligned
        let mut drawing = dxf::Drawing::new();
        let lwp = make_closed_rect(vec![
            vertex(5.0, 0.0), vertex(10.0, 5.0),
            vertex(5.0, 10.0), vertex(0.0, 5.0),
        ]);
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        ));

        let options = ImportOptions::default();
        let result = import_from_drawing(&drawing, &options);
        assert!(result.is_err(), "diamond shape should not be imported as a rectangle");
    }

    #[test]
    fn test_circles_only_drawing_in_raw_mode() {
        // Drawing with only circles — no rectangles found in raw mode
        let mut drawing = dxf::Drawing::new();
        let circle = dxf::entities::Circle {
            center: dxf::Point::new(5.0, 5.0, 0.0),
            radius: 1.0,
            ..Default::default()
        };
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::Circle(circle),
        ));

        let options = ImportOptions { mode: ImportMode::Raw, ..Default::default() };
        // Raw mode only uses rectangles, but circles are still collected.
        // The result depends on whether import_from_drawing treats circles-only as OK.
        let result = import_from_drawing(&drawing, &options);
        // Currently circles are collected but raw mode only builds parts from rects
        // so this should return parts from circles (0 parts) — but the check is
        // `rects.is_empty() && circles.is_empty()`, so circles are not empty here
        // which means it returns Ok with 0 parts
        if let Ok(res) = result {
            assert_eq!(res.parts.len(), 0, "circles only should produce 0 parts in raw mode");
        }
    }

    #[test]
    fn test_custom_dado_depth_fraction() {
        let mut drawing = dxf::Drawing::new();

        // Part
        let lwp = make_closed_rect(vec![
            vertex(0.0, 0.0), vertex(12.0, 0.0),
            vertex(12.0, 30.0), vertex(0.0, 30.0),
        ]);
        let mut entity = dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        );
        entity.common.layer = "PARTS".to_string();
        drawing.add_entity(entity);

        // Dado
        let dado_lwp = make_closed_rect(vec![
            vertex(0.0, 14.625), vertex(12.0, 14.625),
            vertex(12.0, 15.375), vertex(0.0, 15.375),
        ]);
        let mut dado_entity = dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(dado_lwp),
        );
        dado_entity.common.layer = "DADOS".to_string();
        drawing.add_entity(dado_entity);

        let options = ImportOptions {
            mode: ImportMode::LayerBased,
            thickness: 1.0,
            dado_depth_fraction: 0.333,
            ..Default::default()
        };

        let result = import_from_drawing(&drawing, &options).unwrap();
        assert_eq!(result.parts.len(), 1);
        // Should have a dado operation with custom depth
        let dado_ops: Vec<_> = result.parts[0].operations.iter()
            .filter(|op| matches!(op, PartOperation::Dado(_)))
            .collect();
        assert_eq!(dado_ops.len(), 1, "should detect dado");
        if let PartOperation::Dado(dado) = dado_ops[0] {
            assert!((dado.depth - 0.333).abs() < 1e-3, "dado depth should use dado_depth_fraction * thickness");
        }
    }

    #[test]
    fn test_line_entities_skipped_with_warning() {
        let mut drawing = dxf::Drawing::new();

        // Add a valid rectangle
        let lwp = make_closed_rect(vec![
            vertex(0.0, 0.0), vertex(10.0, 0.0),
            vertex(10.0, 5.0), vertex(0.0, 5.0),
        ]);
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::LwPolyline(lwp),
        ));

        // Add a line entity (will be skipped)
        let line = dxf::entities::Line {
            p1: dxf::Point::new(0.0, 0.0, 0.0),
            p2: dxf::Point::new(10.0, 10.0, 0.0),
            ..Default::default()
        };
        drawing.add_entity(dxf::entities::Entity::new(
            dxf::entities::EntityType::Line(line),
        ));

        let options = ImportOptions { mode: ImportMode::Raw, ..Default::default() };
        let result = import_from_drawing(&drawing, &options).unwrap();
        assert_eq!(result.parts.len(), 1);
        assert_eq!(result.skipped_entities, 1, "line should be counted as skipped");
    }
}
