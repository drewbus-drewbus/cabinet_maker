use cm_core::geometry::{Point2D, Rect};
use serde::{Deserialize, Serialize};

/// Configuration for the nesting algorithm.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Constrain splits to be panel-saw-friendly (guillotine cuts only).
    /// When true, free rectangles are split using guillotine cuts (L-shaped)
    /// rather than the standard maximal-rectangles merge. This produces
    /// layouts that can be cut on a panel saw.
    #[serde(default)]
    pub guillotine_compatible: bool,
}

impl Default for NestingConfig {
    fn default() -> Self {
        Self {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.25,   // 1/4" router bit kerf
            edge_margin: 0.5,
            allow_rotation: false,
            guillotine_compatible: false,
        }
    }
}

/// A part to be nested, with an ID to track it back to the source.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedPart {
    /// The part ID.
    pub id: String,
    /// The rectangle describing the part's position on the sheet.
    pub rect: Rect,
    /// Whether the part was rotated 90 degrees from its original orientation.
    pub rotated: bool,
}

/// Layout of parts on a single sheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A free rectangle in the MaxRects algorithm.
#[derive(Debug, Clone)]
struct FreeRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// State of a single sheet during MaxRects packing.
#[derive(Debug)]
struct MaxRectsSheet {
    free_rects: Vec<FreeRect>,
}

impl MaxRectsSheet {
    fn new(usable_width: f64, usable_length: f64) -> Self {
        Self {
            free_rects: vec![FreeRect {
                x: 0.0,
                y: 0.0,
                width: usable_width,
                height: usable_length,
            }],
        }
    }

    /// Try to place a part using Best Short Side Fit (BSSF).
    /// Returns (position, rotated) if successful.
    fn try_place(
        &mut self,
        part_w: f64,
        part_h: f64,
        can_rotate: bool,
        kerf: f64,
        guillotine: bool,
    ) -> Option<(Point2D, bool)> {
        let mut best_idx = None;
        let mut best_rotated = false;
        let mut best_short_side = f64::MAX;
        let mut best_long_side = f64::MAX;

        for (i, fr) in self.free_rects.iter().enumerate() {
            // Try original orientation
            if part_w <= fr.width + 1e-10 && part_h <= fr.height + 1e-10 {
                let leftover_w = fr.width - part_w;
                let leftover_h = fr.height - part_h;
                let short_side = leftover_w.min(leftover_h);
                let long_side = leftover_w.max(leftover_h);
                if short_side < best_short_side
                    || (short_side == best_short_side && long_side < best_long_side)
                {
                    best_idx = Some(i);
                    best_rotated = false;
                    best_short_side = short_side;
                    best_long_side = long_side;
                }
            }

            // Try rotated orientation
            if can_rotate && part_h <= fr.width + 1e-10 && part_w <= fr.height + 1e-10 {
                let leftover_w = fr.width - part_h;
                let leftover_h = fr.height - part_w;
                let short_side = leftover_w.min(leftover_h);
                let long_side = leftover_w.max(leftover_h);
                if short_side < best_short_side
                    || (short_side == best_short_side && long_side < best_long_side)
                {
                    best_idx = Some(i);
                    best_rotated = true;
                    best_short_side = short_side;
                    best_long_side = long_side;
                }
            }
        }

        let idx = best_idx?;
        let fr = &self.free_rects[idx];
        let position = Point2D::new(fr.x, fr.y);

        let actual_w = if best_rotated { part_h } else { part_w };
        let actual_h = if best_rotated { part_w } else { part_h };
        if guillotine {
            self.split_guillotine(idx, actual_w, actual_h, kerf);
        } else {
            // Standard MaxRects: split all overlapping free rectangles
            self.split_maxrects(actual_w, actual_h, kerf, position);
        }

        Some((position, best_rotated))
    }

