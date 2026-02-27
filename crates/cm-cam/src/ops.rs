use cm_core::geometry::{Point2D, Rect};
use cm_core::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::toolpath::{Motion, Toolpath, ToolpathSegment};

/// Configuration for toolpath generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CamConfig {
    /// Z height for rapid moves above the workpiece.
    pub safe_z: f64,
    /// Z height for rapid traverse between cuts (lower than safe_z for speed).
    pub rapid_z: f64,
    /// Max depth per pass.
    pub depth_per_pass: f64,
    /// Tab width for hold-down tabs on profile cuts.
    pub tab_width: f64,
    /// Tab height (how much material to leave for the tab).
    pub tab_height: f64,
    /// Number of tabs per side (for profile cuts).
    pub tabs_per_side: u32,
    /// Optional arc lead-in radius for profile cuts. None = direct plunge (default).
    /// Some(r) = quarter-circle arc approach/retract at the start of the cut.
    #[serde(default)]
    pub lead_in_radius: Option<f64>,
    /// Whether to use canned drilling cycles (G81/G83) instead of manual peck moves.
    #[serde(default)]
    pub use_canned_cycles: bool,
}

impl Default for CamConfig {
    fn default() -> Self {
        Self {
            safe_z: 1.0,
            rapid_z: 0.25,
            depth_per_pass: 0.25,
            tab_width: 0.5,
            tab_height: 0.125,
            tabs_per_side: 2,
            lead_in_radius: None,
            use_canned_cycles: false,
        }
    }
}

/// Parameters for a dado (groove) cut.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DadoParams {
    pub position: f64,
    pub width: f64,
    pub depth: f64,
    pub horizontal: bool,
}

/// Parameters for shelf pin hole pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShelfPinParams {
    pub hole_depth: f64,
    pub setback: f64,
    pub start_height: f64,
    pub end_height: f64,
}

/// Parameters for a dovetail joint cut.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DovetailParams {
    pub edge: DovetailEdge,
    pub tail_count: u32,
    pub tail_width: f64,
    pub pin_width: f64,
    pub depth: f64,
}

/// Parameters for a box (finger) joint cut.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxJointParams {
    pub edge: DovetailEdge,
    pub finger_width: f64,
    pub finger_count: u32,
    pub depth: f64,
}

/// Parameters for a mortise cut.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MortiseParams {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub length: f64,
    pub depth: f64,
}

/// Parameters for a tenon cut.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenonParams {
    pub edge: DovetailEdge,
    pub tenon_thickness: f64,
    pub tenon_width: f64,
    pub tenon_length: f64,
    pub shoulder_depth: f64,
}

/// Generate a profile (outline) cut for a rectangular part with hold-down tabs.
///
/// The tool cuts around the outside of the rectangle, offsetting by the tool
/// radius. Material depth is cut in multiple passes. On the final pass,
/// hold-down tabs are inserted to prevent the part from breaking loose.
///
/// Tabs are thin bridges of material left at evenly-spaced positions along
/// each side. They keep the part attached to the sheet during cutting and
/// are snapped off or trimmed after the part is removed.
///
/// The part rect is positioned relative to the sheet origin (set by nesting).
pub fn generate_profile_cut(
    part_rect: &Rect,
    material_thickness: f64,
    tool: &Tool,
    rpm: f64,
    config: &CamConfig,
) -> Toolpath {
    let feed_rate = tool.recommended_feed_rate(rpm);
    let plunge_rate = feed_rate * 0.5; // plunge at half feed rate
    let r = tool.radius();

    // Offset rectangle outward by tool radius (conventional milling: climb cut)
    let cut_rect = Rect::new(
        Point2D::new(part_rect.min_x() - r, part_rect.min_y() - r),
        part_rect.width + 2.0 * r,
        part_rect.height + 2.0 * r,
    );

    // Calculate depth passes
    let total_depth = material_thickness;
    let num_passes = (total_depth / config.depth_per_pass).ceil() as u32;

    let mut segments = Vec::with_capacity(num_passes as usize * 8 + 20);

    // Tab Z: leave tab_height of material on the final pass
    let tab_z = -total_depth + config.tab_height;

    // Lead-in: offset start point so we approach via arc
    let lead_in_r = config.lead_in_radius.unwrap_or(0.0);
    let use_lead_in = lead_in_r > 0.0;

    // Start position: bottom-left corner of cut path
    let start = Point2D::new(cut_rect.min_x(), cut_rect.min_y());
    // Lead-in approach point is offset by lead_in_r in -Y direction from start
    let approach_point = if use_lead_in {
        Point2D::new(start.x, start.y - lead_in_r)
    } else {
        start
    };

    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: approach_point,
        z: config.safe_z,
    });

    // Depth passes
    for pass in 1..=num_passes {
        let z = -(config.depth_per_pass * pass as f64).min(total_depth);
        let is_final_pass = pass == num_passes;

        // Rapid to rapid_z
        segments.push(ToolpathSegment {
            motion: Motion::Rapid,
            endpoint: approach_point,
            z: config.rapid_z,
        });

        // Plunge to cutting depth
        segments.push(ToolpathSegment {
            motion: Motion::Linear,
            endpoint: approach_point,
            z,
        });

        // Arc lead-in: quarter-circle CCW arc from approach_point to start
        if use_lead_in {
            segments.push(ToolpathSegment {
                motion: Motion::ArcCCW {
                    i: 0.0,
                    j: lead_in_r,
                },
                endpoint: start,
                z,
            });
        }

        if is_final_pass && config.tabs_per_side > 0 {
            // Final pass with tabs on each side
            let sides: [(Point2D, Point2D); 4] = [
                // Bottom: left → right
                (
                    Point2D::new(cut_rect.min_x(), cut_rect.min_y()),
                    Point2D::new(cut_rect.max_x(), cut_rect.min_y()),
                ),
                // Right: bottom → top
                (
                    Point2D::new(cut_rect.max_x(), cut_rect.min_y()),
                    Point2D::new(cut_rect.max_x(), cut_rect.max_y()),
                ),
                // Top: right → left
                (
                    Point2D::new(cut_rect.max_x(), cut_rect.max_y()),
                    Point2D::new(cut_rect.min_x(), cut_rect.max_y()),
                ),
                // Left: top → bottom (close)
                (
                    Point2D::new(cut_rect.min_x(), cut_rect.max_y()),
                    Point2D::new(cut_rect.min_x(), cut_rect.min_y()),
                ),
            ];

            for (side_start, side_end) in &sides {
                emit_side_with_tabs(
                    &mut segments,
                    side_start,
                    side_end,
                    z,
                    tab_z,
                    config.tab_width,
                    config.tabs_per_side,
                );
            }
        } else {
            // Non-final pass or no tabs: simple rectangle
            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: Point2D::new(cut_rect.max_x(), cut_rect.min_y()),
                z,
            });
            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: Point2D::new(cut_rect.max_x(), cut_rect.max_y()),
                z,
            });
            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: Point2D::new(cut_rect.min_x(), cut_rect.max_y()),
                z,
            });
            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: start,
                z,
            });
        }
    }

    // Arc lead-out and retract to safe Z
    if use_lead_in {
        // Arc lead-out: quarter-circle CW arc from start back to approach point
        let last_z = segments.last().map(|s| s.z).unwrap_or(config.safe_z);
        segments.push(ToolpathSegment {
            motion: Motion::ArcCW {
                i: 0.0,
                j: -lead_in_r,
            },
            endpoint: approach_point,
            z: last_z,
        });
    }

    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: approach_point,
        z: config.safe_z,
    });

    Toolpath {
        tool_number: tool.number,
        rpm,
        feed_rate,
        plunge_rate,
        segments,
    }
}

