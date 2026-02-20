use cm_core::geometry::Point2D;
use serde::{Deserialize, Serialize};

/// A complete toolpath for one operation on one part.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Toolpath {
    /// Tool number to use (references the tool library).
    pub tool_number: u32,

    /// Spindle RPM.
    pub rpm: f64,

    /// Feed rate for cutting moves (project units per minute).
    pub feed_rate: f64,

    /// Plunge feed rate (Z moves into material).
    pub plunge_rate: f64,

    /// The sequence of motions.
    pub segments: Vec<ToolpathSegment>,
}

/// A single segment of a toolpath.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ToolpathSegment {
    pub motion: Motion,
    pub endpoint: Point2D,
    /// Z height at the endpoint. Negative = into material (convention: Z=0 is material surface).
    pub z: f64,
}

/// Types of CNC motion.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Motion {
    /// G00: Rapid move (no cutting, max speed).
    Rapid,
    /// G01: Linear feed (cutting).
    Linear,
    /// G02: Clockwise arc.
    ArcCW {
        /// Arc center relative to start point (I, J values).
        i: f64,
        j: f64,
    },
    /// G03: Counter-clockwise arc.
    ArcCCW {
        /// Arc center relative to start point (I, J values).
        i: f64,
        j: f64,
    },
    /// G81/G83: Canned drilling cycle.
    DrillCycle {
        /// R plane — retract height between pecks.
        retract_z: f64,
        /// Final Z depth.
        final_z: f64,
        /// Peck depth (Q value). 0 = G81 simple drill, >0 = G83 peck drill.
        peck_depth: f64,
    },
}

/// Style of corner fillet for internal corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilletStyle {
    /// Diagonal extension into the corner (45-degree).
    DogBone,
    /// Extension along the longer edge at the corner.
    TBone,
}

/// Scan a toolpath for collinear or circular patterns among consecutive Linear segments
/// and replace them with arc segments where possible.
///
/// The algorithm uses a sliding window: for each group of 3+ consecutive Linear segments
/// at the same Z height, it tests whether they lie on a circle within the given tolerance.
/// If so, it replaces them with a single ArcCW or ArcCCW segment.
pub fn arc_fit(toolpath: &mut Toolpath, tolerance: f64) {
    let segments = &toolpath.segments;
    if segments.len() < 3 {
        return;
    }

    let mut new_segments: Vec<ToolpathSegment> = Vec::with_capacity(segments.len());
    let mut i = 0;

    while i < segments.len() {
        // Only try to fit arcs on consecutive Linear segments at the same Z
        if !matches!(segments[i].motion, Motion::Linear) {
            new_segments.push(segments[i].clone());
            i += 1;
            continue;
        }

        // Collect a run of consecutive Linear segments at the same Z
        let run_start = i;
        let z = segments[i].z;
        let mut run_end = i + 1;
        while run_end < segments.len()
            && matches!(segments[run_end].motion, Motion::Linear)
            && (segments[run_end].z - z).abs() < 1e-10
        {
            run_end += 1;
        }

        let run_len = run_end - run_start;
        if run_len < 3 {
            // Not enough segments for arc fitting
            for seg in &segments[run_start..run_end] {
                new_segments.push(seg.clone());
            }
            i = run_end;
            continue;
        }

        // Get all points in this run (including the point before the run as the start)
        let _start_point = if run_start > 0 {
            segments[run_start - 1].endpoint
        } else {
            segments[run_start].endpoint
        };

        // Try to fit arcs from the beginning of the run
        let mut j = run_start;
        while j < run_end {
            if j + 2 < run_end {
                // Try fitting 3 points as an arc
                let p0 = if j > 0 {
                    segments[j - 1].endpoint
                } else {
                    // First segment in run with no predecessor — skip arc fitting for this point
                    new_segments.push(segments[j].clone());
                    j += 1;
                    continue;
                };
                let p1 = segments[j].endpoint;
                let p2 = segments[j + 1].endpoint;

                // Extend the arc as far as possible
                let mut arc_end = j + 2;
                if let Some((cx, cy, r)) = fit_circle(p0, p1, p2) {
                    // Check if subsequent points also lie on this circle
                    while arc_end < run_end {
                        let pt = segments[arc_end].endpoint;
                        let dist = ((pt.x - cx).powi(2) + (pt.y - cy).powi(2)).sqrt();
                        if (dist - r).abs() > tolerance {
                            break;
                        }
                        arc_end += 1;
                    }

                    // Determine CW or CCW using cross product
                    let endpoint = segments[arc_end - 1].endpoint;
                    let cross = (p1.x - p0.x) * (p2.y - p0.y) - (p1.y - p0.y) * (p2.x - p0.x);

                    // I, J are relative to the start point of the arc
                    let arc_i = cx - p0.x;
                    let arc_j = cy - p0.y;

                    let motion = if cross > 0.0 {
                        Motion::ArcCCW { i: arc_i, j: arc_j }
                    } else {
                        Motion::ArcCW { i: arc_i, j: arc_j }
                    };

                    new_segments.push(ToolpathSegment {
                        motion,
                        endpoint,
                        z,
                    });

                    j = arc_end;
                    continue;
                }
            }

            // Couldn't fit an arc, keep the linear segment
            new_segments.push(segments[j].clone());
            j += 1;
        }

        i = run_end;
    }

    toolpath.segments = new_segments;
}

