use cm_core::geometry::{Point2D, Rect};

/// Configuration for the nesting algorithm.
#[derive(Debug, Clone)]
pub struct NestingConfig {
    /// Sheet width (e.g., 48.0 for a 4'x8' sheet cut in half).
    pub sheet_width: f64,
    /// Sheet length (e.g., 96.0 for a 4'x8' sheet).
    pub sheet_length: f64,
    /// Kerf width (material removed by the saw blade / router bit).
    /// Parts are spaced apart by this amount to account for the cut width.
    pub kerf: f64,
    /// Margin from sheet edges (to avoid clamps, edge damage, etc.).
    pub edge_margin: f64,
    /// Whether to allow rotating parts 90 degrees to fit better.
    /// Only valid when grain direction doesn't matter.
    pub allow_rotation: bool,
}

impl Default for NestingConfig {
    fn default() -> Self {
        Self {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.25,   // 1/4" router bit kerf
            edge_margin: 0.5,
            allow_rotation: false,
        }
    }
}

/// A part to be nested, with an ID to track it back to the source.
#[derive(Debug, Clone)]
pub struct NestingPart {
    /// Unique identifier (e.g., "left_side", "shelf_0").
    pub id: String,
    /// Width of the part.
    pub width: f64,
    /// Height of the part.
    pub height: f64,
    /// Whether this part can be rotated 90 degrees.
    pub can_rotate: bool,
}

/// A placed part on a sheet, with its position and whether it was rotated.
#[derive(Debug, Clone)]
pub struct PlacedPart {
    /// The part ID.
    pub id: String,
    /// The rectangle describing the part's position on the sheet.
    pub rect: Rect,
    /// Whether the part was rotated 90 degrees from its original orientation.
    pub rotated: bool,
}

/// Layout of parts on a single sheet.
#[derive(Debug, Clone)]
pub struct SheetLayout {
    /// Sheet index (0-based).
    pub sheet_index: usize,
    /// The sheet dimensions.
    pub sheet_rect: Rect,
    /// Parts placed on this sheet.
    pub parts: Vec<PlacedPart>,
    /// Waste area (sheet area - parts area).
    pub waste_area: f64,
    /// Utilization percentage (parts area / sheet area * 100).
    pub utilization: f64,
}

/// Result of the nesting operation.
#[derive(Debug, Clone)]
pub struct NestingResult {
    /// Layouts for each sheet used.
    pub sheets: Vec<SheetLayout>,
    /// Parts that couldn't be placed (too large for any sheet).
    pub unplaced: Vec<String>,
    /// Total number of sheets used.
    pub sheet_count: usize,
    /// Overall utilization across all sheets.
    pub overall_utilization: f64,
}

/// A shelf in the shelf-based packing algorithm.
/// Each shelf is a horizontal strip across the sheet at a certain Y position.
#[derive(Debug)]
struct Shelf {
    /// Y position of the bottom of this shelf.
    y: f64,
    /// Height of this shelf (determined by the tallest part placed on it).
    height: f64,
    /// Current X position (where the next part would go).
    x_cursor: f64,
}