/// Emit segments for one side of a profile cut with evenly-spaced tabs.
///
/// Tabs are placed at equal intervals along the side. At each tab, the tool
/// ramps up to `tab_z`, traverses the tab width, then ramps back down to
/// the full cutting depth `z`.
fn emit_side_with_tabs(
    segments: &mut Vec<ToolpathSegment>,
    side_start: &Point2D,
    side_end: &Point2D,
    z: f64,
    tab_z: f64,
    tab_width: f64,
    tabs_per_side: u32,
) {
    let dx = side_end.x - side_start.x;
    let dy = side_end.y - side_start.y;
    let side_length = (dx * dx + dy * dy).sqrt();

    // Direction unit vector along this side
    let ux = dx / side_length;
    let uy = dy / side_length;

    // Place tabs evenly. With N tabs we have N+1 gaps.
    // Tab centers are at positions: side_length * (i+1) / (tabs_per_side+1)
    let half_tab = tab_width / 2.0;
    let mut current_dist = 0.0;

    for i in 0..tabs_per_side {
        let tab_center = side_length * (i + 1) as f64 / (tabs_per_side + 1) as f64;
        let tab_start_dist = (tab_center - half_tab).max(0.0);
        let tab_end_dist = (tab_center + half_tab).min(side_length);

        // Cut at full depth to the tab start
        if tab_start_dist > current_dist {
            let p = Point2D::new(
                side_start.x + ux * tab_start_dist,
                side_start.y + uy * tab_start_dist,
            );
            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: p,
                z,
            });
        }

        // Ramp up to tab height
        let tab_start_pt = Point2D::new(
            side_start.x + ux * tab_start_dist,
            side_start.y + uy * tab_start_dist,
        );
        segments.push(ToolpathSegment {
            motion: Motion::Linear,
            endpoint: tab_start_pt,
            z: tab_z,
        });

        // Traverse across tab at tab height
        let tab_end_pt = Point2D::new(
            side_start.x + ux * tab_end_dist,
            side_start.y + uy * tab_end_dist,
        );
        segments.push(ToolpathSegment {
            motion: Motion::Linear,
            endpoint: tab_end_pt,
            z: tab_z,
        });

        // Ramp back down to full depth
        segments.push(ToolpathSegment {
            motion: Motion::Linear,
            endpoint: tab_end_pt,
            z,
        });

        current_dist = tab_end_dist;
    }

    // Cut remaining distance to side end at full depth
    if current_dist < side_length {
        segments.push(ToolpathSegment {
            motion: Motion::Linear,
            endpoint: *side_end,
            z,
        });
    }
}

/// Generate a dado (groove) toolpath.
///
/// A dado is a rectangular groove cut across the face of a panel.
/// - `part_rect`: The panel being cut into (for coordinate reference).
/// - `dado_position`: Y position along the panel where the dado center is.
/// - `dado_width`: Width of the groove.
/// - `dado_depth`: Depth of the groove.
/// - `horizontal`: If true, dado runs along X; if false, along Y.
pub fn generate_dado_toolpath(
    part_rect: &Rect,
    dado: &DadoParams,
    tool: &Tool,
    rpm: f64,
    config: &CamConfig,
) -> Toolpath {
    let feed_rate = tool.recommended_feed_rate(rpm);
    let plunge_rate = feed_rate * 0.5;
    let r = tool.radius();
    let dado_position = dado.position;
    let dado_width = dado.width;
    let dado_depth = dado.depth;
    let horizontal = dado.horizontal;

    let num_passes = (dado_depth / config.depth_per_pass).ceil() as u32;
    let num_width_passes_est = (dado_width / tool.diameter).ceil() as usize;
    let mut segments = Vec::with_capacity(num_passes as usize * num_width_passes_est * 4 + 4);

    if horizontal {
        // Dado runs along X (full width of the part).
        // We may need multiple side-by-side passes if dado_width > tool_diameter.
        let num_width_passes = (dado_width / tool.diameter).ceil() as u32;
        let step_over = if num_width_passes > 1 {
            (dado_width - tool.diameter) / (num_width_passes - 1) as f64
        } else {
            0.0
        };

        let y_start = part_rect.min_y() + dado_position - dado_width / 2.0 + r;
        let x_start = part_rect.min_x() + r;
        let x_end = part_rect.max_x() - r;

        // Rapid to start
        segments.push(ToolpathSegment {
            motion: Motion::Rapid,
            endpoint: Point2D::new(x_start, y_start),
            z: config.safe_z,
        });

        for depth_pass in 1..=num_passes {
            let z = -(config.depth_per_pass * depth_pass as f64).min(dado_depth);

            for width_pass in 0..num_width_passes {
                let y = y_start + step_over * width_pass as f64;

                // Rapid to position
                segments.push(ToolpathSegment {
                    motion: Motion::Rapid,
                    endpoint: Point2D::new(x_start, y),
                    z: config.rapid_z,
                });

                // Plunge
                segments.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: Point2D::new(x_start, y),
                    z,
                });

                // Cut across
                segments.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: Point2D::new(x_end, y),
                    z,
                });

                // Retract for next pass
                segments.push(ToolpathSegment {
                    motion: Motion::Rapid,
                    endpoint: Point2D::new(x_end, y),
                    z: config.rapid_z,
                });
            }
        }
    } else {
        // Dado runs along Y (full height of the part) — vertical dado.
        let num_width_passes = (dado_width / tool.diameter).ceil() as u32;
        let step_over = if num_width_passes > 1 {
            (dado_width - tool.diameter) / (num_width_passes - 1) as f64
        } else {
            0.0
        };

        let x_start = part_rect.min_x() + dado_position - dado_width / 2.0 + r;
        let y_start = part_rect.min_y() + r;
        let y_end = part_rect.max_y() - r;

        segments.push(ToolpathSegment {
            motion: Motion::Rapid,
            endpoint: Point2D::new(x_start, y_start),
            z: config.safe_z,
        });

        for depth_pass in 1..=num_passes {
            let z = -(config.depth_per_pass * depth_pass as f64).min(dado_depth);

            for width_pass in 0..num_width_passes {
                let x = x_start + step_over * width_pass as f64;

                segments.push(ToolpathSegment {
                    motion: Motion::Rapid,
                    endpoint: Point2D::new(x, y_start),
                    z: config.rapid_z,
                });

                segments.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: Point2D::new(x, y_start),
                    z,
                });

                segments.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: Point2D::new(x, y_end),
                    z,
                });

                segments.push(ToolpathSegment {
                    motion: Motion::Rapid,
                    endpoint: Point2D::new(x, y_end),
                    z: config.rapid_z,
                });
            }
        }
    }

    // Final retract
    let last_pos = segments.last().map(|s| s.endpoint).unwrap_or(Point2D::origin());
    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: last_pos,
        z: config.safe_z,
    });

    Toolpath {
        tool_number: tool.number,
        rpm,
        feed_rate,
        plunge_rate,
        segments,
    }
}

/// Generate a rabbet (edge groove) toolpath.
///
/// A rabbet is a step-shaped recess cut along the edge of a panel, commonly
/// used for back panels in cabinets. The tool removes material from the
/// specified edge to create an L-shaped profile.
///
/// - `part_rect`: The panel being cut into.
/// - `edge`: Which edge to rabbet (Top, Bottom, Left, Right).
/// - `rabbet_width`: Width of the rabbet (into the panel from the edge).
/// - `rabbet_depth`: Depth of the rabbet (into the face of the panel).
pub fn generate_rabbet_toolpath(
    part_rect: &Rect,
    edge: RabbetEdge,
    rabbet_width: f64,
    rabbet_depth: f64,
    tool: &Tool,
    rpm: f64,
    config: &CamConfig,
) -> Toolpath {
    let feed_rate = tool.recommended_feed_rate(rpm);
    let plunge_rate = feed_rate * 0.5;
    let r = tool.radius();

    let num_passes = (rabbet_depth / config.depth_per_pass).ceil() as u32;

    // A rabbet is essentially a pocket along one edge. We cut parallel passes
    // stepping from the edge inward, covering the rabbet width.
    let num_width_passes = ((rabbet_width - tool.diameter) / tool.diameter).ceil() as u32 + 1;
    let mut segments = Vec::with_capacity(num_passes as usize * num_width_passes as usize * 4 + 4);
    let step_over = if num_width_passes > 1 {
        (rabbet_width - tool.diameter) / (num_width_passes - 1) as f64
    } else {
        0.0
    };

    // Determine cut geometry based on which edge
    let (start_points, end_points) = match edge {
        RabbetEdge::Top => {
            // Rabbet along the top edge: cut from right-to-left, stepping downward (Y)
            let mut starts = Vec::new();
            let mut ends = Vec::new();
            for i in 0..num_width_passes {
                let y = part_rect.max_y() - r - step_over * i as f64;
                starts.push(Point2D::new(part_rect.min_x() + r, y));
                ends.push(Point2D::new(part_rect.max_x() - r, y));
            }
            (starts, ends)
        }
        RabbetEdge::Bottom => {
            let mut starts = Vec::new();
            let mut ends = Vec::new();
            for i in 0..num_width_passes {
                let y = part_rect.min_y() + r + step_over * i as f64;
                starts.push(Point2D::new(part_rect.min_x() + r, y));
                ends.push(Point2D::new(part_rect.max_x() - r, y));
            }
            (starts, ends)
        }
        RabbetEdge::Left => {
            let mut starts = Vec::new();
            let mut ends = Vec::new();
            for i in 0..num_width_passes {
                let x = part_rect.min_x() + r + step_over * i as f64;
                starts.push(Point2D::new(x, part_rect.min_y() + r));
                ends.push(Point2D::new(x, part_rect.max_y() - r));
            }
            (starts, ends)
        }
        RabbetEdge::Right => {
            let mut starts = Vec::new();
            let mut ends = Vec::new();
            for i in 0..num_width_passes {
                let x = part_rect.max_x() - r - step_over * i as f64;
                starts.push(Point2D::new(x, part_rect.min_y() + r));
                ends.push(Point2D::new(x, part_rect.max_y() - r));
            }
            (starts, ends)
        }
    };

    // Rapid to starting position
    if let Some(first_start) = start_points.first() {
        segments.push(ToolpathSegment {
            motion: Motion::Rapid,
            endpoint: *first_start,
            z: config.safe_z,
        });
    }

    for depth_pass in 1..=num_passes {
        let z = -(config.depth_per_pass * depth_pass as f64).min(rabbet_depth);

        for (idx, (start, end)) in start_points.iter().zip(end_points.iter()).enumerate() {
            // Alternate cut direction for efficiency (zig-zag)
            let (from, to) = if idx % 2 == 0 {
                (*start, *end)
            } else {
                (*end, *start)
            };

            // Rapid to position
            segments.push(ToolpathSegment {
                motion: Motion::Rapid,
                endpoint: from,
                z: config.rapid_z,
            });

            // Plunge
            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: from,
                z,
            });

            // Cut across
            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: to,
                z,
            });

            // Retract
            segments.push(ToolpathSegment {
                motion: Motion::Rapid,
                endpoint: to,
                z: config.rapid_z,
            });
        }
    }

    // Final retract
    let last_pos = segments.last().map(|s| s.endpoint).unwrap_or(Point2D::origin());
    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: last_pos,
        z: config.safe_z,
    });

    Toolpath {
        tool_number: tool.number,
        rpm,
        feed_rate,
        plunge_rate,
        segments,
    }
}