/// Fit a circle through three points. Returns (cx, cy, radius) or None if collinear.
fn fit_circle(p0: Point2D, p1: Point2D, p2: Point2D) -> Option<(f64, f64, f64)> {
    let ax = p0.x;
    let ay = p0.y;
    let bx = p1.x;
    let by = p1.y;
    let cx = p2.x;
    let cy = p2.y;

    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    if d.abs() < 1e-10 {
        return None; // Collinear
    }

    let ux = ((ax * ax + ay * ay) * (by - cy)
        + (bx * bx + by * by) * (cy - ay)
        + (cx * cx + cy * cy) * (ay - by))
        / d;
    let uy = ((ax * ax + ay * ay) * (cx - bx)
        + (bx * bx + by * by) * (ax - cx)
        + (cx * cx + cy * cy) * (bx - ax))
        / d;

    let r = ((ax - ux).powi(2) + (ay - uy).powi(2)).sqrt();
    Some((ux, uy, r))
}

/// Optimize the order of toolpaths to minimize rapid travel distance.
///
/// Uses a nearest-neighbor greedy algorithm: starting from (0,0), pick the
/// toolpath whose start point is closest to the current tool position, then
/// move to its end point, and repeat. Toolpaths are grouped by tool number
/// so tool changes are not affected.
pub fn optimize_rapid_order(toolpaths: &mut Vec<Toolpath>) {
    if toolpaths.len() <= 1 {
        return;
    }

    // Group by tool number to preserve tool-change ordering
    let mut groups: Vec<(u32, Vec<usize>)> = Vec::new();
    for (i, tp) in toolpaths.iter().enumerate() {
        if let Some(group) = groups.last_mut()
            && group.0 == tp.tool_number
        {
            group.1.push(i);
            continue;
        }
        groups.push((tp.tool_number, vec![i]));
    }

    // Pre-cache start/end points to avoid repeated segment lookups
    let start_points: Vec<Option<Point2D>> = toolpaths
        .iter()
        .map(|tp| tp.segments.first().map(|s| s.endpoint))
        .collect();
    let end_points: Vec<Option<Point2D>> = toolpaths
        .iter()
        .map(|tp| tp.segments.last().map(|s| s.endpoint))
        .collect();

    let mut order: Vec<usize> = Vec::with_capacity(toolpaths.len());
    let mut current_pos = Point2D::origin();

    for (_tool, indices) in &groups {
        let mut remaining: Vec<usize> = indices.clone();
        while !remaining.is_empty() {
            // Find the toolpath with start point closest to current_pos
            let mut best_ri = 0;
            let mut best_dist = f64::MAX;
            for (ri, &tp_idx) in remaining.iter().enumerate() {
                if let Some(pt) = start_points[tp_idx] {
                    let dx = pt.x - current_pos.x;
                    let dy = pt.y - current_pos.y;
                    let dist = dx * dx + dy * dy;
                    if dist < best_dist {
                        best_dist = dist;
                        best_ri = ri;
                    }
                }
            }

            let tp_idx = remaining.swap_remove(best_ri);
            if let Some(pt) = end_points[tp_idx] {
                current_pos = pt;
            }
            order.push(tp_idx);
        }
    }

    // Move toolpaths into new order without cloning
    let mut slots = std::mem::take(toolpaths);
    *toolpaths = order
        .into_iter()
        .map(|idx| std::mem::take(&mut slots[idx]))
        .collect();
}