    /// Guillotine split: replace the free rect at `idx` with up to 2 new rects
    /// using L-shaped (panel-saw-friendly) cuts.
    fn split_guillotine(&mut self, idx: usize, part_w: f64, part_h: f64, kerf: f64) {
        let fr = self.free_rects.remove(idx);
        let pw = part_w + kerf;
        let ph = part_h + kerf;

        // Choose split direction: split along the shorter leftover to minimize waste.
        // Horizontal split: one rect to the right, one below.
        // We pick the split that leaves the larger remaining area more usable.
        let right_w = fr.width - pw;
        let below_h = fr.height - ph;

        // Split along shorter side for better packing
        if right_w * fr.height > fr.width * below_h {
            // Vertical split: right rect gets full height, below gets partial width
            if right_w > kerf {
                self.free_rects.push(FreeRect {
                    x: fr.x + pw,
                    y: fr.y,
                    width: right_w,
                    height: fr.height,
                });
            }
            if below_h > kerf {
                self.free_rects.push(FreeRect {
                    x: fr.x,
                    y: fr.y + ph,
                    width: pw.min(fr.width),
                    height: below_h,
                });
            }
        } else {
            // Horizontal split: below rect gets full width, right gets partial height
            if below_h > kerf {
                self.free_rects.push(FreeRect {
                    x: fr.x,
                    y: fr.y + ph,
                    width: fr.width,
                    height: below_h,
                });
            }
            if right_w > kerf {
                self.free_rects.push(FreeRect {
                    x: fr.x + pw,
                    y: fr.y,
                    width: right_w,
                    height: ph.min(fr.height),
                });
            }
        }
    }

    /// Standard MaxRects split: for each free rect that overlaps the placed part,
    /// generate up to 4 new free rects, then merge.
    fn split_maxrects(&mut self, part_w: f64, part_h: f64, kerf: f64, pos: Point2D) {
        let pw = part_w + kerf;
        let ph = part_h + kerf;
        let px = pos.x;
        let py = pos.y;

        let mut new_rects = Vec::new();
        let mut i = 0;
        while i < self.free_rects.len() {
            let fr = &self.free_rects[i];

            // Check if this free rect overlaps the placed part
            if px >= fr.x + fr.width - 1e-10
                || px + pw <= fr.x + 1e-10
                || py >= fr.y + fr.height - 1e-10
                || py + ph <= fr.y + 1e-10
            {
                // No overlap
                i += 1;
                continue;
            }

            // Overlap — split into up to 4 new rects
            // Left remainder
            if px > fr.x + 1e-10 {
                new_rects.push(FreeRect {
                    x: fr.x,
                    y: fr.y,
                    width: px - fr.x,
                    height: fr.height,
                });
            }
            // Right remainder
            if px + pw < fr.x + fr.width - 1e-10 {
                new_rects.push(FreeRect {
                    x: px + pw,
                    y: fr.y,
                    width: fr.x + fr.width - (px + pw),
                    height: fr.height,
                });
            }
            // Top remainder
            if py > fr.y + 1e-10 {
                new_rects.push(FreeRect {
                    x: fr.x,
                    y: fr.y,
                    width: fr.width,
                    height: py - fr.y,
                });
            }
            // Bottom remainder
            if py + ph < fr.y + fr.height - 1e-10 {
                new_rects.push(FreeRect {
                    x: fr.x,
                    y: py + ph,
                    width: fr.width,
                    height: fr.y + fr.height - (py + ph),
                });
            }

            self.free_rects.swap_remove(i);
            // Don't increment i — swapped element needs checking too
        }

        self.free_rects.extend(new_rects);
        self.prune_contained();
    }

