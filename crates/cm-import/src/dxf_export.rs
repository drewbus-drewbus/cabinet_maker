//! DXF export for individual parts and nested sheet layouts.
//!
//! Uses layers to organize entities:
//! - `SHEET` — sheet boundary outline
//! - `PARTS` — part profile outlines
//! - `LABELS` — part label text
//! - `DIMENSIONS` — dimension annotations

use cm_core::geometry::Rect;
use dxf::entities::{Entity, EntityType, LwPolyline, MText};
use dxf::enums::AcadVersion;
use dxf::{Drawing, LwPolylineVertex, Point};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("DXF write error: {0}")]
    Write(String),
}

/// Layer names used in DXF export.
pub mod layers {
    pub const SHEET: &str = "SHEET";
    pub const PARTS: &str = "PARTS";
    pub const LABELS: &str = "LABELS";
    pub const DIMENSIONS: &str = "DIMENSIONS";
}

fn new_drawing() -> Drawing {
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R2000;
    drawing
}

/// Export a single part as a DXF rectangle with label and dimensions.
pub fn export_part_dxf(rect: &Rect, label: &str) -> Result<Vec<u8>, ExportError> {
    let mut drawing = new_drawing();
    add_layers(&mut drawing);

    add_rect_entity(&mut drawing, rect, layers::PARTS);
    add_label(&mut drawing, label, rect.origin.x + rect.width / 2.0, rect.origin.y + rect.height / 2.0);
    add_dimension_text(
        &mut drawing,
        &format!("{:.3}\" x {:.3}\"", rect.width, rect.height),
        rect.origin.x + rect.width / 2.0,
        rect.origin.y - 0.5,
    );

    drawing_to_bytes(&drawing)
}

/// A placed part for sheet export.
pub struct ExportPlacedPart<'a> {
    pub id: &'a str,
    pub rect: &'a Rect,
}

