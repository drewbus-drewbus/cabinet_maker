use cm_cam::toolpath::{Motion, Toolpath};
use cm_core::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::machine::MachineProfile;

/// Result of validating a project against a machine profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Errors that must be resolved before cutting (abort conditions).
    pub errors: Vec<ValidationError>,
    /// Warnings that should be reviewed but won't prevent cutting.
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// A validation error that prevents cutting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    /// A part exceeds the machine's X or Y travel.
    PartExceedsTravel {
        part_label: String,
        part_width: f64,
        part_height: f64,
        travel_x: f64,
        travel_y: f64,
    },
    /// Requested RPM is outside the machine's spindle range.
    RpmOutOfRange {
        requested: f64,
        min: f64,
        max: f64,
    },
    /// Cut depth exceeds the tool's cutting length.
    CutDepthExceedsTool {
        part_label: String,
        cut_depth: f64,
        cutting_length: f64,
        tool_description: String,
    },
    /// G-code coordinates exceed machine travel bounds.
    GcodeBoundsExceeded {
        axis: char,
        value: f64,
        limit: f64,
    },
}

/// A validation warning that should be reviewed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationWarning {
    /// Part is too large for the machine bed and needs manual pre-cutting.
    PartNeedsPreCutting {
        part_label: String,
        part_width: f64,
        part_height: f64,
        travel_x: f64,
        travel_y: f64,
    },
    /// Multiple tools are required but the machine has no ATC.
    MultipleToolsNoAtc {
        tool_count: usize,
    },
    /// Sheet is larger than machine bed — nesting will span multiple setups.
    SheetExceedsBed {
        sheet_width: f64,
        sheet_length: f64,
        travel_x: f64,
        travel_y: f64,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PartExceedsTravel {
                part_label,
                part_width,
                part_height,
                travel_x,
                travel_y,
            } => write!(
                f,
                "Part '{}' ({:.3}\" x {:.3}\") exceeds machine travel ({:.1}\" x {:.1}\")",
                part_label, part_width, part_height, travel_x, travel_y,
            ),
            Self::RpmOutOfRange {
                requested,
                min,
                max,
            } => write!(
                f,
                "Requested RPM {} is outside machine range ({}-{})",
                *requested as u32, *min as u32, *max as u32,
            ),
            Self::CutDepthExceedsTool {
                part_label,
                cut_depth,
                cutting_length,
                tool_description,
            } => write!(
                f,
                "Part '{}' requires {:.3}\" cut depth but {} only has {:.3}\" cutting length",
                part_label, cut_depth, tool_description, cutting_length,
            ),
            Self::GcodeBoundsExceeded { axis, value, limit } => write!(
                f,
                "G-code {} coordinate {:.4} exceeds machine travel limit {:.4}",
                axis, value, limit,
            ),
        }
    }
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PartNeedsPreCutting {
                part_label,
                part_width,
                part_height,
                travel_x,
                travel_y,
            } => write!(
                f,
                "Part '{}' ({:.3}\" x {:.3}\") exceeds bed ({:.1}\" x {:.1}\") — pre-cut to fit",
                part_label, part_width, part_height, travel_x, travel_y,
            ),
            Self::MultipleToolsNoAtc { tool_count } => write!(
                f,
                "{} tools required but machine has no automatic tool changer — manual changes needed",
                tool_count,
            ),
            Self::SheetExceedsBed {
                sheet_width,
                sheet_length,
                travel_x,
                travel_y,
            } => write!(
                f,
                "Sheet ({:.0}\" x {:.0}\") exceeds machine bed ({:.1}\" x {:.1}\")",
                sheet_width, sheet_length, travel_x, travel_y,
            ),
        }
    }
}

/// Information about a part for validation purposes.
pub struct PartInfo {
    pub label: String,
    pub width: f64,
    pub height: f64,
    pub thickness: f64,
    /// Maximum operation depth (deepest dado, rabbet, or drill on this part).
    pub max_operation_depth: f64,
}