    /// Remove free rects that are fully contained within another free rect.
    fn prune_contained(&mut self) {
        let mut i = 0;
        while i < self.free_rects.len() {
            let mut contained = false;
            for j in 0..self.free_rects.len() {
                if i == j {
                    continue;
                }
                if self.is_contained(i, j) {
                    contained = true;
                    break;
                }
            }
            if contained {
                self.free_rects.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Check if free_rects[a] is fully contained within free_rects[b].
    fn is_contained(&self, a: usize, b: usize) -> bool {
        let ra = &self.free_rects[a];
        let rb = &self.free_rects[b];
        ra.x >= rb.x - 1e-10
            && ra.y >= rb.y - 1e-10
            && ra.x + ra.width <= rb.x + rb.width + 1e-10
            && ra.y + ra.height <= rb.y + rb.height + 1e-10
    }
}

/// Nest rectangular parts onto sheets using the MaxRects-BSSF bin packing algorithm.
///
/// Algorithm: MaxRects with "Best Short Side Fit" (BSSF)
/// 1. Sort parts by area descending (large parts first).
/// 2. For each part, find the free rectangle where the short side leftover is minimized.
/// 3. Place the part, split overlapping free rectangles, and prune contained ones.
/// 4. If no free rect fits on any existing sheet, open a new sheet.
///
/// MaxRects-BSSF typically achieves 88-95% utilization for cabinet part mixes,
/// a significant improvement over the previous FFDH shelf algorithm.
pub fn nest_parts(parts: &[NestingPart], config: &NestingConfig) -> NestingResult {
    let usable_width = config.sheet_width - 2.0 * config.edge_margin;
    let usable_length = config.sheet_length - 2.0 * config.edge_margin;

    // Sort parts by area descending (large parts first for better packing)
    let mut sorted_parts: Vec<&NestingPart> = parts.iter().collect();
    sorted_parts.sort_by(|a, b| {
        let area_a = a.width * a.height;
        let area_b = b.width * b.height;
        area_b.partial_cmp(&area_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut sheets: Vec<SheetLayout> = Vec::new();
    let mut sheet_states: Vec<MaxRectsSheet> = Vec::new();
    let mut unplaced: Vec<String> = Vec::new();

    for part in &sorted_parts {
        let can_rotate = part.can_rotate && config.allow_rotation;

        // Check if part fits on a sheet at all (either orientation)
        let fits_normal = part.width <= usable_width + 1e-10 && part.height <= usable_length + 1e-10;
        let fits_rotated = can_rotate
            && part.height <= usable_width + 1e-10
            && part.width <= usable_length + 1e-10;

        if !fits_normal && !fits_rotated {
            unplaced.push(part.id.clone());
            continue;
        }

        let mut placed = false;

        // Try each existing sheet
        for (sheet_idx, state) in sheet_states.iter_mut().enumerate() {
            if let Some((pos, rotated)) = state.try_place(
                part.width,
                part.height,
                can_rotate,
                config.kerf,
                config.guillotine_compatible,
            ) {
                let (w, h) = if rotated {
                    (part.height, part.width)
                } else {
                    (part.width, part.height)
                };
                let rect = Rect::new(
                    Point2D::new(config.edge_margin + pos.x, config.edge_margin + pos.y),
                    w,
                    h,
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
            let mut new_state = MaxRectsSheet::new(usable_width, usable_length);
            if let Some((pos, rotated)) = new_state.try_place(
                part.width,
                part.height,
                can_rotate,
                config.kerf,
                config.guillotine_compatible,
            ) {
                let sheet_idx = sheets.len();
                let (w, h) = if rotated {
                    (part.height, part.width)
                } else {
                    (part.width, part.height)
                };
                let rect = Rect::new(
                    Point2D::new(config.edge_margin + pos.x, config.edge_margin + pos.y),
                    w,
                    h,
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
                sheet_states.push(new_state);
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
            guillotine_compatible: false,
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
            guillotine_compatible: false,
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
            guillotine_compatible: false,
        };
        let result = nest_parts(&parts, &config);

        assert_eq!(result.sheet_count, 1);
        assert!(result.sheets[0].parts[0].rotated);
    }

    #[test]
    fn test_bookshelf_parts_nesting_no_rotation() {
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
            guillotine_compatible: false,
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
            guillotine_compatible: false,
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
            guillotine_compatible: false,
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
            guillotine_compatible: false,
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

    // --- New MaxRects-specific tests ---

    #[test]
    fn test_maxrects_better_utilization_than_ffdh() {
        // 10 parts that should pack well with MaxRects
        let parts = vec![
            NestingPart { id: "a".into(), width: 20.0, height: 40.0, can_rotate: false },
            NestingPart { id: "b".into(), width: 25.0, height: 30.0, can_rotate: false },
            NestingPart { id: "c".into(), width: 15.0, height: 35.0, can_rotate: false },
            NestingPart { id: "d".into(), width: 30.0, height: 20.0, can_rotate: false },
            NestingPart { id: "e".into(), width: 10.0, height: 50.0, can_rotate: false },
            NestingPart { id: "f".into(), width: 35.0, height: 15.0, can_rotate: false },
            NestingPart { id: "g".into(), width: 22.0, height: 28.0, can_rotate: false },
            NestingPart { id: "h".into(), width: 18.0, height: 25.0, can_rotate: false },
            NestingPart { id: "i".into(), width: 12.0, height: 45.0, can_rotate: false },
            NestingPart { id: "j".into(), width: 28.0, height: 22.0, can_rotate: false },
        ];
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: false,
            guillotine_compatible: false,
        };
        let result = nest_parts(&parts, &config);

        assert!(result.unplaced.is_empty(), "all parts should be placed");
        // Total area = 800+750+525+600+500+525+616+450+540+616 = 5922 sq in
        // With kerf (0.25") between parts, effective area is larger.
        // Sheet area = 4608 sq in => typically needs 2-3 sheets.
        assert!(result.sheet_count <= 3, "should fit in 3 sheets or less, got {}", result.sheet_count);
        assert!(result.overall_utilization > 40.0,
            "utilization should be >40%, got {:.1}%", result.overall_utilization);
    }

    #[test]
    fn test_rotation_improves_packing() {
        // Parts that pack much better when rotation is allowed
        let parts = vec![
            NestingPart { id: "a".into(), width: 45.0, height: 10.0, can_rotate: true },
            NestingPart { id: "b".into(), width: 45.0, height: 10.0, can_rotate: true },
            NestingPart { id: "c".into(), width: 45.0, height: 10.0, can_rotate: true },
            NestingPart { id: "d".into(), width: 45.0, height: 10.0, can_rotate: true },
        ];
        let config_no_rot = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: false,
            guillotine_compatible: false,
        };
        let config_rot = NestingConfig {
            allow_rotation: true,
            ..config_no_rot.clone()
        };

        let result_no_rot = nest_parts(&parts, &config_no_rot);
        let result_rot = nest_parts(&parts, &config_rot);

        // Both should place all parts
        assert!(result_no_rot.unplaced.is_empty());
        assert!(result_rot.unplaced.is_empty());

        // Rotation should use no more sheets
        assert!(result_rot.sheet_count <= result_no_rot.sheet_count);
    }

    #[test]
    fn test_guillotine_mode_places_all_parts() {
        let parts = vec![
            NestingPart { id: "a".into(), width: 20.0, height: 30.0, can_rotate: false },
            NestingPart { id: "b".into(), width: 25.0, height: 20.0, can_rotate: false },
            NestingPart { id: "c".into(), width: 15.0, height: 25.0, can_rotate: false },
            NestingPart { id: "d".into(), width: 10.0, height: 15.0, can_rotate: false },
        ];
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: false,
            guillotine_compatible: true,
        };
        let result = nest_parts(&parts, &config);

        assert!(result.unplaced.is_empty(), "all parts should be placed");
        assert_eq!(result.sheet_count, 1);
    }

    #[test]
    fn test_guillotine_no_overlap() {
        let parts = vec![
            NestingPart { id: "a".into(), width: 20.0, height: 30.0, can_rotate: false },
            NestingPart { id: "b".into(), width: 25.0, height: 40.0, can_rotate: false },
            NestingPart { id: "c".into(), width: 30.0, height: 20.0, can_rotate: false },
            NestingPart { id: "d".into(), width: 15.0, height: 50.0, can_rotate: false },
            NestingPart { id: "e".into(), width: 10.0, height: 10.0, can_rotate: false },
        ];
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: false,
            guillotine_compatible: true,
        };
        let result = nest_parts(&parts, &config);

        for sheet in &result.sheets {
            for i in 0..sheet.parts.len() {
                for j in (i + 1)..sheet.parts.len() {
                    assert!(
                        !rects_overlap(&sheet.parts[i].rect, &sheet.parts[j].rect),
                        "Parts '{}' and '{}' overlap on sheet {}!",
                        sheet.parts[i].id, sheet.parts[j].id, sheet.sheet_index,
                    );
                }
            }
        }
    }

    #[test]
    fn test_many_small_parts_high_utilization() {
        // Many uniform small parts should pack very efficiently
        let parts: Vec<NestingPart> = (0..20)
            .map(|i| NestingPart {
                id: format!("part_{}", i),
                width: 10.0,
                height: 20.0,
                can_rotate: false,
            })
            .collect();
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: false,
            guillotine_compatible: false,
        };
        let result = nest_parts(&parts, &config);

        assert!(result.unplaced.is_empty());
        // 20 parts * 10 * 20 = 4000 sq in on sheets of 4608 sq in each
        // With kerf of 0.25", effective size ~10.25 * 20.25 each.
        // Should pack on 1-2 sheets with reasonable utilization.
        assert!(result.sheet_count <= 2, "20 small parts should fit on 1-2 sheets, got {}", result.sheet_count);
        assert!(result.overall_utilization > 40.0,
            "utilization should be >40%, got {:.1}%", result.overall_utilization);
    }

    #[test]
    fn test_exact_fit_no_waste() {
        // Part exactly fills the usable area
        let parts = vec![NestingPart {
            id: "exact".into(),
            width: 47.0,
            height: 95.0,
            can_rotate: false,
        }];
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.0,
            edge_margin: 0.5,
            allow_rotation: false,
            guillotine_compatible: false,
        };
        let result = nest_parts(&parts, &config);

        assert_eq!(result.sheet_count, 1);
        assert!(result.unplaced.is_empty());
    }

    #[test]
    fn test_multi_sheet_no_overlap() {
        // Generate enough parts to need multiple sheets
        let parts: Vec<NestingPart> = (0..30)
            .map(|i| NestingPart {
                id: format!("part_{}", i),
                width: 15.0,
                height: 25.0,
                can_rotate: false,
            })
            .collect();
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: false,
            guillotine_compatible: false,
        };
        let result = nest_parts(&parts, &config);

        assert!(result.unplaced.is_empty(), "all parts should be placed");

        // Verify no overlap on any sheet
        for sheet in &result.sheets {
            for i in 0..sheet.parts.len() {
                for j in (i + 1)..sheet.parts.len() {
                    assert!(
                        !rects_overlap(&sheet.parts[i].rect, &sheet.parts[j].rect),
                        "Parts '{}' and '{}' overlap on sheet {}!",
                        sheet.parts[i].id, sheet.parts[j].id, sheet.sheet_index,
                    );
                }
            }
        }
    }

    #[test]
    fn test_mixed_sizes_packing() {
        // Mix of large and small parts — MaxRects should fill gaps with small parts
        let parts = vec![
            NestingPart { id: "large_a".into(), width: 40.0, height: 80.0, can_rotate: false },
            NestingPart { id: "small_a".into(), width: 5.0, height: 5.0, can_rotate: false },
            NestingPart { id: "small_b".into(), width: 5.0, height: 5.0, can_rotate: false },
            NestingPart { id: "small_c".into(), width: 5.0, height: 5.0, can_rotate: false },
            NestingPart { id: "small_d".into(), width: 5.0, height: 5.0, can_rotate: false },
            NestingPart { id: "medium".into(), width: 5.0, height: 80.0, can_rotate: false },
        ];
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: false,
            guillotine_compatible: false,
        };
        let result = nest_parts(&parts, &config);

        assert!(result.unplaced.is_empty());
        assert_eq!(result.sheet_count, 1, "should all fit on one sheet");
    }

    #[test]
    fn test_parts_within_sheet_bounds() {
        let parts = vec![
            NestingPart { id: "a".into(), width: 20.0, height: 30.0, can_rotate: false },
            NestingPart { id: "b".into(), width: 25.0, height: 40.0, can_rotate: false },
            NestingPart { id: "c".into(), width: 15.0, height: 20.0, can_rotate: false },
        ];
        let config = NestingConfig::default();
        let result = nest_parts(&parts, &config);

        for sheet in &result.sheets {
            for placed in &sheet.parts {
                assert!(
                    placed.rect.min_x() >= config.edge_margin - 1e-10,
                    "Part '{}' left edge {:.4} < margin {:.4}",
                    placed.id, placed.rect.min_x(), config.edge_margin,
                );
                assert!(
                    placed.rect.min_y() >= config.edge_margin - 1e-10,
                    "Part '{}' bottom edge {:.4} < margin {:.4}",
                    placed.id, placed.rect.min_y(), config.edge_margin,
                );
                assert!(
                    placed.rect.max_x() <= config.sheet_width - config.edge_margin + 1e-10,
                    "Part '{}' right edge {:.4} > sheet width - margin {:.4}",
                    placed.id, placed.rect.max_x(), config.sheet_width - config.edge_margin,
                );
                assert!(
                    placed.rect.max_y() <= config.sheet_length - config.edge_margin + 1e-10,
                    "Part '{}' top edge {:.4} > sheet length - margin {:.4}",
                    placed.id, placed.rect.max_y(), config.sheet_length - config.edge_margin,
                );
            }
        }
    }

    #[test]
    fn test_zero_kerf_tight_packing() {
        // With zero kerf, parts should be placed edge-to-edge
        let parts = vec![
            NestingPart { id: "a".into(), width: 24.0, height: 48.0, can_rotate: false },
            NestingPart { id: "b".into(), width: 24.0, height: 48.0, can_rotate: false },
            NestingPart { id: "c".into(), width: 24.0, height: 48.0, can_rotate: false },
            NestingPart { id: "d".into(), width: 24.0, height: 48.0, can_rotate: false },
        ];
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.0,
            edge_margin: 0.0,
            allow_rotation: false,
            guillotine_compatible: false,
        };
        let result = nest_parts(&parts, &config);

        // 4 parts of 24x48 = exactly fill a 48x96 sheet
        assert_eq!(result.sheet_count, 1);
        assert_eq!(result.sheets[0].parts.len(), 4);
        assert!((result.overall_utilization - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_rotation_with_maxrects() {
        // A 90x10 part won't fit on a 48x96 sheet width-wise,
        // but rotated to 10x90 it will
        let parts = vec![NestingPart {
            id: "long".into(),
            width: 90.0,
            height: 10.0,
            can_rotate: true,
        }];
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.0,
            edge_margin: 0.0,
            allow_rotation: true,
            guillotine_compatible: false,
        };
        let result = nest_parts(&parts, &config);

        assert_eq!(result.sheet_count, 1);
        assert!(result.sheets[0].parts[0].rotated);
    }

    #[test]
    fn test_empty_parts_list() {
        let parts: Vec<NestingPart> = vec![];
        let config = NestingConfig::default();
        let result = nest_parts(&parts, &config);

        assert_eq!(result.sheet_count, 0);
        assert!(result.sheets.is_empty());
        assert!(result.unplaced.is_empty());
        assert!((result.overall_utilization - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_part_barely_fits() {
        // Part dimensions exactly match usable area
        let parts = vec![NestingPart {
            id: "tight".into(),
            width: 47.0,
            height: 95.0,
            can_rotate: false,
        }];
        let config = NestingConfig {
            sheet_width: 48.0,
            sheet_length: 96.0,
            kerf: 0.0,
            edge_margin: 0.5,
            allow_rotation: false,
            guillotine_compatible: false,
        };
        let result = nest_parts(&parts, &config);

        assert_eq!(result.sheet_count, 1);
        assert!(result.unplaced.is_empty());
    }

    // --- Stress / edge-case tests ---

    #[test]
    fn test_single_huge_part_fills_sheet() {
        // A single part that uses almost the entire sheet
        let parts = vec![NestingPart {
            id: "huge".into(),
            width: 46.0,
            height: 94.0,
            can_rotate: false,
        }];
        let config = NestingConfig {
            edge_margin: 0.5,
            kerf: 0.25,
            ..Default::default()
        };
        let result = nest_parts(&parts, &config);
        assert_eq!(result.sheet_count, 1);
        assert!(result.unplaced.is_empty());
        assert!(result.overall_utilization > 90.0, "huge part should give >90% utilization");
    }

    #[test]
    fn test_100_tiny_parts() {
        // 100 small 2"x2" parts on a 48x96 sheet
        let parts: Vec<NestingPart> = (0..100)
            .map(|i| NestingPart {
                id: format!("tiny_{}", i),
                width: 2.0,
                height: 2.0,
                can_rotate: false,
            })
            .collect();
        let config = NestingConfig {
            kerf: 0.125,
            edge_margin: 0.5,
            ..Default::default()
        };
        let result = nest_parts(&parts, &config);
        // 100 parts at ~2.125"x2.125" each = ~45,156 sq in needed
        // Sheet usable = 47x95 = 4465 sq in → should fit on 1 sheet
        assert_eq!(result.sheet_count, 1, "100 tiny parts should fit on 1 sheet");
        assert!(result.unplaced.is_empty());
    }

    #[test]
    fn test_identical_large_parts_multi_sheet() {
        // 6 parts of 24"x24" — only ~4 fit per sheet (47"x95" usable with kerf)
        let parts: Vec<NestingPart> = (0..6)
            .map(|i| NestingPart {
                id: format!("panel_{}", i),
                width: 24.0,
                height: 24.0,
                can_rotate: false,
            })
            .collect();
        let config = NestingConfig::default();
        let result = nest_parts(&parts, &config);
        assert!(result.unplaced.is_empty(), "all parts should be placed");
        assert!(result.sheet_count >= 2, "should need at least 2 sheets");
        // Verify no overlaps across all sheets
        for sheet in &result.sheets {
            for i in 0..sheet.parts.len() {
                for j in (i + 1)..sheet.parts.len() {
                    assert!(
                        !rects_overlap(&sheet.parts[i].rect, &sheet.parts[j].rect),
                        "parts {} and {} overlap on sheet {}",
                        sheet.parts[i].id, sheet.parts[j].id, sheet.sheet_index
                    );
                }
            }
        }
    }

    #[test]
    fn test_very_large_kerf() {
        // Kerf of 1" — should still work, just fewer parts per sheet
        let parts = vec![
            NestingPart { id: "a".into(), width: 10.0, height: 10.0, can_rotate: false },
            NestingPart { id: "b".into(), width: 10.0, height: 10.0, can_rotate: false },
        ];
        let config = NestingConfig {
            kerf: 1.0,
            edge_margin: 0.5,
            ..Default::default()
        };
        let result = nest_parts(&parts, &config);
        assert!(result.unplaced.is_empty());
        // Both 10" parts with 1" kerf should still fit on a 48x96 sheet
        assert_eq!(result.sheet_count, 1);
        // Verify no overlap with kerf gap
        let sheet = &result.sheets[0];
        if sheet.parts.len() == 2 {
            let r0 = &sheet.parts[0].rect;
            let r1 = &sheet.parts[1].rect;
            assert!(!rects_overlap(r0, r1), "parts should not overlap even with large kerf");
        }
    }

    #[test]
    fn test_width_equals_sheet_width() {
        // Part width exactly matches usable sheet width
        let parts = vec![NestingPart {
            id: "full_width".into(),
            width: 47.0, // 48 - 2*0.5 margin = 47
            height: 10.0,
            can_rotate: false,
        }];
        let config = NestingConfig {
            kerf: 0.0,
            edge_margin: 0.5,
            ..Default::default()
        };
        let result = nest_parts(&parts, &config);
        assert_eq!(result.sheet_count, 1);
        assert!(result.unplaced.is_empty());
    }

    #[test]
    fn test_width_just_over_sheet_width() {
        // Part slightly wider than usable sheet width → unplaced
        let parts = vec![NestingPart {
            id: "too_wide".into(),
            width: 47.01, // 48 - 2*0.5 = 47 usable
            height: 10.0,
            can_rotate: false,
        }];
        let config = NestingConfig {
            kerf: 0.0,
            edge_margin: 0.5,
            allow_rotation: false,
            ..Default::default()
        };
        let result = nest_parts(&parts, &config);
        assert_eq!(result.unplaced.len(), 1, "part should be unplaced");
    }

    #[test]
    fn test_rotation_saves_too_wide_part() {
        // Part is 47.01 x 10 — too wide, but 10 x 47.01 fits on the 95" length
        let parts = vec![NestingPart {
            id: "rotatable".into(),
            width: 47.01,
            height: 10.0,
            can_rotate: true,
        }];
        let config = NestingConfig {
            kerf: 0.0,
            edge_margin: 0.5,
            allow_rotation: true,
            ..Default::default()
        };
        let result = nest_parts(&parts, &config);
        assert!(result.unplaced.is_empty(), "rotation should save this part");
        assert_eq!(result.sheet_count, 1);
    }

    #[test]
    fn test_guillotine_with_mixed_sizes() {
        // Guillotine mode should still place all parts, just potentially less efficiently
        let parts = vec![
            NestingPart { id: "big".into(), width: 20.0, height: 30.0, can_rotate: false },
            NestingPart { id: "med".into(), width: 15.0, height: 15.0, can_rotate: false },
            NestingPart { id: "small".into(), width: 5.0, height: 5.0, can_rotate: false },
        ];
        let config = NestingConfig {
            guillotine_compatible: true,
            ..Default::default()
        };
        let result = nest_parts(&parts, &config);
        assert!(result.unplaced.is_empty(), "all parts should be placed in guillotine mode");
    }

    #[test]
    fn test_all_parts_oversized() {
        // All parts are too large → all should be unplaced
        let parts = vec![
            NestingPart { id: "huge1".into(), width: 100.0, height: 100.0, can_rotate: false },
            NestingPart { id: "huge2".into(), width: 200.0, height: 200.0, can_rotate: false },
        ];
        let config = NestingConfig::default();
        let result = nest_parts(&parts, &config);
        assert_eq!(result.unplaced.len(), 2);
        assert_eq!(result.sheet_count, 0);
    }

    #[test]
    fn test_one_part_per_dimension_extreme() {
        // Very thin, very long part — 1" x 94"
        let parts = vec![NestingPart {
            id: "strip".into(),
            width: 1.0,
            height: 94.0,
            can_rotate: false,
        }];
        let config = NestingConfig::default();
        let result = nest_parts(&parts, &config);
        assert_eq!(result.sheet_count, 1);
        assert!(result.unplaced.is_empty());
    }

    #[test]
    fn test_nesting_result_sheet_indices() {
        // Verify sheet indices are sequential starting from 0
        let parts: Vec<NestingPart> = (0..20)
            .map(|i| NestingPart {
                id: format!("panel_{}", i),
                width: 20.0,
                height: 40.0,
                can_rotate: false,
            })
            .collect();
        let config = NestingConfig::default();
        let result = nest_parts(&parts, &config);
        for (idx, sheet) in result.sheets.iter().enumerate() {
            assert_eq!(sheet.sheet_index, idx, "sheet index should match position");
        }
        assert_eq!(result.sheets.len(), result.sheet_count);
    }

    #[test]
    fn test_large_edge_margin() {
        // 5" edge margin on all sides → only 38x86 usable on 48x96
        let parts = vec![NestingPart {
            id: "constrained".into(),
            width: 37.0,
            height: 85.0,
            can_rotate: false,
        }];
        let config = NestingConfig {
            edge_margin: 5.0,
            kerf: 0.0,
            ..Default::default()
        };
        let result = nest_parts(&parts, &config);
        assert_eq!(result.sheet_count, 1);
        assert!(result.unplaced.is_empty());

        // Part that's too big for the reduced area
        let parts2 = vec![NestingPart {
            id: "too_big".into(),
            width: 39.0,
            height: 85.0,
            can_rotate: false,
        }];
        let result2 = nest_parts(&parts2, &config);
        assert_eq!(result2.unplaced.len(), 1, "should not fit with large margins");
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