/// Nest rectangular parts onto sheets using a shelf-based bin packing algorithm.
///
/// Algorithm: "Shelf First Fit Decreasing Height" (FFDH)
/// 1. Sort parts by height (tallest first) for better packing.
/// 2. For each part, try to fit it on an existing shelf.
/// 3. If no shelf fits, start a new shelf.
/// 4. If the sheet is full, start a new sheet.
///
/// This is a well-known heuristic for 2D rectangular bin packing that typically
/// achieves 70-85% utilization for typical cabinet part mixes.
pub fn nest_parts(parts: &[NestingPart], config: &NestingConfig) -> NestingResult {
    let usable_width = config.sheet_width - 2.0 * config.edge_margin;
    let usable_length = config.sheet_length - 2.0 * config.edge_margin;

    // Expand each part to include quantity and sort by height descending
    let mut sorted_parts: Vec<&NestingPart> = parts.iter().collect();
    sorted_parts.sort_by(|a, b| {
        // Sort by height descending (tallest first), then by width descending
        let h_cmp = b.height.partial_cmp(&a.height).unwrap();
        if h_cmp == std::cmp::Ordering::Equal {
            b.width.partial_cmp(&a.width).unwrap()
        } else {
            h_cmp
        }
    });

    let mut sheets: Vec<SheetLayout> = Vec::new();
    let mut sheet_shelves: Vec<Vec<Shelf>> = Vec::new();
    let mut unplaced: Vec<String> = Vec::new();

    for part in &sorted_parts {
        let (part_w, part_h, rotated) = best_orientation(
            part.width, part.height,
            usable_width, usable_length,
            part.can_rotate && config.allow_rotation,
        );

        // Check if part fits on any sheet at all
        if part_w > usable_width || part_h > usable_length {
            unplaced.push(part.id.clone());
            continue;
        }

        let mut placed = false;

        // Try each existing sheet
        for (sheet_idx, shelves) in sheet_shelves.iter_mut().enumerate() {
            if let Some(position) = try_place_on_shelves(
                shelves, part_w, part_h, usable_width, usable_length, config.kerf,
            ) {
                let rect = Rect::new(
                    Point2D::new(
                        config.edge_margin + position.x,
                        config.edge_margin + position.y,
                    ),
                    part_w,
                    part_h,
                );
                sheets[sheet_idx].parts.push(PlacedPart {
                    id: part.id.clone(),
                    rect,
                    rotated,
                });
                placed = true;
                break;
            }
        }

        // If not placed, start a new sheet
        if !placed {
            let mut new_shelves = Vec::new();
            if let Some(position) = try_place_on_shelves(
                &mut new_shelves, part_w, part_h, usable_width, usable_length, config.kerf,
            ) {
                let sheet_idx = sheets.len();
                let rect = Rect::new(
                    Point2D::new(
                        config.edge_margin + position.x,
                        config.edge_margin + position.y,
                    ),
                    part_w,
                    part_h,
                );
                sheets.push(SheetLayout {
                    sheet_index: sheet_idx,
                    sheet_rect: Rect::new(
                        Point2D::origin(),
                        config.sheet_width,
                        config.sheet_length,
                    ),
                    parts: vec![PlacedPart {
                        id: part.id.clone(),
                        rect,
                        rotated,
                    }],
                    waste_area: 0.0,
                    utilization: 0.0,
                });
                sheet_shelves.push(new_shelves);
            } else {
                unplaced.push(part.id.clone());
            }
        }
    }

    // Calculate utilization for each sheet
    let sheet_area = config.sheet_width * config.sheet_length;
    let mut total_parts_area = 0.0;
    let mut total_sheet_area = 0.0;

    for sheet in &mut sheets {
        let parts_area: f64 = sheet.parts.iter().map(|p| p.rect.area()).sum();
        sheet.waste_area = sheet_area - parts_area;
        sheet.utilization = (parts_area / sheet_area) * 100.0;
        total_parts_area += parts_area;
        total_sheet_area += sheet_area;
    }

    let overall_utilization = if total_sheet_area > 0.0 {
        (total_parts_area / total_sheet_area) * 100.0
    } else {
        0.0
    };

    NestingResult {
        sheet_count: sheets.len(),
        sheets,
        unplaced,
        overall_utilization,
    }
}

/// Determine the best orientation for a part (original or rotated).
fn best_orientation(
    width: f64,
    height: f64,
    usable_width: f64,
    usable_length: f64,
    can_rotate: bool,
) -> (f64, f64, bool) {
    // Try original orientation first
    if width <= usable_width && height <= usable_length {
        return (width, height, false);
    }
    // Try rotated if allowed
    if can_rotate && height <= usable_width && width <= usable_length {
        return (height, width, true);
    }
    // Return original even if it doesn't fit (will be caught by caller)
    (width, height, false)
}