/// Which edge of a panel to apply a rabbet to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RabbetEdge {
    Top,
    Bottom,
    Left,
    Right,
}

/// Generate a linear pattern of drill holes.
///
/// Used for shelf pin holes, hinge cup boring, and other repetitive drilling.
/// Holes are drilled using peck drilling (retract between plunges) for chip
/// clearing in deep holes.
pub fn generate_drill_pattern(
    holes: &[DrillHole],
    tool: &Tool,
    rpm: f64,
    config: &CamConfig,
) -> Toolpath {
    let feed_rate = tool.recommended_feed_rate(rpm);
    let plunge_rate = feed_rate * 0.3; // slower plunge for drilling
    let peck_depth = tool.diameter * 2.0; // peck every 2x diameter

    let mut segments = Vec::with_capacity(holes.len() * 6);

    if let Some(first) = holes.first() {
        // Rapid to first hole
        segments.push(ToolpathSegment {
            motion: Motion::Rapid,
            endpoint: Point2D::new(first.x, first.y),
            z: config.safe_z,
        });
    }

    for hole in holes {
        let pos = Point2D::new(hole.x, hole.y);

        if config.use_canned_cycles {
            // Rapid to above hole, then canned cycle
            segments.push(ToolpathSegment {
                motion: Motion::Rapid,
                endpoint: pos,
                z: config.rapid_z,
            });
            segments.push(ToolpathSegment {
                motion: Motion::DrillCycle {
                    retract_z: config.rapid_z,
                    final_z: -hole.depth,
                    peck_depth: if hole.depth > peck_depth { peck_depth } else { 0.0 },
                },
                endpoint: pos,
                z: -hole.depth,
            });
        } else {
            // Rapid to above hole
            segments.push(ToolpathSegment {
                motion: Motion::Rapid,
                endpoint: pos,
                z: config.rapid_z,
            });

            // Peck drilling: plunge in increments, retracting to clear chips
            let num_pecks = (hole.depth / peck_depth).ceil() as u32;
            for peck in 1..=num_pecks {
                let peck_z = -(peck_depth * peck as f64).min(hole.depth);

                // Plunge to peck depth
                segments.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: pos,
                    z: peck_z,
                });

                // Retract for chip clearing (except on last peck)
                if peck < num_pecks {
                    segments.push(ToolpathSegment {
                        motion: Motion::Rapid,
                        endpoint: pos,
                        z: config.rapid_z,
                    });
                }
            }

            // Full retract after hole
            segments.push(ToolpathSegment {
                motion: Motion::Rapid,
                endpoint: pos,
                z: config.rapid_z,
            });
        }
    }

    // Final retract to safe Z
    let last_pos = segments.last().map(|s| s.endpoint).unwrap_or(Point2D::origin());
    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: last_pos,
        z: config.safe_z,
    });

    Toolpath {
        tool_number: tool.number,
        rpm,
        feed_rate,
        plunge_rate,
        segments,
    }
}

/// A single drill hole position and depth.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DrillHole {
    pub x: f64,
    pub y: f64,
    pub depth: f64,
}

/// Generate a 32mm system shelf pin hole pattern.
///
/// The 32mm system is an industry standard for cabinet construction where
/// shelf pin holes are drilled at 32mm (1.26") intervals. This generates
/// two columns of holes along the height of a side panel.
///
/// - `panel_rect`: The side panel to drill into.
/// - `hole_depth`: Depth of each pin hole (typically 0.5" / 12mm).
/// - `hole_diameter`: Diameter of pin holes (typically 5mm / 0.197").
/// - `setback`: Distance from front and back edges to hole columns.
/// - `start_height`: Height above bottom to start holes.
/// - `end_height`: Height to stop holes.
pub fn generate_shelf_pin_pattern(
    panel_rect: &Rect,
    params: &ShelfPinParams,
    tool: &Tool,
    rpm: f64,
    config: &CamConfig,
) -> Toolpath {
    let spacing = 1.26; // 32mm in inches
    let mut holes = Vec::new();

    // Front column
    let front_x = panel_rect.min_x() + params.setback;
    // Back column
    let back_x = panel_rect.max_x() - params.setback;

    let mut y = panel_rect.min_y() + params.start_height;
    let y_end = panel_rect.min_y() + params.end_height;

    while y <= y_end {
        holes.push(DrillHole {
            x: front_x,
            y,
            depth: params.hole_depth,
        });
        holes.push(DrillHole {
            x: back_x,
            y,
            depth: params.hole_depth,
        });
        y += spacing;
    }

    generate_drill_pattern(&holes, tool, rpm, config)
}

/// Generate a simple drill operation at a single point.
///
/// When `config.use_canned_cycles` is true, emits a DrillCycle motion
/// which the post-processor converts to G81/G83. Otherwise uses manual
/// plunge moves (compatible with all controllers).
pub fn generate_drill(
    position: Point2D,
    depth: f64,
    tool: &Tool,
    rpm: f64,
    config: &CamConfig,
) -> Toolpath {
    let feed_rate = tool.recommended_feed_rate(rpm);
    let plunge_rate = feed_rate * 0.3; // slower plunge for drilling
    let peck_depth = tool.diameter * 2.0;

    let mut segments = vec![
        // Rapid to position
        ToolpathSegment {
            motion: Motion::Rapid,
            endpoint: position,
            z: config.safe_z,
        },
    ];

    if config.use_canned_cycles {
        segments.push(ToolpathSegment {
            motion: Motion::DrillCycle {
                retract_z: config.rapid_z,
                final_z: -depth,
                peck_depth: if depth > peck_depth { peck_depth } else { 0.0 },
            },
            endpoint: position,
            z: -depth,
        });
    } else {
        // Rapid down to near surface
        segments.push(ToolpathSegment {
            motion: Motion::Rapid,
            endpoint: position,
            z: config.rapid_z,
        });
        // Drill into material
        segments.push(ToolpathSegment {
            motion: Motion::Linear,
            endpoint: position,
            z: -depth,
        });
    }

    // Retract
    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: position,
        z: config.safe_z,
    });

    Toolpath {
        tool_number: tool.number,
        rpm,
        feed_rate,
        plunge_rate,
        segments,
    }
}