/// Validate parts and tools against a machine profile before generating G-code.
///
/// Checks that all parts fit within the machine's travel limits, RPM is within
/// the spindle range, and cut depths are within tool capabilities.
pub fn validate_project(
    parts: &[PartInfo],
    tools: &[Tool],
    rpm: f64,
    machine: &MachineProfile,
    sheet_width: Option<f64>,
    sheet_length: Option<f64>,
) -> ValidationResult {
    let mut result = ValidationResult::new();
    let travel_x = machine.machine.travel_x;
    let travel_y = machine.machine.travel_y;

    // Check RPM range
    if rpm < machine.machine.min_rpm || rpm > machine.machine.max_rpm {
        result.errors.push(ValidationError::RpmOutOfRange {
            requested: rpm,
            min: machine.machine.min_rpm,
            max: machine.machine.max_rpm,
        });
    }

    // Check each part against machine travel and tool cutting length
    for part in parts {
        // Part must fit within machine travel (with some margin for tool offset)
        let fits_normal = part.width <= travel_x && part.height <= travel_y;
        let fits_rotated = part.height <= travel_x && part.width <= travel_y;

        if !fits_normal && !fits_rotated {
            // Part can't fit on the machine at all — this is a warning, not an error,
            // because the user might pre-cut or use a different machine.
            result
                .warnings
                .push(ValidationWarning::PartNeedsPreCutting {
                    part_label: part.label.clone(),
                    part_width: part.width,
                    part_height: part.height,
                    travel_x,
                    travel_y,
                });
        }

        // Check profile cut depth against tool cutting length
        for tool in tools {
            if part.thickness > tool.cutting_length {
                result
                    .errors
                    .push(ValidationError::CutDepthExceedsTool {
                        part_label: part.label.clone(),
                        cut_depth: part.thickness,
                        cutting_length: tool.cutting_length,
                        tool_description: tool.description.clone(),
                    });
            }

            // Check operation depths
            if part.max_operation_depth > tool.cutting_length {
                result
                    .errors
                    .push(ValidationError::CutDepthExceedsTool {
                        part_label: part.label.clone(),
                        cut_depth: part.max_operation_depth,
                        cutting_length: tool.cutting_length,
                        tool_description: tool.description.clone(),
                    });
            }
        }
    }

    // Check if multiple tools are needed without ATC
    if tools.len() > 1 && !machine.machine.has_atc {
        result
            .warnings
            .push(ValidationWarning::MultipleToolsNoAtc {
                tool_count: tools.len(),
            });
    }

    // Check if sheet exceeds machine bed
    if let (Some(sw), Some(sl)) = (sheet_width, sheet_length)
        && (sw > travel_x || sl > travel_y)
    {
        result.warnings.push(ValidationWarning::SheetExceedsBed {
            sheet_width: sw,
            sheet_length: sl,
            travel_x,
            travel_y,
        });
    }

    result
}