/// Try to place a part on existing shelves, or create a new shelf.
/// Returns the position (relative to usable area origin) if successful.
fn try_place_on_shelves(
    shelves: &mut Vec<Shelf>,
    part_w: f64,
    part_h: f64,
    usable_width: f64,
    usable_length: f64,
    kerf: f64,
) -> Option<Point2D> {
    // Try to fit on an existing shelf
    for shelf in shelves.iter_mut() {
        // Part must fit within shelf height and remaining width
        if part_h <= shelf.height && shelf.x_cursor + part_w <= usable_width {
            let position = Point2D::new(shelf.x_cursor, shelf.y);
            shelf.x_cursor += part_w + kerf;
            return Some(position);
        }
    }

    // Create a new shelf
    let shelf_y = if let Some(last) = shelves.last() {
        last.y + last.height + kerf
    } else {
        0.0
    };

    // Check if new shelf fits on the sheet
    if shelf_y + part_h <= usable_length {
        let position = Point2D::new(0.0, shelf_y);
        shelves.push(Shelf {
            y: shelf_y,
            height: part_h,
            x_cursor: part_w + kerf,
        });
        return Some(position);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_part_fits_on_sheet() {
        let parts = vec![NestingPart {
            id: "panel".into(),
            width: 12.0,
            height: 30.0,
            can_rotate: false,
        }];
        let config = NestingConfig::default();
        let result = nest_parts(&parts, &config);

        assert_eq!(result.sheet_count, 1);
        assert_eq!(result.sheets[0].parts.len(), 1);
        assert!(result.unplaced.is_empty());
    }

    #[test]
    fn test_multiple_parts_on_one_sheet() {
        let parts = vec![
            NestingPart { id: "left_side".into(), width: 12.0, height: 30.0, can_rotate: false },
            NestingPart { id: "right_side".into(), width: 12.0, height: 30.0, can_rotate: false },
            NestingPart { id: "top".into(), width: 35.25, height: 12.0, can_rotate: false },
            NestingPart { id: "bottom".into(), width: 35.25, height: 12.0, can_rotate: false },
        ];
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: false,
        };
        let result = nest_parts(&parts, &config);

        assert_eq!(result.sheet_count, 1, "all parts should fit on one sheet");
        assert_eq!(result.sheets[0].parts.len(), 4);
        assert!(result.unplaced.is_empty());
    }

    #[test]
    fn test_parts_overflow_to_second_sheet() {
        // Fill a small sheet and overflow to a second
        let parts = vec![
            NestingPart { id: "a".into(), width: 20.0, height: 20.0, can_rotate: false },
            NestingPart { id: "b".into(), width: 20.0, height: 20.0, can_rotate: false },
            NestingPart { id: "c".into(), width: 20.0, height: 20.0, can_rotate: false },
        ];
        let config = NestingConfig {
            sheet_width: 24.0,
            sheet_length: 24.0,
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: false,
        };
        let result = nest_parts(&parts, &config);

        assert!(result.sheet_count >= 2, "should need at least 2 sheets");
        assert!(result.unplaced.is_empty());
    }

    #[test]
    fn test_oversized_part_is_unplaced() {
        let parts = vec![NestingPart {
            id: "huge".into(),
            width: 100.0,
            height: 100.0,
            can_rotate: false,
        }];
        let config = NestingConfig::default();
        let result = nest_parts(&parts, &config);

        assert_eq!(result.sheet_count, 0);
        assert_eq!(result.unplaced.len(), 1);
    }

    #[test]
    fn test_rotation_helps_fit() {
        // A 40x10 part won't fit on a 30x50 sheet normally, but rotated (10x40) it will
        let parts = vec![NestingPart {
            id: "tall".into(),
            width: 40.0,
            height: 10.0,
            can_rotate: true,
        }];
        let config = NestingConfig {
            sheet_width: 30.0,
            sheet_length: 50.0,
            kerf: 0.0,
            edge_margin: 0.0,
            allow_rotation: true,
        };
        let result = nest_parts(&parts, &config);

        assert_eq!(result.sheet_count, 1);
        assert!(result.sheets[0].parts[0].rotated);
    }

    #[test]
    fn test_bookshelf_parts_nesting_no_rotation() {
        // Simulate a real bookshelf: 2 sides, top, bottom, 2 shelves, back.
        // Without rotation, the wide parts (35.25") can't share shelves
        // on a 47" usable width, so the shelf algorithm needs 2 sheets.
        let parts = vec![
            NestingPart { id: "left_side".into(), width: 12.0, height: 30.0, can_rotate: false },
            NestingPart { id: "right_side".into(), width: 12.0, height: 30.0, can_rotate: false },
            NestingPart { id: "top".into(), width: 35.25, height: 12.0, can_rotate: false },
            NestingPart { id: "bottom".into(), width: 35.25, height: 12.0, can_rotate: false },
            NestingPart { id: "shelf_1".into(), width: 35.25, height: 11.75, can_rotate: false },
            NestingPart { id: "shelf_2".into(), width: 35.25, height: 11.75, can_rotate: false },
            NestingPart { id: "back".into(), width: 35.25, height: 29.25, can_rotate: false },
        ];
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: false,
        };
        let result = nest_parts(&parts, &config);

        // All parts placed, reasonable sheet count
        assert!(result.unplaced.is_empty(), "all parts should be placed");
        assert!(result.sheet_count <= 2, "should need at most 2 sheets");

        let total_placed: usize = result.sheets.iter().map(|s| s.parts.len()).sum();
        assert_eq!(total_placed, 7);
        assert!(result.overall_utilization > 30.0, "utilization should be reasonable");

        // Verify no parts overlap on any sheet
        for sheet in &result.sheets {
            let placed = &sheet.parts;
            for i in 0..placed.len() {
                for j in (i + 1)..placed.len() {
                    assert!(
                        !rects_overlap(&placed[i].rect, &placed[j].rect),
                        "Parts '{}' and '{}' overlap on sheet {}!",
                        placed[i].id, placed[j].id, sheet.sheet_index,
                    );
                }
            }
        }
    }

    #[test]
    fn test_bookshelf_parts_nesting_with_rotation() {
        // With rotation allowed, the sides (12x30) can be rotated to (30x12)
        // which allows more efficient packing on shelves.
        let parts = vec![
            NestingPart { id: "left_side".into(), width: 12.0, height: 30.0, can_rotate: true },
            NestingPart { id: "right_side".into(), width: 12.0, height: 30.0, can_rotate: true },
            NestingPart { id: "top".into(), width: 35.25, height: 12.0, can_rotate: true },
            NestingPart { id: "bottom".into(), width: 35.25, height: 12.0, can_rotate: true },
            NestingPart { id: "shelf_1".into(), width: 35.25, height: 11.75, can_rotate: true },
            NestingPart { id: "shelf_2".into(), width: 35.25, height: 11.75, can_rotate: true },
            NestingPart { id: "back".into(), width: 35.25, height: 29.25, can_rotate: true },
        ];
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: true,
        };
        let result = nest_parts(&parts, &config);

        assert!(result.unplaced.is_empty(), "all parts should be placed");

        let total_placed: usize = result.sheets.iter().map(|s| s.parts.len()).sum();
        assert_eq!(total_placed, 7);
    }

    #[test]
    fn test_utilization_calculation() {
        let parts = vec![NestingPart {
            id: "panel".into(),
            width: 24.0,
            height: 48.0,
            can_rotate: false,
        }];
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.0,
            edge_margin: 0.0,
            allow_rotation: false,
        };
        let result = nest_parts(&parts, &config);

        // 24*48 / (48*96) = 1152/4608 = 25%
        assert!(
            (result.overall_utilization - 25.0).abs() < 0.1,
            "utilization should be 25%, got {:.1}%",
            result.overall_utilization,
        );
    }

    #[test]
    fn test_kerf_prevents_overlap() {
        // Two 23" wide parts on a 48" sheet with 0.5" margins
        // Usable width = 47". Without kerf: 23+23=46 fits. With 0.25" kerf: 23+0.25+23=46.25 fits.
        let parts = vec![
            NestingPart { id: "a".into(), width: 23.0, height: 10.0, can_rotate: false },
            NestingPart { id: "b".into(), width: 23.0, height: 10.0, can_rotate: false },
        ];
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: false,
        };
        let result = nest_parts(&parts, &config);

        assert_eq!(result.sheet_count, 1);
        assert_eq!(result.sheets[0].parts.len(), 2);

        // Verify gap between parts >= kerf
        let a = &result.sheets[0].parts[0].rect;
        let b = &result.sheets[0].parts[1].rect;
        if a.max_x() < b.min_x() {
            assert!(b.min_x() - a.max_x() >= config.kerf - 1e-10);
        }
    }

    /// Helper: check if two rectangles overlap (excluding touching edges).
    fn rects_overlap(a: &Rect, b: &Rect) -> bool {
        let eps = 1e-6;
        a.min_x() < b.max_x() - eps
            && a.max_x() > b.min_x() + eps
            && a.min_y() < b.max_y() - eps
            && a.max_y() > b.min_y() + eps
    }
}