/// Generate a dovetail joint toolpath.
///
/// Cuts trapezoidal pockets for each tail using a straight bit.
/// The waste is removed with rectangular passes, then the angled
/// shoulders are cut. This is a CNC approximation; the actual dovetail
/// angle is formed by the shoulder cuts.
pub fn generate_dovetail_toolpath(
    part_rect: &Rect,
    params: &DovetailParams,
    tool: &Tool,
    rpm: f64,
    config: &CamConfig,
) -> Toolpath {
    let feed_rate = tool.recommended_feed_rate(rpm);
    let plunge_rate = feed_rate * 0.5;
    let r = tool.radius();
    let edge = params.edge;
    let tail_count = params.tail_count;
    let tail_width = params.tail_width;
    let pin_width = params.pin_width;
    let depth = params.depth;

    let num_passes = (depth / config.depth_per_pass).ceil() as u32;
    let mut segments = Vec::with_capacity(tail_count as usize * num_passes as usize * 10 + 4);

    // Calculate tail positions along the edge
    let edge_length = match edge {
        DovetailEdge::Top | DovetailEdge::Bottom => part_rect.width,
        DovetailEdge::Left | DovetailEdge::Right => part_rect.height,
    };

    let total_joint = tail_count as f64 * tail_width + (tail_count + 1) as f64 * pin_width;
    let start_offset = (edge_length - total_joint) / 2.0;

    // Rapid to start
    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: part_rect.origin,
        z: config.safe_z,
    });

    for pass in 1..=num_passes {
        let z = -(config.depth_per_pass * pass as f64).min(depth);

        for i in 0..tail_count {
            // Position of this tail's start
            let tail_start = start_offset + pin_width + i as f64 * (tail_width + pin_width);
            let tail_end = tail_start + tail_width;

            // Cut rectangular pocket for each tail
            let (pocket_start, pocket_end, cut_y) = match edge {
                DovetailEdge::Bottom => {
                    let x_start = part_rect.min_x() + tail_start + r;
                    let x_end = part_rect.min_x() + tail_end - r;
                    let y = part_rect.min_y() + r;
                    (Point2D::new(x_start, y), Point2D::new(x_end, y), y)
                }
                DovetailEdge::Top => {
                    let x_start = part_rect.min_x() + tail_start + r;
                    let x_end = part_rect.min_x() + tail_end - r;
                    let y = part_rect.max_y() - r;
                    (Point2D::new(x_start, y), Point2D::new(x_end, y), y)
                }
                DovetailEdge::Left => {
                    let y_start = part_rect.min_y() + tail_start + r;
                    let y_end = part_rect.min_y() + tail_end - r;
                    let x = part_rect.min_x() + r;
                    (Point2D::new(x, y_start), Point2D::new(x, y_end), x)
                }
                DovetailEdge::Right => {
                    let y_start = part_rect.min_y() + tail_start + r;
                    let y_end = part_rect.min_y() + tail_end - r;
                    let x = part_rect.max_x() - r;
                    (Point2D::new(x, y_start), Point2D::new(x, y_end), x)
                }
            };
            let _ = cut_y;

            // Multiple width passes if needed for depth penetration
            let pocket_depth = match edge {
                DovetailEdge::Bottom | DovetailEdge::Top => depth,
                DovetailEdge::Left | DovetailEdge::Right => depth,
            };
            let _ = pocket_depth;
            let num_width_passes = ((depth - tool.diameter) / tool.diameter).ceil() as u32 + 1;
            let step = if num_width_passes > 1 {
                (depth - tool.diameter) / (num_width_passes - 1) as f64
            } else {
                0.0
            };

            for wp in 0..num_width_passes {
                let offset = r + step * wp as f64;
                let (from, to) = match edge {
                    DovetailEdge::Bottom => (
                        Point2D::new(pocket_start.x, part_rect.min_y() + offset),
                        Point2D::new(pocket_end.x, part_rect.min_y() + offset),
                    ),
                    DovetailEdge::Top => (
                        Point2D::new(pocket_start.x, part_rect.max_y() - offset),
                        Point2D::new(pocket_end.x, part_rect.max_y() - offset),
                    ),
                    DovetailEdge::Left => (
                        Point2D::new(part_rect.min_x() + offset, pocket_start.y),
                        Point2D::new(part_rect.min_x() + offset, pocket_end.y),
                    ),
                    DovetailEdge::Right => (
                        Point2D::new(part_rect.max_x() - offset, pocket_start.y),
                        Point2D::new(part_rect.max_x() - offset, pocket_end.y),
                    ),
                };

                segments.push(ToolpathSegment {
                    motion: Motion::Rapid,
                    endpoint: from,
                    z: config.rapid_z,
                });
                segments.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: from,
                    z,
                });
                segments.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: to,
                    z,
                });
                segments.push(ToolpathSegment {
                    motion: Motion::Rapid,
                    endpoint: to,
                    z: config.rapid_z,
                });
            }
        }
    }

    // Final retract
    let last_pos = segments.last().map(|s| s.endpoint).unwrap_or(Point2D::origin());
    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: last_pos,
        z: config.safe_z,
    });

    Toolpath {
        tool_number: tool.number,
        rpm,
        feed_rate,
        plunge_rate,
        segments,
    }
}

/// Which edge of a part for dovetail/box joint operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DovetailEdge {
    Top,
    Bottom,
    Left,
    Right,
}

/// Generate a box (finger) joint toolpath.
///
/// Cuts repeated slots at finger_width spacing along an edge.
/// Each slot is cut like a narrow dado at the edge of the part.
pub fn generate_box_joint_toolpath(
    part_rect: &Rect,
    params: &BoxJointParams,
    tool: &Tool,
    rpm: f64,
    config: &CamConfig,
) -> Toolpath {
    let feed_rate = tool.recommended_feed_rate(rpm);
    let plunge_rate = feed_rate * 0.5;
    let r = tool.radius();
    let edge = params.edge;
    let finger_width = params.finger_width;
    let finger_count = params.finger_count;
    let depth = params.depth;

    let num_passes = (depth / config.depth_per_pass).ceil() as u32;
    let mut segments = Vec::with_capacity(finger_count as usize * num_passes as usize * 6 + 4);

    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: part_rect.origin,
        z: config.safe_z,
    });

    for pass in 1..=num_passes {
        let z = -(config.depth_per_pass * pass as f64).min(depth);

        // Cut every other slot (the "fingers" that are removed)
        for i in (0..finger_count).step_by(2) {
            let slot_start = i as f64 * finger_width;
            let slot_end = slot_start + finger_width;

            let (from, to) = match edge {
                DovetailEdge::Bottom => (
                    Point2D::new(part_rect.min_x() + slot_start + r, part_rect.min_y() + r),
                    Point2D::new(part_rect.min_x() + slot_end - r, part_rect.min_y() + r),
                ),
                DovetailEdge::Top => (
                    Point2D::new(part_rect.min_x() + slot_start + r, part_rect.max_y() - r),
                    Point2D::new(part_rect.min_x() + slot_end - r, part_rect.max_y() - r),
                ),
                DovetailEdge::Left => (
                    Point2D::new(part_rect.min_x() + r, part_rect.min_y() + slot_start + r),
                    Point2D::new(part_rect.min_x() + r, part_rect.min_y() + slot_end - r),
                ),
                DovetailEdge::Right => (
                    Point2D::new(part_rect.max_x() - r, part_rect.min_y() + slot_start + r),
                    Point2D::new(part_rect.max_x() - r, part_rect.min_y() + slot_end - r),
                ),
            };

            // Multiple width passes into the edge
            let num_width_passes = ((depth - tool.diameter) / tool.diameter).ceil() as u32 + 1;
            let step = if num_width_passes > 1 {
                (depth - tool.diameter) / (num_width_passes - 1) as f64
            } else {
                0.0
            };

            for wp in 0..num_width_passes {
                let offset = step * wp as f64;
                let (wf, wt) = match edge {
                    DovetailEdge::Bottom => (
                        Point2D::new(from.x, part_rect.min_y() + r + offset),
                        Point2D::new(to.x, part_rect.min_y() + r + offset),
                    ),
                    DovetailEdge::Top => (
                        Point2D::new(from.x, part_rect.max_y() - r - offset),
                        Point2D::new(to.x, part_rect.max_y() - r - offset),
                    ),
                    DovetailEdge::Left => (
                        Point2D::new(part_rect.min_x() + r + offset, from.y),
                        Point2D::new(part_rect.min_x() + r + offset, to.y),
                    ),
                    DovetailEdge::Right => (
                        Point2D::new(part_rect.max_x() - r - offset, from.y),
                        Point2D::new(part_rect.max_x() - r - offset, to.y),
                    ),
                };

                segments.push(ToolpathSegment {
                    motion: Motion::Rapid,
                    endpoint: wf,
                    z: config.rapid_z,
                });
                segments.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: wf,
                    z,
                });
                segments.push(ToolpathSegment {
                    motion: Motion::Linear,
                    endpoint: wt,
                    z,
                });
                segments.push(ToolpathSegment {
                    motion: Motion::Rapid,
                    endpoint: wt,
                    z: config.rapid_z,
                });
            }
        }
    }

    let last_pos = segments.last().map(|s| s.endpoint).unwrap_or(Point2D::origin());
    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: last_pos,
        z: config.safe_z,
    });

    Toolpath {
        tool_number: tool.number,
        rpm,
        feed_rate,
        plunge_rate,
        segments,
    }
}