/// Apply dog-bone or T-bone corner fillets to internal 90-degree corners.
///
/// Scans for right-angle direction changes among Linear segments and inserts
/// a small extension (= tool_radius) into each internal corner so square
/// parts can fit properly in CNC-routed slots.
pub fn apply_corner_fillets(toolpath: &mut Toolpath, tool_radius: f64, style: FilletStyle) {
    if toolpath.segments.len() < 2 || tool_radius <= 0.0 {
        return;
    }

    let mut new_segments: Vec<ToolpathSegment> = Vec::with_capacity(toolpath.segments.len() * 2);

    for i in 0..toolpath.segments.len() {
        new_segments.push(toolpath.segments[i].clone());

        // Need at least a previous segment and a next segment (both Linear at same Z)
        if i == 0 || i + 1 >= toolpath.segments.len() {
            continue;
        }

        let prev = &toolpath.segments[i - 1];
        let curr = &toolpath.segments[i];
        let next = &toolpath.segments[i + 1];

        // Only apply to consecutive Linear segments at the same Z
        if !matches!(prev.motion, Motion::Linear)
            || !matches!(curr.motion, Motion::Linear)
            || !matches!(next.motion, Motion::Linear)
        {
            continue;
        }

        if (prev.z - curr.z).abs() > 1e-10 || (curr.z - next.z).abs() > 1e-10 {
            continue;
        }

        // Direction vectors
        let d1x = curr.endpoint.x - prev.endpoint.x;
        let d1y = curr.endpoint.y - prev.endpoint.y;
        let d2x = next.endpoint.x - curr.endpoint.x;
        let d2y = next.endpoint.y - curr.endpoint.y;

        let len1 = (d1x * d1x + d1y * d1y).sqrt();
        let len2 = (d2x * d2x + d2y * d2y).sqrt();
        if len1 < 1e-10 || len2 < 1e-10 {
            continue;
        }

        // Normalize
        let u1x = d1x / len1;
        let u1y = d1y / len1;
        let u2x = d2x / len2;
        let u2y = d2y / len2;

        // Check for ~90 degree turn using dot product
        let dot = u1x * u2x + u1y * u2y;
        if dot.abs() > 0.1 {
            continue; // Not a right angle
        }

        // Cross product to detect internal vs external corner
        // Cross product could be used to detect internal vs external corners,
        // but we apply fillets to all right-angle corners for slot clearing.

        let corner = curr.endpoint;
        let z = curr.z;

        match style {
            FilletStyle::DogBone => {
                // Diagonal extension: move tool_radius in the diagonal direction
                // (bisector of the two incoming directions, pointing into the corner)
                let bisect_x = -(u1x + u2x);
                let bisect_y = -(u1y + u2y);
                let bisect_len = (bisect_x * bisect_x + bisect_y * bisect_y).sqrt();
                if bisect_len < 1e-10 {
                    continue;
                }
                let ext_x = corner.x + (bisect_x / bisect_len) * tool_radius;
                let ext_y = corner.y + (bisect_y / bisect_len) * tool_radius;

                // Insert: go to extension point, then return to corner
                new_segments.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: Point2D::new(ext_x, ext_y),
                    z,
                });
                new_segments.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: corner,
                    z,
                });
            }
            FilletStyle::TBone => {
                // Extension along the longer incoming edge
                let (ext_ux, ext_uy) = if len1 >= len2 {
                    (u1x, u1y) // extend along incoming direction
                } else {
                    (-u2x, -u2y) // extend against outgoing direction
                };
                let ext_x = corner.x + ext_ux * tool_radius;
                let ext_y = corner.y + ext_uy * tool_radius;

                new_segments.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: Point2D::new(ext_x, ext_y),
                    z,
                });
                new_segments.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: corner,
                    z,
                });
            }
        }
    }

    toolpath.segments = new_segments;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_fit_collinear_segments_unchanged() {
        // 3 collinear points should not be replaced with an arc
        let mut tp = Toolpath {
            tool_number: 1,
            rpm: 5000.0,
            feed_rate: 100.0,
            plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(0.0, 0.0), z: 0.0 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(1.0, 0.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(2.0, 0.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(3.0, 0.0), z: -0.5 },
            ],
        };
        let _orig_len = tp.segments.len();
        arc_fit(&mut tp, 0.001);
        // Collinear points don't form a circle — should remain linear
        assert!(tp.segments.iter().all(|s| !matches!(s.motion, Motion::ArcCW { .. } | Motion::ArcCCW { .. })));
    }

    #[test]
    fn test_arc_fit_circular_segments() {
        // Generate points on a circle and verify they get collapsed to an arc
        let cx = 5.0;
        let cy = 5.0;
        let r = 2.0;
        let n = 8;
        let mut segments = vec![
            ToolpathSegment {
                motion: Motion::Rapid,
                endpoint: Point2D::new(cx + r, cy),
                z: 0.0,
            },
        ];
        for i in 1..=n {
            let angle = std::f64::consts::PI * 2.0 * i as f64 / n as f64;
            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: Point2D::new(cx + r * angle.cos(), cy + r * angle.sin()),
                z: -0.5,
            });
        }

        let mut tp = Toolpath {
            tool_number: 1,
            rpm: 5000.0,
            feed_rate: 100.0,
            plunge_rate: 50.0,
            segments,
        };

        arc_fit(&mut tp, 0.01);
        // Should have fewer segments (arcs collapsed)
        let arc_count = tp.segments.iter()
            .filter(|s| matches!(s.motion, Motion::ArcCW { .. } | Motion::ArcCCW { .. }))
            .count();
        assert!(arc_count > 0, "should have at least one arc segment after fitting");
    }

    #[test]
    fn test_optimize_rapid_order_reduces_travel() {
        let tp_far = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(90.0, 90.0), z: 1.0 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(90.0, 80.0), z: -0.5 },
            ],
        };
        let tp_near = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(1.0, 1.0), z: 1.0 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(1.0, 10.0), z: -0.5 },
            ],
        };

        let mut tps = vec![tp_far, tp_near];
        optimize_rapid_order(&mut tps);

        // The near toolpath should come first since we start from (0,0)
        assert_eq!(tps[0].segments[0].endpoint.x, 1.0);
    }

    #[test]
    fn test_optimize_rapid_preserves_tool_groups() {
        let tp1_t1 = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(10.0, 10.0), z: 1.0 },
            ],
        };
        let tp2_t2 = Toolpath {
            tool_number: 2, rpm: 3000.0, feed_rate: 80.0, plunge_rate: 40.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(5.0, 5.0), z: 1.0 },
            ],
        };

        let mut tps = vec![tp1_t1, tp2_t2];
        optimize_rapid_order(&mut tps);

        // Tool order should be preserved
        assert_eq!(tps[0].tool_number, 1);
        assert_eq!(tps[1].tool_number, 2);
    }

    #[test]
    fn test_dogbone_fillet_on_right_angle() {
        // Create an L-shaped path with a 90-degree corner
        let mut tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(5.0, 0.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(5.0, 5.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(10.0, 5.0), z: -0.5 },
            ],
        };

        let orig_len = tp.segments.len();
        apply_corner_fillets(&mut tp, 0.125, FilletStyle::DogBone);

        // Should have inserted 2 extra segments (extension + return) at the corner
        assert!(tp.segments.len() > orig_len,
            "should have more segments after filleting, got {} vs {}", tp.segments.len(), orig_len);
    }

    #[test]
    fn test_tbone_fillet_on_right_angle() {
        let mut tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(5.0, 0.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(5.0, 5.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(10.0, 5.0), z: -0.5 },
            ],
        };

        apply_corner_fillets(&mut tp, 0.125, FilletStyle::TBone);
        assert!(tp.segments.len() > 3);
    }

    #[test]
    fn test_fillet_extension_distance() {
        let mut tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(5.0, 0.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(5.0, 5.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(10.0, 5.0), z: -0.5 },
            ],
        };

        let tool_radius = 0.125;
        apply_corner_fillets(&mut tp, tool_radius, FilletStyle::DogBone);

        // Find the extension point (first new segment after the corner)
        // The corner is at (5, 5). The extension should be tool_radius away.
        let corner = Point2D::new(5.0, 5.0);
        for seg in &tp.segments {
            let dx = seg.endpoint.x - corner.x;
            let dy = seg.endpoint.y - corner.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > 1e-10 && dist < tool_radius * 2.0 {
                assert!(
                    (dist - tool_radius).abs() < 1e-10,
                    "extension should be exactly tool_radius ({}) away, got {}",
                    tool_radius, dist,
                );
            }
        }
    }

    // --- Additional toolpath quality tests ---

    #[test]
    fn test_arc_fit_empty_toolpath() {
        let mut tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![],
        };
        arc_fit(&mut tp, 0.001);
        assert!(tp.segments.is_empty(), "empty toolpath should stay empty");
    }

    #[test]
    fn test_arc_fit_single_segment() {
        let mut tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(1.0, 1.0), z: 0.0 },
            ],
        };
        arc_fit(&mut tp, 0.001);
        assert_eq!(tp.segments.len(), 1);
    }

    #[test]
    fn test_arc_fit_preserves_rapids() {
        let mut tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(0.0, 0.0), z: 1.0 },
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(5.0, 5.0), z: 1.0 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(10.0, 5.0), z: -0.5 },
            ],
        };
        arc_fit(&mut tp, 0.001);
        // Rapids should not be touched
        let rapid_count = tp.segments.iter()
            .filter(|s| matches!(s.motion, Motion::Rapid))
            .count();
        assert_eq!(rapid_count, 2, "rapids should be preserved");
    }

    #[test]
    fn test_arc_fit_different_z_levels() {
        // Segments at different Z levels should not be arc-fitted together
        let mut tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(0.0, 0.0), z: 0.0 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(1.0, 0.0), z: -0.25 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(2.0, 0.0), z: -0.50 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(3.0, 0.0), z: -0.75 },
            ],
        };
        arc_fit(&mut tp, 0.001);
        // Different Z levels → each is its own "run" → no arcs
        let arc_count = tp.segments.iter()
            .filter(|s| matches!(s.motion, Motion::ArcCW { .. } | Motion::ArcCCW { .. }))
            .count();
        assert_eq!(arc_count, 0, "different Z levels should not produce arcs");
    }

    #[test]
    fn test_optimize_rapid_order_empty() {
        let mut tps: Vec<Toolpath> = vec![];
        optimize_rapid_order(&mut tps);
        assert!(tps.is_empty());
    }

    #[test]
    fn test_optimize_rapid_order_single() {
        let mut tps = vec![Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(10.0, 10.0), z: 1.0 },
            ],
        }];
        optimize_rapid_order(&mut tps);
        assert_eq!(tps.len(), 1);
    }

    #[test]
    fn test_optimize_rapid_nearest_neighbor() {
        // 3 toolpaths at (10,10), (1,1), (5,5) — from origin, nearest is (1,1), then (5,5), then (10,10)
        let tp_a = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(10.0, 10.0), z: 1.0 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(11.0, 10.0), z: -0.5 },
            ],
        };
        let tp_b = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(1.0, 1.0), z: 1.0 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(2.0, 1.0), z: -0.5 },
            ],
        };
        let tp_c = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(5.0, 5.0), z: 1.0 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(6.0, 5.0), z: -0.5 },
            ],
        };

        let mut tps = vec![tp_a, tp_b, tp_c];
        optimize_rapid_order(&mut tps);

        // From origin (0,0): nearest is (1,1), then from (2,1): nearest is (5,5), then (10,10)
        assert!((tps[0].segments[0].endpoint.x - 1.0).abs() < 0.01, "first should be nearest to origin");
        assert!((tps[1].segments[0].endpoint.x - 5.0).abs() < 0.01, "second should be at (5,5)");
        assert!((tps[2].segments[0].endpoint.x - 10.0).abs() < 0.01, "third should be at (10,10)");
    }

    #[test]
    fn test_optimize_rapid_order_preserves_all_toolpaths() {
        // Verify no data loss from mem::take reordering
        let tps_data: Vec<(f64, f64, usize)> = vec![
            (20.0, 20.0, 3),
            (1.0, 1.0, 5),
            (10.0, 10.0, 2),
            (5.0, 5.0, 4),
        ];

        let mut tps: Vec<Toolpath> = tps_data.iter().map(|(x, y, seg_count)| {
            let mut segs = vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(*x, *y), z: 1.0 },
            ];
            for i in 1..*seg_count {
                segs.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: Point2D::new(*x + i as f64, *y),
                    z: -0.5,
                });
            }
            Toolpath { tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0, segments: segs }
        }).collect();

        let original_count = tps.len();
        let original_total_segments: usize = tps.iter().map(|t| t.segments.len()).sum();

        optimize_rapid_order(&mut tps);

        // Same number of toolpaths
        assert_eq!(tps.len(), original_count, "should preserve toolpath count");

        // Same total segments (no data loss)
        let reordered_total_segments: usize = tps.iter().map(|t| t.segments.len()).sum();
        assert_eq!(reordered_total_segments, original_total_segments, "should preserve total segment count");

        // No empty toolpaths from mem::take
        for (i, tp) in tps.iter().enumerate() {
            assert!(!tp.segments.is_empty(), "toolpath {} should not be empty after reorder", i);
        }
    }

    #[test]
    fn test_optimize_rapid_order_swap_remove_nearest_neighbor() {
        // Verify ordering still minimizes travel with swap_remove
        let make_tp = |x: f64, y: f64| Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(x, y), z: 1.0 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(x + 1.0, y), z: -0.5 },
            ],
        };

        let mut tps = vec![
            make_tp(50.0, 50.0),
            make_tp(2.0, 2.0),
            make_tp(30.0, 30.0),
            make_tp(8.0, 8.0),
            make_tp(15.0, 15.0),
        ];

        optimize_rapid_order(&mut tps);

        // From (0,0): nearest = (2,2), then (8,8), (15,15), (30,30), (50,50)
        let xs: Vec<f64> = tps.iter().map(|t| t.segments[0].endpoint.x).collect();
        assert!((xs[0] - 2.0).abs() < 0.01, "first should be nearest to origin, got {}", xs[0]);
        // Each successive should be farther from origin (greedy nearest neighbor)
        for w in xs.windows(2) {
            assert!(w[0] <= w[1] + 1.0, "should generally proceed outward: {} -> {}", w[0], w[1]);
        }
    }

    #[test]
    fn test_fillet_on_rectangular_path() {
        // Full rectangle: 4 corners, all 90 degrees
        let mut tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(10.0, 0.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(10.0, 5.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(0.0, 5.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(0.0, 0.0), z: -0.5 },
            ],
        };

        apply_corner_fillets(&mut tp, 0.125, FilletStyle::DogBone);
        // Each of 3 internal corners should get 2 extra segments (extension + return)
        // But only corners between consecutive Linear segments that form a 90-degree angle
        // Corners: at (10,0)→(10,5), at (10,5)→(0,5), at (0,5)→(0,0) = 3 corners
        // Each adds 2 segments: 4 original + 3*2 = 10
        assert!(tp.segments.len() >= 7, "rectangle should get fillets at corners, got {}", tp.segments.len());
    }

    #[test]
    fn test_fillet_with_zero_tool_radius() {
        // tool_radius=0 should effectively be a no-op
        let mut tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(5.0, 0.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(5.0, 5.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(10.0, 5.0), z: -0.5 },
            ],
        };

        let orig_len = tp.segments.len();
        apply_corner_fillets(&mut tp, 0.0, FilletStyle::DogBone);
        // With zero radius, extension point is at the corner itself → effectively no change
        // (may still add segments but they'll be zero-length)
        assert!(tp.segments.len() >= orig_len);
    }

    #[test]
    fn test_fillet_does_not_modify_rapids() {
        let mut tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(5.0, 0.0), z: 1.0 },
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(5.0, 5.0), z: 1.0 },
                ToolpathSegment { motion: Motion::Rapid, endpoint: Point2D::new(10.0, 5.0), z: 1.0 },
            ],
        };

        let orig_len = tp.segments.len();
        apply_corner_fillets(&mut tp, 0.125, FilletStyle::DogBone);
        assert_eq!(tp.segments.len(), orig_len, "rapids should not get fillets");
    }

    #[test]
    fn test_no_fillet_on_non_right_angle() {
        // 45-degree angle — should not get a fillet
        let mut tp = Toolpath {
            tool_number: 1, rpm: 5000.0, feed_rate: 100.0, plunge_rate: 50.0,
            segments: vec![
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(5.0, 0.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(5.0, 5.0), z: -0.5 },
                ToolpathSegment { motion: Motion::Linear, endpoint: Point2D::new(10.0, 10.0), z: -0.5 },
            ],
        };

        let orig_len = tp.segments.len();
        apply_corner_fillets(&mut tp, 0.125, FilletStyle::DogBone);
        assert_eq!(tp.segments.len(), orig_len, "no fillet on non-right-angle");
    }
}
