use cm_core::geometry::{Point2D, Rect};
use cm_core::tool::Tool;

use crate::toolpath::{Motion, Toolpath, ToolpathSegment};

/// Configuration for toolpath generation.
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
        }
    }
}

/// Generate a profile (outline) cut for a rectangular part.
///
/// The tool cuts around the outside of the rectangle, offsetting by the tool
/// radius. Material depth is cut in multiple passes.
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

    let mut segments = Vec::new();

    // Calculate depth passes
    let total_depth = material_thickness;
    let num_passes = (total_depth / config.depth_per_pass).ceil() as u32;

    // Rapid to start position (above bottom-left corner of cut path)
    let start = Point2D::new(cut_rect.min_x(), cut_rect.min_y());
    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: start,
        z: config.safe_z,
    });

    // Depth passes
    for pass in 1..=num_passes {
        let z = -(config.depth_per_pass * pass as f64).min(total_depth);

        // Rapid to rapid_z
        segments.push(ToolpathSegment {
            motion: Motion::Rapid,
            endpoint: start,
            z: config.rapid_z,
        });

        // Plunge to cutting depth
        segments.push(ToolpathSegment {
            motion: Motion::Linear,
            endpoint: start,
            z,
        });

        // Cut rectangle clockwise: bottom-left → bottom-right → top-right → top-left → close
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

    // Retract to safe Z
    segments.push(ToolpathSegment {
        motion: Motion::Rapid,
        endpoint: start,
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
    dado_position: f64,
    dado_width: f64,
    dado_depth: f64,
    horizontal: bool,
    tool: &Tool,
    rpm: f64,
    config: &CamConfig,
) -> Toolpath {
    let feed_rate = tool.recommended_feed_rate(rpm);
    let plunge_rate = feed_rate * 0.5;
    let r = tool.radius();

    let mut segments = Vec::new();
    let num_passes = (dado_depth / config.depth_per_pass).ceil() as u32;

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

/// Generate a simple drill operation at a single point.
pub fn generate_drill(
    position: Point2D,
    depth: f64,
    tool: &Tool,
    rpm: f64,
    config: &CamConfig,
) -> Toolpath {
    let feed_rate = tool.recommended_feed_rate(rpm);
    let plunge_rate = feed_rate * 0.3; // slower plunge for drilling

    let segments = vec![
        // Rapid to position
        ToolpathSegment {
            motion: Motion::Rapid,
            endpoint: position,
            z: config.safe_z,
        },
        // Rapid down to near surface
        ToolpathSegment {
            motion: Motion::Rapid,
            endpoint: position,
            z: config.rapid_z,
        },
        // Drill into material
        ToolpathSegment {
            motion: Motion::Linear,
            endpoint: position,
            z: -depth,
        },
        // Retract
        ToolpathSegment {
            motion: Motion::Rapid,
            endpoint: position,
            z: config.safe_z,
        },
    ];

    Toolpath {
        tool_number: tool.number,
        rpm,
        feed_rate,
        plunge_rate,
        segments,
    }
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

        // 0.75 / 0.25 = 3 passes. Each pass has: rapid_z + plunge + 4 sides = 6 segments.
        // Plus initial rapid + final retract = 2.
        // Total: 2 + 3*6 = 20
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
    fn test_dado_toolpath_generation() {
        let rect = Rect::from_dimensions(12.0, 30.0);
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let tp = generate_dado_toolpath(&rect, 10.0, 0.75, 0.375, true, &tool, 5000.0, &config);

        // Should have segments
        assert!(!tp.segments.is_empty());

        // Verify all cuts are at or above dado depth
        for seg in &tp.segments {
            assert!(seg.z >= -0.375 - 1e-10, "cut too deep: {}", seg.z);
        }
    }
}