/// Validate generated G-code toolpaths against machine travel limits.
///
/// Scans all toolpath coordinates to ensure nothing exceeds the machine's
/// physical travel limits. This is a post-generation safety check.
pub fn validate_toolpaths(
    toolpaths: &[Toolpath],
    machine: &MachineProfile,
) -> ValidationResult {
    let mut result = ValidationResult::new();
    let travel_x = machine.machine.travel_x;
    let travel_y = machine.machine.travel_y;
    let travel_z = machine.machine.travel_z;

    let mut max_x: f64 = 0.0;
    let mut max_y: f64 = 0.0;
    let mut max_z: f64 = 0.0;
    let mut min_z: f64 = 0.0;

    for tp in toolpaths {
        for seg in &tp.segments {
            max_x = max_x.max(seg.endpoint.x);
            max_y = max_y.max(seg.endpoint.y);
            max_z = max_z.max(seg.z);
            min_z = min_z.min(seg.z);

            // Check for rapid moves below Z=0 (safety critical)
            if matches!(seg.motion, Motion::Rapid) && seg.z < 0.0 {
                result
                    .errors
                    .push(ValidationError::GcodeBoundsExceeded {
                        axis: 'Z',
                        value: seg.z,
                        limit: 0.0,
                    });
            }
        }
    }

    if max_x > travel_x {
        result
            .errors
            .push(ValidationError::GcodeBoundsExceeded {
                axis: 'X',
                value: max_x,
                limit: travel_x,
            });
    }

    if max_y > travel_y {
        result
            .errors
            .push(ValidationError::GcodeBoundsExceeded {
                axis: 'Y',
                value: max_y,
                limit: travel_y,
            });
    }

    // Z travel check: total Z range must fit within machine travel
    let z_range = max_z - min_z;
    if z_range > travel_z {
        result
            .errors
            .push(ValidationError::GcodeBoundsExceeded {
                axis: 'Z',
                value: z_range,
                limit: travel_z,
            });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use cm_cam::ops::{generate_profile_cut, CamConfig};
    use cm_core::geometry::Rect;

    fn test_machine() -> MachineProfile {
        MachineProfile::tormach_pcnc1100()
    }

    #[test]
    fn test_valid_project_passes() {
        let parts = vec![PartInfo {
            label: "side".into(),
            width: 8.0,
            height: 9.0,
            thickness: 0.75,
            max_operation_depth: 0.375,
        }];
        let tools = vec![Tool::quarter_inch_endmill()];
        let machine = test_machine();
        let result = validate_project(&parts, &tools, 5000.0, &machine, Some(48.0), Some(96.0));
        assert!(result.is_ok(), "valid project should pass: {:?}", result.errors);
    }

    #[test]
    fn test_rpm_out_of_range_error() {
        let parts = vec![];
        let tools = vec![Tool::quarter_inch_endmill()];
        let machine = test_machine();

        // Too high
        let result = validate_project(&parts, &tools, 10000.0, &machine, None, None);
        assert!(result.has_errors());
        assert!(matches!(
            result.errors[0],
            ValidationError::RpmOutOfRange { .. }
        ));

        // Too low
        let result = validate_project(&parts, &tools, 50.0, &machine, None, None);
        assert!(result.has_errors());
    }

    #[test]
    fn test_cut_depth_exceeds_tool() {
        let parts = vec![PartInfo {
            label: "thick_panel".into(),
            width: 8.0,
            height: 8.0,
            thickness: 2.0, // 2" thick — exceeds 1" cutting length
            max_operation_depth: 0.375,
        }];
        let tools = vec![Tool::quarter_inch_endmill()]; // 1" cutting length
        let machine = test_machine();
        let result = validate_project(&parts, &tools, 5000.0, &machine, None, None);
        assert!(result.has_errors());
        assert!(matches!(
            result.errors[0],
            ValidationError::CutDepthExceedsTool { .. }
        ));
    }

    #[test]
    fn test_large_part_warns_pre_cutting() {
        let parts = vec![PartInfo {
            label: "full_side".into(),
            width: 24.0, // exceeds 18" X travel and 9.5" Y travel
            height: 30.0,
            thickness: 0.75,
            max_operation_depth: 0.375,
        }];
        let tools = vec![Tool::quarter_inch_endmill()];
        let machine = test_machine();
        let result = validate_project(&parts, &tools, 5000.0, &machine, None, None);
        assert!(result.has_warnings());
        assert!(matches!(
            result.warnings[0],
            ValidationWarning::PartNeedsPreCutting { .. }
        ));
    }

    #[test]
    fn test_multiple_tools_no_atc_warns() {
        let parts = vec![];
        let tools = vec![
            Tool::quarter_inch_endmill(),
            Tool {
                number: 2,
                tool_type: cm_core::tool::ToolType::Drill,
                diameter: 0.197,
                flutes: 1,
                cutting_length: 1.0,
                description: "5mm drill".into(),
            },
        ];
        let machine = test_machine(); // has_atc = false
        let result = validate_project(&parts, &tools, 5000.0, &machine, None, None);
        assert!(result.has_warnings());
        assert!(matches!(
            result.warnings[0],
            ValidationWarning::MultipleToolsNoAtc { .. }
        ));
    }

    #[test]
    fn test_sheet_exceeds_bed_warns() {
        let parts = vec![];
        let tools = vec![Tool::quarter_inch_endmill()];
        let machine = test_machine();
        let result =
            validate_project(&parts, &tools, 5000.0, &machine, Some(48.0), Some(96.0));
        assert!(result.has_warnings());
        assert!(matches!(
            result.warnings.iter().find(|w| matches!(w, ValidationWarning::SheetExceedsBed { .. })),
            Some(_)
        ));
    }

    #[test]
    fn test_toolpath_bounds_valid() {
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        let rect = Rect::from_dimensions(5.0, 3.0);
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);
        let machine = test_machine();

        let result = validate_toolpaths(&[tp], &machine);
        assert!(result.is_ok(), "small part should pass bounds check: {:?}", result.errors);
    }

    #[test]
    fn test_toolpath_bounds_exceeded() {
        let tool = Tool::quarter_inch_endmill();
        let config = CamConfig::default();
        // Part positioned beyond machine travel
        let rect = Rect::new(
            cm_core::geometry::Point2D::new(15.0, 7.0),
            10.0,
            5.0,
        );
        let tp = generate_profile_cut(&rect, 0.75, &tool, 5000.0, &config);
        let machine = test_machine(); // 18" x 9.5" travel

        let result = validate_toolpaths(&[tp], &machine);
        assert!(result.has_errors(), "out-of-bounds part should fail");
    }
}