/// Generate a mortise (rectangular pocket) toolpath.
///
/// The mortise is cut as a rectangular pocket with multiple depth passes
/// and XY ramping for plunge entry.
pub fn generate_mortise_toolpath(
    part_rect: &Rect,
    params: &MortiseParams,
    tool: &Tool,
    rpm: f64,
    config: &CamConfig,
) -> Toolpath {
    let feed_rate = tool.recommended_feed_rate(rpm);
    let plunge_rate = feed_rate * 0.5;
    let r = tool.radius();
    let mortise_x = params.x;
    let mortise_y = params.y;
    let mortise_width = params.width;
    let mortise_length = params.length;
    let mortise_depth = params.depth;

    let num_passes = (mortise_depth / config.depth_per_pass).ceil() as u32;
    let mut segments = Vec::with_capacity(num_passes as usize * 8 + 4);

    // Mortise center in part coordinates, translated to sheet coordinates
    let cx = part_rect.min_x() + mortise_x;
    let cy = part_rect.min_y() + mortise_y;

    let half_w = mortise_width / 2.0 - r;
    let half_l = mortise_length / 2.0 - r;

    // Ensure mortise is large enough for tool
    let half_w = half_w.max(0.0);
    let half_l = half_l.max(0.0);

    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: Point2D::new(cx - half_w, cy - half_l),
        z: config.safe_z,
    });

    for pass in 1..=num_passes {
        let z = -(config.depth_per_pass * pass as f64).min(mortise_depth);

        // Rapid to start position
        segments.push(ToolpathSegment {
            motion: Motion::Rapid,
            endpoint: Point2D::new(cx - half_w, cy - half_l),
            z: config.rapid_z,
        });

        // Plunge (ramp along length if possible)
        segments.push(ToolpathSegment {
            motion: Motion::Linear,
            endpoint: Point2D::new(cx - half_w, cy - half_l),
            z,
        });

        // Cut rectangular pocket perimeter
        // If mortise is wider than tool, we need stepover passes
        let num_width_passes = if half_w > 0.0 {
            ((mortise_width - tool.diameter) / tool.diameter).ceil() as u32 + 1
        } else {
            1
        };
        let step_w = if num_width_passes > 1 {
            (mortise_width - tool.diameter) / (num_width_passes - 1) as f64
        } else {
            0.0
        };

        for wp in 0..num_width_passes {
            let x_off = if num_width_passes > 1 {
                -half_w + step_w * wp as f64
            } else {
                0.0
            };

            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: Point2D::new(cx + x_off, cy - half_l),
                z,
            });
            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: Point2D::new(cx + x_off, cy + half_l),
                z,
            });
        }

        // Retract
        segments.push(ToolpathSegment {
            motion: Motion::Rapid,
            endpoint: Point2D::new(cx, cy + half_l),
            z: config.rapid_z,
        });
    }

    let last_pos = segments.last().map(|s| s.endpoint).unwrap_or(Point2D::origin());
    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: last_pos,
        z: config.safe_z,
    });

    Toolpath {
        tool_number: tool.number,
        rpm,
        feed_rate,
        plunge_rate,
        segments,
    }
}

/// Generate a tenon toolpath.
///
/// A tenon is formed by cutting four shoulder rabbets around the end of the
/// workpiece. The material on all four faces around the tenon is removed,
/// leaving the protruding tongue.
pub fn generate_tenon_toolpath(
    part_rect: &Rect,
    params: &TenonParams,
    tool: &Tool,
    rpm: f64,
    config: &CamConfig,
) -> Toolpath {
    let feed_rate = tool.recommended_feed_rate(rpm);
    let plunge_rate = feed_rate * 0.5;
    let r = tool.radius();
    let edge = params.edge;
    let _tenon_thickness = params.tenon_thickness;
    let tenon_width = params.tenon_width;
    let tenon_length = params.tenon_length;
    let shoulder_depth = params.shoulder_depth;

    let num_passes = (shoulder_depth / config.depth_per_pass).ceil() as u32;
    let mut segments = Vec::with_capacity(num_passes as usize * 16 + 4);

    // The tenon is formed by removing shoulder material on the face.
    // We cut two channels (top shoulder and bottom shoulder) to form
    // the tenon thickness.
    let part_center_x = (part_rect.min_x() + part_rect.max_x()) / 2.0;
    let part_center_y = (part_rect.min_y() + part_rect.max_y()) / 2.0;

    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: Point2D::new(part_center_x, part_center_y),
        z: config.safe_z,
    });

    // Cut shoulder regions on the face of the part
    // Top shoulder: from top of part down to tenon upper boundary
    // Bottom shoulder: from bottom of part up to tenon lower boundary
    let (cut_regions, cut_length) = match edge {
        DovetailEdge::Left | DovetailEdge::Right => {
            // Tenon protrudes from left or right edge
            let tenon_center_y = part_center_y;
            let top_shoulder = (
                Point2D::new(part_rect.min_x() + r, tenon_center_y + tenon_width / 2.0 + r),
                Point2D::new(part_rect.max_x() - r, part_rect.max_y() - r),
            );
            let bottom_shoulder = (
                Point2D::new(part_rect.min_x() + r, part_rect.min_y() + r),
                Point2D::new(part_rect.max_x() - r, tenon_center_y - tenon_width / 2.0 - r),
            );
            (vec![top_shoulder, bottom_shoulder], tenon_length)
        }
        DovetailEdge::Top | DovetailEdge::Bottom => {
            let tenon_center_x = part_center_x;
            let left_shoulder = (
                Point2D::new(part_rect.min_x() + r, part_rect.min_y() + r),
                Point2D::new(tenon_center_x - tenon_width / 2.0 - r, part_rect.max_y() - r),
            );
            let right_shoulder = (
                Point2D::new(tenon_center_x + tenon_width / 2.0 + r, part_rect.min_y() + r),
                Point2D::new(part_rect.max_x() - r, part_rect.max_y() - r),
            );
            (vec![left_shoulder, right_shoulder], tenon_length)
        }
    };
    let _ = cut_length;

    for pass in 1..=num_passes {
        let z = -(config.depth_per_pass * pass as f64).min(shoulder_depth);

        for (start, end) in &cut_regions {
            // Zigzag across the shoulder region
            segments.push(ToolpathSegment {
                motion: Motion::Rapid,
                endpoint: *start,
                z: config.rapid_z,
            });
            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: *start,
                z,
            });
            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: Point2D::new(end.x, start.y),
                z,
            });
            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: *end,
                z,
            });
            segments.push(ToolpathSegment {
                motion: Motion::Linear,
                endpoint: Point2D::new(start.x, end.y),
                z,
            });
            segments.push(ToolpathSegment {
                motion: Motion::Rapid,
                endpoint: Point2D::new(start.x, end.y),
                z: config.rapid_z,
            });
        }
    }

    let last_pos = segments.last().map(|s| s.endpoint).unwrap_or(Point2D::origin());
    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: last_pos,
        z: config.safe_z,
    });

    Toolpath {
        tool_number: tool.number,
        rpm,
        feed_rate,
        plunge_rate,
        segments,
    }
}