/// Export a nested sheet layout as DXF.
///
/// Draws the sheet outline on SHEET layer, each part on PARTS layer,
/// and part labels on LABELS layer.
pub fn export_sheet_dxf(
    sheet_rect: &Rect,
    placed_parts: &[ExportPlacedPart<'_>],
) -> Result<Vec<u8>, ExportError> {
    let mut drawing = new_drawing();
    add_layers(&mut drawing);

    // Sheet boundary
    add_rect_entity(&mut drawing, sheet_rect, layers::SHEET);

    // Parts
    for part in placed_parts {
        add_rect_entity(&mut drawing, part.rect, layers::PARTS);

        // Label at center of part
        let cx = part.rect.origin.x + part.rect.width / 2.0;
        let cy = part.rect.origin.y + part.rect.height / 2.0;
        add_label(&mut drawing, part.id, cx, cy);

        // Dimension text below part
        add_dimension_text(
            &mut drawing,
            &format!("{:.3}x{:.3}", part.rect.width, part.rect.height),
            cx,
            part.rect.origin.y - 0.3,
        );
    }

    drawing_to_bytes(&drawing)
}

fn add_layers(drawing: &mut Drawing) {
    for name in [layers::SHEET, layers::PARTS, layers::LABELS, layers::DIMENSIONS] {
        let layer = dxf::tables::Layer {
            name: name.to_string(),
            color: match name {
                layers::PARTS => dxf::Color::from_index(1), // red
                layers::LABELS => dxf::Color::from_index(3), // green
                layers::DIMENSIONS => dxf::Color::from_index(5), // blue
                _ => dxf::Color::by_layer(),
            },
            ..Default::default()
        };
        drawing.add_layer(layer);
    }
}

fn add_rect_entity(drawing: &mut Drawing, rect: &Rect, layer: &str) {
    let x0 = rect.origin.x;
    let y0 = rect.origin.y;
    let x1 = x0 + rect.width;
    let y1 = y0 + rect.height;

    let mut lwp = LwPolyline::default();
    lwp.set_is_closed(true);
    lwp.vertices = vec![
        LwPolylineVertex { x: x0, y: y0, ..Default::default() },
        LwPolylineVertex { x: x1, y: y0, ..Default::default() },
        LwPolylineVertex { x: x1, y: y1, ..Default::default() },
        LwPolylineVertex { x: x0, y: y1, ..Default::default() },
    ];

    let mut entity = Entity::new(EntityType::LwPolyline(lwp));
    entity.common.layer = layer.to_string();
    drawing.add_entity(entity);
}

fn add_label(drawing: &mut Drawing, text: &str, x: f64, y: f64) {
    let mtext = MText {
        text: text.to_string(),
        insertion_point: Point::new(x, y, 0.0),
        initial_text_height: 0.5,
        ..Default::default()
    };

    let mut entity = Entity::new(EntityType::MText(mtext));
    entity.common.layer = layers::LABELS.to_string();
    drawing.add_entity(entity);
}

fn add_dimension_text(drawing: &mut Drawing, text: &str, x: f64, y: f64) {
    let mtext = MText {
        text: text.to_string(),
        insertion_point: Point::new(x, y, 0.0),
        initial_text_height: 0.3,
        ..Default::default()
    };

    let mut entity = Entity::new(EntityType::MText(mtext));
    entity.common.layer = layers::DIMENSIONS.to_string();
    drawing.add_entity(entity);
}

fn drawing_to_bytes(drawing: &Drawing) -> Result<Vec<u8>, ExportError> {
    let mut buf = Vec::new();
    drawing
        .save(&mut buf)
        .map_err(|e| ExportError::Write(e.to_string()))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cm_core::geometry::Point2D;

    #[test]
    fn test_export_single_part() {
        let rect = Rect::new(Point2D::new(0.0, 0.0), 36.0, 12.0);
        let bytes = export_part_dxf(&rect, "left_side").unwrap();
        assert!(!bytes.is_empty(), "DXF output should not be empty");

        // Write to file, reload and verify entities
        let tmp = std::env::temp_dir().join("test_single_part.dxf");
        std::fs::write(&tmp, &bytes).unwrap();
        let drawing = Drawing::load_file(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);

        let entities: Vec<_> = drawing.entities().collect();
        // 1 rect + 1 label + 1 dimension text = 3 entities
        assert_eq!(entities.len(), 3, "expected 3 entities, got {}", entities.len());
        assert!(entities.iter().any(|e| matches!(&e.specific, EntityType::LwPolyline(_))));
        assert!(entities.iter().any(|e| matches!(&e.specific, EntityType::MText(_))));
    }

    #[test]
    fn test_export_sheet_layout() {
        let sheet = Rect::new(Point2D::new(0.0, 0.0), 48.0, 96.0);
        let part1_rect = Rect::new(Point2D::new(0.0, 0.0), 12.0, 30.0);
        let part2_rect = Rect::new(Point2D::new(13.0, 0.0), 12.0, 30.0);

        let parts = vec![
            ExportPlacedPart { id: "left_side", rect: &part1_rect },
            ExportPlacedPart { id: "right_side", rect: &part2_rect },
        ];

        let bytes = export_sheet_dxf(&sheet, &parts).unwrap();
        assert!(!bytes.is_empty());

        let tmp = std::env::temp_dir().join("test_sheet_layout.dxf");
        std::fs::write(&tmp, &bytes).unwrap();
        let drawing = Drawing::load_file(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);

        let entities: Vec<_> = drawing.entities().collect();
        // 1 sheet rect + 2 part rects + 2 labels + 2 dim texts = 7
        assert_eq!(entities.len(), 7, "expected 7 entities, got {}", entities.len());
        let lwp_count = entities.iter().filter(|e| matches!(&e.specific, EntityType::LwPolyline(_))).count();
        assert_eq!(lwp_count, 3, "expected 3 LwPolyline (1 sheet + 2 parts)");
    }

    #[test]
    fn test_export_round_trip_via_file() {
        let rect = Rect::new(Point2D::new(5.0, 10.0), 24.5, 30.75);
        let bytes = export_part_dxf(&rect, "shelf").unwrap();

        // Write to temp file and re-import
        let tmp = std::env::temp_dir().join("test_export_round_trip.dxf");
        std::fs::write(&tmp, &bytes).unwrap();
        let drawing = Drawing::load_file(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);

        // Find the LwPolyline and verify its vertices match the rect
        let polylines: Vec<_> = drawing.entities().filter(|e| {
            matches!(&e.specific, EntityType::LwPolyline(_))
        }).collect();
        assert_eq!(polylines.len(), 1, "expected 1 LwPolyline, got {}", polylines.len());

        if let EntityType::LwPolyline(lwp) = &polylines[0].specific {
            assert_eq!(lwp.vertices.len(), 4);
            // Bottom-left
            assert!((lwp.vertices[0].x - 5.0).abs() < 1e-6);
            assert!((lwp.vertices[0].y - 10.0).abs() < 1e-6);
            // Bottom-right
            assert!((lwp.vertices[1].x - 29.5).abs() < 1e-6);
            assert!((lwp.vertices[1].y - 10.0).abs() < 1e-6);
            // Top-right
            assert!((lwp.vertices[2].x - 29.5).abs() < 1e-6);
            assert!((lwp.vertices[2].y - 40.75).abs() < 1e-6);
            // Top-left
            assert!((lwp.vertices[3].x - 5.0).abs() < 1e-6);
            assert!((lwp.vertices[3].y - 40.75).abs() < 1e-6);
        } else {
            panic!("expected LwPolyline");
        }
    }

    #[test]
    fn test_export_layers_present() {
        let rect = Rect::new(Point2D::new(0.0, 0.0), 10.0, 20.0);
        let bytes = export_part_dxf(&rect, "test").unwrap();

        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains(layers::PARTS), "missing PARTS layer");
        assert!(content.contains(layers::LABELS), "missing LABELS layer");
        assert!(content.contains(layers::DIMENSIONS), "missing DIMENSIONS layer");
        assert!(content.contains(layers::SHEET), "missing SHEET layer");
    }

    #[test]
    fn test_export_empty_sheet() {
        let sheet = Rect::new(Point2D::new(0.0, 0.0), 48.0, 96.0);
        let bytes = export_sheet_dxf(&sheet, &[]).unwrap();

        let tmp = std::env::temp_dir().join("test_empty_sheet.dxf");
        std::fs::write(&tmp, &bytes).unwrap();
        let drawing = Drawing::load_file(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);

        let entities: Vec<_> = drawing.entities().collect();
        // Just the sheet rectangle
        assert_eq!(entities.len(), 1);
        assert!(matches!(&entities[0].specific, EntityType::LwPolyline(_)));
    }

    #[test]
    fn test_export_dimension_text() {
        let rect = Rect::new(Point2D::new(0.0, 0.0), 12.0, 30.0);
        let bytes = export_part_dxf(&rect, "test_part").unwrap();

        let tmp = std::env::temp_dir().join("test_dim_text.dxf");
        std::fs::write(&tmp, &bytes).unwrap();
        let drawing = Drawing::load_file(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);

        // Find MText entities and check for dimension text
        let mtexts: Vec<_> = drawing.entities().filter_map(|e| {
            if let EntityType::MText(ref mt) = e.specific {
                Some(mt.text.clone())
            } else {
                None
            }
        }).collect();
        assert!(mtexts.iter().any(|t| t.contains("12.000") && t.contains("30.000")),
            "should have dimension text with 12.000 and 30.000, got: {:?}", mtexts);
    }
}