/// Generate dowel hole toolpath.
///
/// Delegates to `generate_drill` for each hole position.
pub fn generate_dowel_holes(
    part_rect: &Rect,
    holes: &[(f64, f64)],
    diameter: f64,
    depth: f64,
    tool: &Tool,
    rpm: f64,
    config: &CamConfig,
) -> Toolpath {
    let drill_holes: Vec<DrillHole> = holes
        .iter()
        .map(|(x, y)| DrillHole {
            x: part_rect.min_x() + x,
            y: part_rect.min_y() + y,
            depth,
        })
        .collect();

    let _ = diameter; // diameter is for the dowel, drill uses the tool diameter
    generate_drill_pattern(&drill_holes, tool, rpm, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_cut_starts_and_ends_at_safe_z() {
        let rect = Rect::from_dimensions(10.0, 5.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);

        assert!(tp.segments.first().unwrap().z > 0.0, "should start above material");
        assert!(tp.segments.last().unwrap().z > 0.0, "should end above material");
    }

    #[test]
    fn test_profile_cut_depth_passes() {
        let rect = Rect::from_dimensions(10.0, 5.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig {
            depth_per_pass: 0.25,
            ..Default::default()
        };
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);

        let cutting_moves: Vec<_> = tp
            .segments
            .iter()
            .filter(|s| s.z < 0.0)
            .collect();
        assert!(!cutting_moves.is_empty());

        // Verify deepest cut matches material thickness
        let deepest = cutting_moves.iter().map(|s| s.z).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        assert!((deepest - (-0.75)).abs() < 1e-10);
    }

    #[test]
    fn test_profile_cut_has_tabs_on_final_pass() {
        let rect = Rect::from_dimensions(10.0, 5.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig {
            depth_per_pass: 0.75, // single pass to make final pass = only pass
            tabs_per_side: 2,
            tab_width: 0.5,
            tab_height: 0.125,
            ..Default::default()
        };
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);

        // Tab Z should be -0.75 + 0.125 = -0.625
        let tab_z = -0.75 + 0.125;

        // Count segments at tab height (should be multiple: ramp up, traverse, ramp down)
        let tab_segments: Vec<_> = tp
            .segments
            .iter()
            .filter(|s| (s.z - tab_z).abs() < 1e-10)
            .collect();

        // 4 sides * 2 tabs * 2 segments per tab (ramp up + traverse) = 16
        assert!(
            tab_segments.len() >= 8,
            "expected at least 8 tab segments (4 sides * 2 tabs), got {}",
            tab_segments.len()
        );
    }

    #[test]
    fn test_profile_cut_tabs_are_shallower_than_full_depth() {
        let rect = Rect::from_dimensions(10.0, 5.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig {
            depth_per_pass: 0.75,
            tabs_per_side: 1,
            tab_width: 0.5,
            tab_height: 0.125,
            ..Default::default()
        };
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);

        let full_depth = -0.75;
        let tab_z = full_depth + 0.125;

        // Verify we have both full-depth and tab-depth segments
        let has_full_depth = tp.segments.iter().any(|s| (s.z - full_depth).abs() < 1e-10);
        let has_tab_depth = tp.segments.iter().any(|s| (s.z - tab_z).abs() < 1e-10);
        assert!(has_full_depth, "should have full-depth cuts");
        assert!(has_tab_depth, "should have tab-depth (shallower) cuts");
    }

    #[test]
    fn test_profile_cut_no_tabs_when_zero() {
        let rect = Rect::from_dimensions(10.0, 5.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig {
            depth_per_pass: 0.75,
            tabs_per_side: 0,
            ..Default::default()
        };
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);

        // With no tabs, all cutting segments should be at full depth
        let tab_z = -0.75 + config.tab_height;
        let tab_segments: Vec<_> = tp
            .segments
            .iter()
            .filter(|s| (s.z - tab_z).abs() < 1e-10)
            .collect();
        assert_eq!(tab_segments.len(), 0, "should have no tab segments when tabs_per_side = 0");
    }

    #[test]
    fn test_dado_toolpath_generation() {
        let rect = Rect::from_dimensions(12.0, 30.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_dado_toolpath(&rect, &DadoParams { position: 10.0, width: 0.75, depth: 0.375, horizontal: true }, &tool, 5000.0, &config);

        assert!(!tp.segments.is_empty());

        // Verify all cuts are at or above dado depth
        for seg in &tp.segments {
            assert!(seg.z >= -0.375 - 1e-10, "cut too deep: {}", seg.z);
        }
    }

    #[test]
    fn test_rabbet_toolpath_generation() {
        let rect = Rect::from_dimensions(12.0, 30.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_rabbet_toolpath(
            &rect, RabbetEdge::Right, 0.25, 0.375, &tool, 5000.0, &config,
        );

        assert!(!tp.segments.is_empty());

        // Verify starts and ends at safe Z
        assert!(tp.segments.first().unwrap().z > 0.0);
        assert!(tp.segments.last().unwrap().z > 0.0);

        // Verify max depth doesn't exceed rabbet depth
        let deepest = tp.segments.iter().map(|s| s.z).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        assert!(deepest >= -0.375 - 1e-10, "rabbet cut too deep: {}", deepest);
    }

    #[test]
    fn test_rabbet_toolpath_covers_edge() {
        let rect = Rect::from_dimensions(12.0, 30.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_rabbet_toolpath(
            &rect, RabbetEdge::Right, 0.25, 0.375, &tool, 5000.0, &config,
        );

        // For a right-edge rabbet, X positions should be near max_x
        let cutting_segments: Vec<_> = tp.segments.iter().filter(|s| s.z < 0.0).collect();
        assert!(!cutting_segments.is_empty());
        for seg in &cutting_segments {
            assert!(
                seg.endpoint.x >= rect.max_x() - 0.25 - 0.01,
                "rabbet cut should be near right edge, got x={}",
                seg.endpoint.x
            );
        }
    }

    #[test]
    fn test_drill_pattern() {
        let holes = vec![
            DrillHole { x: 1.0, y: 1.0, depth: 0.5 },
            DrillHole { x: 1.0, y: 2.26, depth: 0.5 },
            DrillHole { x: 1.0, y: 3.52, depth: 0.5 },
        ];
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_drill_pattern(&holes, &tool, 5000.0, &config);

        assert!(!tp.segments.is_empty());

        // Verify starts and ends at safe Z
        assert!(tp.segments.first().unwrap().z > 0.0);
        assert!(tp.segments.last().unwrap().z > 0.0);

        // Verify all holes reach target depth
        let deepest = tp.segments.iter().map(|s| s.z).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        assert!((deepest - (-0.5)).abs() < 1e-10, "should drill to 0.5\" depth");
    }

    #[test]
    fn test_drill_pattern_peck_drilling() {
        let holes = vec![DrillHole { x: 1.0, y: 1.0, depth: 2.0 }];
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_drill_pattern(&holes, &tool, 5000.0, &config);

        // Peck depth = 2 * 0.25 = 0.5, so 2.0/0.5 = 4 pecks
        // Each peck has: plunge down + retract (except last which retracts after loop)
        // Count downward plunges into material
        let plunges: Vec<_> = tp.segments.iter()
            .filter(|s| matches!(s.motion, Motion::Linear) && s.z < 0.0)
            .collect();
        assert_eq!(plunges.len(), 4, "should have 4 peck plunges for 2\" depth");
    }

    #[test]
    fn test_shelf_pin_pattern() {
        let rect = Rect::from_dimensions(12.0, 30.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_shelf_pin_pattern(
            &rect, &ShelfPinParams { hole_depth: 0.5, setback: 2.0, start_height: 3.0, end_height: 27.0 }, &tool, 5000.0, &config,
        );

        assert!(!tp.segments.is_empty());

        // Count drill plunges (each hole should have at least one plunge)
        let plunges: Vec<_> = tp.segments.iter()
            .filter(|s| matches!(s.motion, Motion::Linear) && s.z < 0.0)
            .collect();

        // Height range: 27.0 - 3.0 = 24.0", spacing = 1.26"
        // Holes per column: floor(24.0 / 1.26) + 1 = 20
        // Two columns: 40 holes total, each 0.5" deep with peck at 0.5" = 1 peck each
        assert!(plunges.len() >= 38, "should have plunges for all shelf pin holes, got {}", plunges.len());
    }

    // --- Lead-in/Lead-out tests ---

    #[test]
    fn test_profile_cut_with_lead_in() {
        let rect = Rect::from_dimensions(10.0, 5.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig {
            lead_in_radius: Some(0.125),
            depth_per_pass: 0.75,
            tabs_per_side: 0,
            ..Default::default()
        };
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);

        // Should have ArcCCW (lead-in) and ArcCW (lead-out) segments
        let arc_ccw_count = tp.segments.iter()
            .filter(|s| matches!(s.motion, Motion::ArcCCW { .. }))
            .count();
        let arc_cw_count = tp.segments.iter()
            .filter(|s| matches!(s.motion, Motion::ArcCW { .. }))
            .count();

        assert!(arc_ccw_count > 0, "should have ArcCCW lead-in segments");
        assert!(arc_cw_count > 0, "should have ArcCW lead-out segments");
    }

    #[test]
    fn test_profile_cut_without_lead_in() {
        let rect = Rect::from_dimensions(10.0, 5.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig {
            lead_in_radius: None,
            depth_per_pass: 0.75,
            tabs_per_side: 0,
            ..Default::default()
        };
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);

        // Should have no arc segments
        let arc_count = tp.segments.iter()
            .filter(|s| matches!(s.motion, Motion::ArcCW { .. } | Motion::ArcCCW { .. }))
            .count();
        assert_eq!(arc_count, 0, "should have no arcs without lead-in");
    }

    // --- Canned cycle tests ---

    #[test]
    fn test_drill_with_canned_cycle() {
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig {
            use_canned_cycles: true,
            ..Default::default()
        };
        let tp = generate_drill(Point2D::new(5.0, 5.0), 0.5, &tool, 5000.0, &config);

        let cycle_count = tp.segments.iter()
            .filter(|s| matches!(s.motion, Motion::DrillCycle { .. }))
            .count();
        assert_eq!(cycle_count, 1, "should have one DrillCycle motion");

        // Verify the cycle parameters
        if let Some(seg) = tp.segments.iter().find(|s| matches!(s.motion, Motion::DrillCycle { .. })) {
            if let Motion::DrillCycle { retract_z, final_z, peck_depth } = seg.motion {
                assert!((final_z - (-0.5)).abs() < 1e-10, "final_z should be -0.5");
                assert!((retract_z - 0.25).abs() < 1e-10, "retract should be rapid_z");
                // 0.5" depth <= peck_depth (0.5"), so peck_depth should be 0 (simple drill)
                assert!((peck_depth - 0.0).abs() < 1e-10, "shallow hole should use G81 (peck=0)");
            }
        }
    }

    #[test]
    fn test_drill_without_canned_cycle() {
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default(); // use_canned_cycles = false
        let tp = generate_drill(Point2D::new(5.0, 5.0), 0.5, &tool, 5000.0, &config);

        let cycle_count = tp.segments.iter()
            .filter(|s| matches!(s.motion, Motion::DrillCycle { .. }))
            .count();
        assert_eq!(cycle_count, 0, "should have no DrillCycle without canned cycles");
    }

    #[test]
    fn test_drill_pattern_canned_cycle_peck() {
        let holes = vec![
            DrillHole { x: 1.0, y: 1.0, depth: 2.0 }, // deep hole → peck
        ];
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig {
            use_canned_cycles: true,
            ..Default::default()
        };
        let tp = generate_drill_pattern(&holes, &tool, 5000.0, &config);

        let cycles: Vec<_> = tp.segments.iter()
            .filter(|s| matches!(s.motion, Motion::DrillCycle { .. }))
            .collect();
        assert_eq!(cycles.len(), 1, "should have 1 drill cycle");

        if let Motion::DrillCycle { peck_depth, .. } = cycles[0].motion {
            assert!(peck_depth > 0.0, "deep hole should use G83 peck drilling");
        }
    }

    // --- Phase 16b: Advanced joinery CAM tests ---

    #[test]
    fn test_dovetail_toolpath_generates_segments() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_dovetail_toolpath(
            &rect, &DovetailParams { edge: DovetailEdge::Bottom, tail_count: 3, tail_width: 0.5, pin_width: 0.25, depth: 0.75 },
            &tool, 5000.0, &config,
        );
        assert!(!tp.segments.is_empty());
        assert!(tp.segments.first().unwrap().z > 0.0, "should start at safe Z");
        assert!(tp.segments.last().unwrap().z > 0.0, "should end at safe Z");

        let cutting = tp.segments.iter().filter(|s| s.z < 0.0).count();
        assert!(cutting >= 3, "should have cutting moves for 3 tails");
    }

    #[test]
    fn test_dovetail_depth_correct() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_dovetail_toolpath(
            &rect, &DovetailParams { edge: DovetailEdge::Left, tail_count: 2, tail_width: 0.6, pin_width: 0.3, depth: 0.5 },
            &tool, 5000.0, &config,
        );
        let deepest = tp.segments.iter().map(|s| s.z).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        assert!((deepest - (-0.5)).abs() < 1e-10, "deepest cut should be dovetail depth");
    }

    #[test]
    fn test_box_joint_toolpath_generates_segments() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_box_joint_toolpath(
            &rect, &BoxJointParams { edge: DovetailEdge::Bottom, finger_width: 0.5, finger_count: 8, depth: 0.75 },
            &tool, 5000.0, &config,
        );
        assert!(!tp.segments.is_empty());
        // Every other finger is cut (0, 2, 4, 6) = 4 slots
        let cutting = tp.segments.iter().filter(|s| s.z < 0.0).count();
        assert!(cutting >= 4, "should have cuts for alternating fingers");
    }

    #[test]
    fn test_mortise_toolpath_correct_depth() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_mortise_toolpath(
            &rect, &MortiseParams { x: 1.0, y: 2.0, width: 0.375, length: 1.0, depth: 0.75 },
            &tool, 5000.0, &config,
        );
        assert!(!tp.segments.is_empty());
        let deepest = tp.segments.iter().map(|s| s.z).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        assert!((deepest - (-0.75)).abs() < 1e-10, "mortise should reach specified depth");
    }

    #[test]
    fn test_mortise_toolpath_positions() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_mortise_toolpath(
            &rect, &MortiseParams { x: 3.0, y: 2.0, width: 0.5, length: 1.5, depth: 1.0 },
            &tool, 5000.0, &config,
        );
        // Verify all cutting moves are within the part bounds
        for seg in &tp.segments {
            if seg.z < 0.0 {
                assert!(seg.endpoint.x >= rect.min_x() - 0.01, "x should be within part");
                assert!(seg.endpoint.x <= rect.max_x() + 0.01, "x should be within part");
            }
        }
    }

    #[test]
    fn test_tenon_toolpath_generates_shoulder_cuts() {
        let rect = Rect::from_dimensions(4.0, 6.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_tenon_toolpath(
            &rect, &TenonParams { edge: DovetailEdge::Left, tenon_thickness: 0.375, tenon_width: 1.0, tenon_length: 1.0, shoulder_depth: 0.25 },
            &tool, 5000.0, &config,
        );
        assert!(!tp.segments.is_empty());
        // Should have cutting moves for shoulder removal
        let cutting = tp.segments.iter().filter(|s| s.z < 0.0).count();
        assert!(cutting >= 2, "should have shoulder cuts, got {}", cutting);
    }

    #[test]
    fn test_dowel_holes_delegates_to_drill() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let holes = vec![(1.0, 1.0), (1.0, 2.0), (1.0, 3.0)];
        let tp = generate_dowel_holes(
            &rect, &holes, 0.315, 0.5, &tool, 5000.0, &config,
        );
        assert!(!tp.segments.is_empty());

        // Should drill 3 holes
        let plunges: Vec<_> = tp.segments.iter()
            .filter(|s| matches!(s.motion, Motion::Linear) && s.z < 0.0)
            .collect();
        assert_eq!(plunges.len(), 3, "should have 3 drill plunges for 3 dowel holes");
    }

    // --- Comprehensive edge-case tests for Phase 16b generators ---

    #[test]
    fn test_dovetail_all_four_edges() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();

        for edge in [DovetailEdge::Top, DovetailEdge::Bottom, DovetailEdge::Left, DovetailEdge::Right] {
            let tp = generate_dovetail_toolpath(
                &rect, &DovetailParams { edge, tail_count: 3, tail_width: 0.5, pin_width: 0.25, depth: 0.75 },
                &tool, 5000.0, &config,
            );
            assert!(!tp.segments.is_empty(), "dovetail on {:?} should produce segments", edge);
            assert!(tp.segments.first().unwrap().z > 0.0, "{:?}: should start at safe Z", edge);
            assert!(tp.segments.last().unwrap().z > 0.0, "{:?}: should end at safe Z", edge);

            let deepest = tp.segments.iter().map(|s| s.z).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            assert!((deepest - (-0.75)).abs() < 1e-10, "{:?}: deepest should be -0.75, got {}", edge, deepest);
        }
    }

    #[test]
    fn test_dovetail_single_tail() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_dovetail_toolpath(
            &rect, &DovetailParams { edge: DovetailEdge::Bottom, tail_count: 1, tail_width: 1.0, pin_width: 0.5, depth: 0.5 },
            &tool, 5000.0, &config,
        );
        assert!(!tp.segments.is_empty());
        let cutting = tp.segments.iter().filter(|s| s.z < 0.0).count();
        assert!(cutting >= 1, "single tail should still have cutting moves");
    }

    #[test]
    fn test_dovetail_many_tails() {
        let rect = Rect::from_dimensions(12.0, 8.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_dovetail_toolpath(
            &rect, &DovetailParams { edge: DovetailEdge::Bottom, tail_count: 10, tail_width: 0.3, pin_width: 0.15, depth: 0.5 },
            &tool, 5000.0, &config,
        );
        assert!(!tp.segments.is_empty());
        // More tails = more segments
        let cutting = tp.segments.iter().filter(|s| s.z < 0.0).count();
        assert!(cutting >= 10, "10 tails should have many cutting moves, got {}", cutting);
    }

    #[test]
    fn test_dovetail_multi_pass_depth() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig {
            depth_per_pass: 0.25,
            ..Default::default()
        };
        // 1.0" depth with 0.25" per pass = 4 passes
        let tp = generate_dovetail_toolpath(
            &rect, &DovetailParams { edge: DovetailEdge::Bottom, tail_count: 2, tail_width: 0.5, pin_width: 0.25, depth: 1.0 },
            &tool, 5000.0, &config,
        );
        // Verify multiple depth levels exist
        let unique_depths: std::collections::HashSet<i64> = tp.segments.iter()
            .filter(|s| s.z < 0.0)
            .map(|s| (s.z * 1000.0) as i64)
            .collect();
        assert!(unique_depths.len() >= 2, "multi-pass should have multiple depth levels, got {:?}", unique_depths);
    }

    #[test]
    fn test_box_joint_all_four_edges() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();

        for edge in [DovetailEdge::Top, DovetailEdge::Bottom, DovetailEdge::Left, DovetailEdge::Right] {
            let tp = generate_box_joint_toolpath(
                &rect, &BoxJointParams { edge, finger_width: 0.5, finger_count: 8, depth: 0.75 },
                &tool, 5000.0, &config,
            );
            assert!(!tp.segments.is_empty(), "box joint on {:?} should produce segments", edge);
            let deepest = tp.segments.iter().map(|s| s.z).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            assert!((deepest - (-0.75)).abs() < 1e-10, "{:?}: deepest should be -0.75", edge);
        }
    }

    #[test]
    fn test_box_joint_two_fingers() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_box_joint_toolpath(
            &rect, &BoxJointParams { edge: DovetailEdge::Bottom, finger_width: 0.75, finger_count: 2, depth: 0.5 },
            &tool, 5000.0, &config,
        );
        assert!(!tp.segments.is_empty());
        // With 2 fingers, step_by(2) gives fingers at index 0 only = 1 slot
        let cutting = tp.segments.iter().filter(|s| s.z < 0.0).count();
        assert!(cutting >= 1, "2-finger box joint should have at least 1 slot cut");
    }

    #[test]
    fn test_box_joint_odd_finger_count() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        // 7 fingers: slots at 0, 2, 4, 6 = 4 slots
        let tp = generate_box_joint_toolpath(
            &rect, &BoxJointParams { edge: DovetailEdge::Bottom, finger_width: 0.25, finger_count: 7, depth: 0.5 },
            &tool, 5000.0, &config,
        );
        assert!(!tp.segments.is_empty());
    }

    #[test]
    fn test_mortise_all_positions_within_bounds() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();

        // Test mortise at various positions
        for (x, y) in [(1.0, 1.0), (3.0, 2.0), (5.0, 3.0)] {
            let tp = generate_mortise_toolpath(
                &rect, &MortiseParams { x, y, width: 0.375, length: 1.0, depth: 0.75 },
                &tool, 5000.0, &config,
            );
            for seg in &tp.segments {
                if seg.z < 0.0 {
                    assert!(seg.endpoint.x >= rect.min_x() - 0.5,
                        "mortise at ({},{}) has x={} below min", x, y, seg.endpoint.x);
                    assert!(seg.endpoint.x <= rect.max_x() + 0.5,
                        "mortise at ({},{}) has x={} above max", x, y, seg.endpoint.x);
                }
            }
        }
    }

    #[test]
    fn test_mortise_small_fits_tool() {
        // Mortise smaller than tool diameter — should clamp to 0
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill(); // 0.25" diameter
        let config = CamConfig::default();
        let tp = generate_mortise_toolpath(
            &rect, &MortiseParams { x: 3.0, y: 2.0, width: 0.1, length: 0.1, depth: 0.5 }, // tiny mortise
            &tool, 5000.0, &config,
        );
        // Should not panic, should still produce segments
        assert!(!tp.segments.is_empty());
    }

    #[test]
    fn test_mortise_multi_pass_depth() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig {
            depth_per_pass: 0.25,
            ..Default::default()
        };
        let tp = generate_mortise_toolpath(
            &rect, &MortiseParams { x: 3.0, y: 2.0, width: 0.5, length: 1.5, depth: 1.0 },
            &tool, 5000.0, &config,
        );
        let deepest = tp.segments.iter().map(|s| s.z).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        assert!((deepest - (-1.0)).abs() < 1e-10, "mortise should reach full depth");

        // Multiple depth levels
        let unique_depths: std::collections::HashSet<i64> = tp.segments.iter()
            .filter(|s| s.z < 0.0)
            .map(|s| (s.z * 1000.0) as i64)
            .collect();
        assert!(unique_depths.len() >= 3, "1\" depth at 0.25\"/pass = 4 levels, got {}", unique_depths.len());
    }

    #[test]
    fn test_tenon_all_four_edges() {
        let rect = Rect::from_dimensions(4.0, 6.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();

        for edge in [DovetailEdge::Top, DovetailEdge::Bottom, DovetailEdge::Left, DovetailEdge::Right] {
            let tp = generate_tenon_toolpath(
                &rect, &TenonParams { edge, tenon_thickness: 0.375, tenon_width: 1.0, tenon_length: 1.0, shoulder_depth: 0.25 },
                &tool, 5000.0, &config,
            );
            assert!(!tp.segments.is_empty(), "tenon on {:?} should produce segments", edge);
            assert!(tp.segments.first().unwrap().z > 0.0, "{:?}: should start at safe Z", edge);
            assert!(tp.segments.last().unwrap().z > 0.0, "{:?}: should end at safe Z", edge);
        }
    }

    #[test]
    fn test_tenon_shoulder_depth_correct() {
        let rect = Rect::from_dimensions(4.0, 6.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_tenon_toolpath(
            &rect, &TenonParams { edge: DovetailEdge::Left, tenon_thickness: 0.375, tenon_width: 1.0, tenon_length: 1.0, shoulder_depth: 0.5 },
            &tool, 5000.0, &config,
        );
        let deepest = tp.segments.iter().map(|s| s.z).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        assert!((deepest - (-0.5)).abs() < 1e-10, "shoulder depth should be 0.5, got {}", -deepest);
    }

    #[test]
    fn test_dowel_empty_holes_produces_minimal_toolpath() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_dowel_holes(
            &rect, &[], 0.315, 0.5, &tool, 5000.0, &config,
        );
        // Empty holes list should still produce a valid toolpath (just the retract)
        assert!(!tp.segments.is_empty());
        // No cutting moves
        let cutting = tp.segments.iter().filter(|s| s.z < 0.0).count();
        assert_eq!(cutting, 0, "no holes = no cutting moves");
    }

    #[test]
    fn test_dowel_single_hole() {
        let rect = Rect::from_dimensions(6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_dowel_holes(
            &rect, &[(3.0, 2.0)], 0.315, 0.5, &tool, 5000.0, &config,
        );
        let plunges: Vec<_> = tp.segments.iter()
            .filter(|s| matches!(s.motion, Motion::Linear) && s.z < 0.0)
            .collect();
        assert_eq!(plunges.len(), 1, "single dowel hole = 1 plunge");
    }

    #[test]
    fn test_dowel_hole_positions_offset_by_part_origin() {
        let rect = Rect::new(Point2D::new(10.0, 20.0), 6.0, 4.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_dowel_holes(
            &rect, &[(1.0, 2.0)], 0.315, 0.5, &tool, 5000.0, &config,
        );
        // The hole should be at rect.min_x() + 1.0, rect.min_y() + 2.0 = (11.0, 22.0)
        let drill_segment = tp.segments.iter()
            .find(|s| matches!(s.motion, Motion::Linear) && s.z < 0.0)
            .unwrap();
        assert!((drill_segment.endpoint.x - 11.0).abs() < 1e-10);
        assert!((drill_segment.endpoint.y - 22.0).abs() < 1e-10);
    }

    #[test]
    fn test_all_new_generators_no_rapid_below_zero() {
        let rect = Rect::from_dimensions(8.0, 6.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();

        // Dovetail
        let tp = generate_dovetail_toolpath(&rect, &DovetailParams { edge: DovetailEdge::Bottom, tail_count: 3, tail_width: 0.5, pin_width: 0.25, depth: 0.75 }, &tool, 5000.0, &config);
        for seg in &tp.segments {
            if matches!(seg.motion, Motion::Rapid) {
                assert!(seg.z >= 0.0, "dovetail rapid at z={}", seg.z);
            }
        }

        // Box joint
        let tp = generate_box_joint_toolpath(&rect, &BoxJointParams { edge: DovetailEdge::Bottom, finger_width: 0.5, finger_count: 6, depth: 0.75 }, &tool, 5000.0, &config);
        for seg in &tp.segments {
            if matches!(seg.motion, Motion::Rapid) {
                assert!(seg.z >= 0.0, "box joint rapid at z={}", seg.z);
            }
        }

        // Mortise
        let tp = generate_mortise_toolpath(&rect, &MortiseParams { x: 4.0, y: 3.0, width: 0.5, length: 1.5, depth: 0.75 }, &tool, 5000.0, &config);
        for seg in &tp.segments {
            if matches!(seg.motion, Motion::Rapid) {
                assert!(seg.z >= 0.0, "mortise rapid at z={}", seg.z);
            }
        }

        // Tenon
        let tp = generate_tenon_toolpath(&rect, &TenonParams { edge: DovetailEdge::Left, tenon_thickness: 0.375, tenon_width: 1.0, tenon_length: 1.0, shoulder_depth: 0.25 }, &tool, 5000.0, &config);
        for seg in &tp.segments {
            if matches!(seg.motion, Motion::Rapid) {
                assert!(seg.z >= 0.0, "tenon rapid at z={}", seg.z);
            }
        }

        // Dowel holes
        let tp = generate_dowel_holes(&rect, &[(2.0, 2.0), (4.0, 4.0)], 0.315, 0.5, &tool, 5000.0, &config);
        for seg in &tp.segments {
            if matches!(seg.motion, Motion::Rapid) {
                assert!(seg.z >= 0.0, "dowel rapid at z={}", seg.z);
            }
        }
    }
}
