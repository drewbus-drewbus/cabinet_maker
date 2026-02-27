use std::collections::HashMap;

use cm_cabinet::part::PartOperation;
use cm_cabinet::project::{Project, TaggedPart};
use cm_cam::ops::{
    generate_box_joint_toolpath, generate_dado_toolpath, generate_dovetail_toolpath,
    generate_dowel_holes, generate_drill, generate_mortise_toolpath, generate_profile_cut,
    generate_rabbet_toolpath, generate_tenon_toolpath, BoxJointParams, CamConfig, DadoParams,
    DovetailEdge, DovetailParams, MortiseParams, RabbetEdge, TenonParams,
};
use cm_cam::{apply_corner_fillets, arc_fit, optimize_rapid_order, FilletStyle, Toolpath};
use cm_core::geometry::{Point2D, Rect};
use cm_core::tool::Tool;
use cm_nesting::packer::{nest_parts, NestingConfig, NestingPart, NestingResult};
use cm_post::gcode::GCodeEmitter;
use cm_post::machine::MachineProfile;
use cm_post::validate::{self, PartInfo};

use crate::error::PipelineError;

/// Configuration for the generation pipeline.
#[derive(Debug, Clone)]
pub struct GenerateConfig {
    pub skip_validation: bool,
    pub enable_hardware: bool,
    pub rpm: Option<f64>,
}

impl Default for GenerateConfig {
    fn default() -> Self {
        Self {
            skip_validation: false,
            enable_hardware: true,
            rpm: None,
        }
    }
}

/// Per-material group output from the pipeline.
#[derive(Debug)]
pub struct MaterialGroupOutput {
    pub material_name: String,
    pub nesting_result: NestingResult,
    pub nesting_config: NestingConfig,
    pub sheet_gcodes: Vec<String>,
}

/// Output from the full generation pipeline.
#[derive(Debug)]
pub struct GenerateResult {
    pub tagged_parts: Vec<TaggedPart>,
    pub material_groups: Vec<MaterialGroupOutput>,
    pub total_sheets: u32,
    pub tool: Tool,
    pub rpm: f64,
}

/// Progress reporting trait for pipeline events.
pub trait ProgressReporter: Send + Sync {
    fn report(&self, event: ProgressEvent);
}

/// Events emitted during pipeline execution.
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    PartsGenerated { count: usize },
    HardwareOpsAdded { count: usize },
    NestingComplete { material: String, sheets: usize, utilization: f64 },
    GcodeEmitted { material: String, sheet: usize },
    Complete,
}

/// No-op reporter for when progress isn't needed.
pub struct NullReporter;
impl ProgressReporter for NullReporter {
    fn report(&self, _event: ProgressEvent) {}
}

/// Run the full generation pipeline.
pub fn generate_pipeline(
    project: &Project,
    machine: &MachineProfile,
    config: &GenerateConfig,
    reporter: &dyn ProgressReporter,
) -> Result<GenerateResult, PipelineError> {
    let all_cabs = project.all_cabinets();

    // Validate cabinets
    if !config.skip_validation {
        for cab in &all_cabs {
            let issues = cm_cabinet::cabinet::validate_cabinet(cab);
            let has_error = issues.iter().any(|i| {
                i.severity == cm_cabinet::cabinet::ValidationSeverity::Error
            });
            if has_error {
                let msgs: Vec<String> = issues
                    .iter()
                    .filter(|i| i.severity == cm_cabinet::cabinet::ValidationSeverity::Error)
                    .map(|i| format!("{}: {}", cab.name, i.message))
                    .collect();
                return Err(PipelineError::CabinetValidation(msgs.join("; ")));
            }
        }
    }

    // Generate all parts
    let mut tagged_parts = project.generate_all_parts();
    reporter.report(ProgressEvent::PartsGenerated { count: tagged_parts.len() });

    // Apply hardware boring patterns
    if config.enable_hardware {
        let hw_count = apply_hardware_boring(&mut tagged_parts, &all_cabs);
        if hw_count > 0 {
            reporter.report(ProgressEvent::HardwareOpsAdded { count: hw_count });
        }
    }

    // Determine tool and RPM
    let tool = project
        .tools
        .first()
        .cloned()
        .unwrap_or_else(Tool::quarter_inch_endmill);
    let rpm = config.rpm.unwrap_or(machine.machine.max_rpm * 0.9);

    // Validation
    if !config.skip_validation {
        validate_parts(&tagged_parts, project, rpm, machine)?;
    }

    // Group by material and process each group
    let groups = Project::group_parts_by_material(&tagged_parts);
    let cam_config = CamConfig {
        safe_z: machine.post.safe_z,
        rapid_z: machine.post.rapid_z,
        ..Default::default()
    };
    let emitter = GCodeEmitter::new(machine, project.project.units);

    let mut material_groups = Vec::new();
    let mut total_sheets: u32 = 0;

    for group in &groups {
        let mat = &group.material;

        let nesting_config = NestingConfig {
            sheet_width: mat.sheet_width.unwrap_or(48.0),
            sheet_length: mat.sheet_length.unwrap_or(96.0),
            kerf: 0.25,
            edge_margin: 0.5,
            allow_rotation: false,
            guillotine_compatible: false,
        };

        // Build nesting parts
        let nesting_parts = build_nesting_parts(&group.parts, all_cabs.len() > 1);
        let nesting_result = nest_parts(&nesting_parts, &nesting_config);
        total_sheets += nesting_result.sheet_count as u32;

        reporter.report(ProgressEvent::NestingComplete {
            material: group.material_name.clone(),
            sheets: nesting_result.sheet_count,
            utilization: nesting_result.overall_utilization,
        });

        // Generate G-code per sheet
        let mut sheet_gcodes = Vec::new();
        for sheet in &nesting_result.sheets {
            let sheet_toolpaths = generate_sheet_toolpaths(
                sheet, &group.parts, &tool, rpm, &cam_config,
            );

            // Post-process toolpaths
            let mut toolpaths = post_process_toolpaths(sheet_toolpaths, tool.diameter / 2.0);

            // Validate toolpaths
            if !config.skip_validation {
                let tp_validation = validate::validate_toolpaths(&toolpaths, machine);
                if tp_validation.has_errors() {
                    let msgs: Vec<String> = tp_validation.errors.iter()
                        .map(|e| e.to_string())
                        .collect();
                    return Err(PipelineError::ToolpathValidation {
                        sheet: sheet.sheet_index + 1,
                        message: msgs.join("; "),
                    });
                }
            }

            // Optimize rapid ordering
            optimize_rapid_order(&mut toolpaths);

            let gcode = emitter.emit(&toolpaths);
            sheet_gcodes.push(gcode);

            reporter.report(ProgressEvent::GcodeEmitted {
                material: group.material_name.clone(),
                sheet: sheet.sheet_index + 1,
            });
        }

        material_groups.push(MaterialGroupOutput {
            material_name: group.material_name.clone(),
            nesting_result,
            nesting_config,
            sheet_gcodes,
        });
    }

    reporter.report(ProgressEvent::Complete);

    Ok(GenerateResult {
        tagged_parts,
        material_groups,
        total_sheets,
        tool,
        rpm,
    })
}

/// Apply hardware boring patterns to tagged parts.
fn apply_hardware_boring(
    tagged_parts: &mut [TaggedPart],
    cabinets: &[&cm_cabinet::cabinet::Cabinet],
) -> usize {
    let mut hw_op_count = 0;
    let part_index: HashMap<(String, String), usize> = tagged_parts
        .iter()
        .enumerate()
        .map(|(i, tp)| ((tp.cabinet_name.clone(), tp.part.label.clone()), i))
        .collect();

    for entry in cabinets {
        let hw_ops = cm_hardware::generate_all_hardware_ops(entry);
        for hw_op in hw_ops {
            if let Some(&idx) = part_index.get(&(entry.name.clone(), hw_op.target_part.clone())) {
                tagged_parts[idx].part.operations.push(hw_op.operation);
                hw_op_count += 1;
            }
        }
    }
    hw_op_count
}

/// Validate parts against machine capabilities.
fn validate_parts(
    tagged_parts: &[TaggedPart],
    project: &Project,
    rpm: f64,
    machine: &MachineProfile,
) -> Result<(), PipelineError> {
    let part_infos: Vec<PartInfo> = tagged_parts.iter().map(|tp| {
        let max_op_depth = tp.part.operations.iter().map(|op| match op {
            PartOperation::Dado(d) => d.depth,
            PartOperation::Rabbet(r) => r.depth,
            PartOperation::Drill(d) => d.depth,
            PartOperation::PocketHole(_) => 0.0,
            PartOperation::Dovetail(d) => d.depth,
            PartOperation::BoxJoint(b) => b.depth,
            PartOperation::Mortise(m) => m.depth,
            PartOperation::Tenon(t) => t.shoulder_depth,
            PartOperation::Dowel(d) => d.depth,
        }).fold(0.0f64, f64::max);
        PartInfo {
            label: format!("{}/{}", tp.cabinet_name, tp.part.label),
            width: tp.part.rect.width,
            height: tp.part.rect.height,
            thickness: tp.part.thickness,
            max_operation_depth: max_op_depth,
        }
    }).collect();

    let primary_mat = project.primary_material();
    let validation = validate::validate_project(
        &part_infos,
        &project.tools,
        rpm,
        machine,
        primary_mat.and_then(|m| m.sheet_width),
        primary_mat.and_then(|m| m.sheet_length),
    );

    if validation.has_errors() {
        let msgs: Vec<String> = validation.errors.iter()
            .map(|e| e.to_string())
            .collect();
        return Err(PipelineError::ProjectValidation(msgs.join("; ")));
    }

    Ok(())
}

/// Build nesting parts from tagged parts.
fn build_nesting_parts(parts: &[&TaggedPart], multi_cabinet: bool) -> Vec<NestingPart> {
    let mut nesting_parts = Vec::new();
    for tp in parts {
        for i in 0..tp.part.quantity {
            let prefix = if multi_cabinet {
                format!("{}/", tp.cabinet_name)
            } else {
                String::new()
            };
            let id = if tp.part.quantity > 1 {
                format!("{}{}_{}", prefix, tp.part.label, i + 1)
            } else {
                format!("{}{}", prefix, tp.part.label)
            };
            nesting_parts.push(NestingPart {
                id,
                width: tp.part.rect.width,
                height: tp.part.rect.height,
                can_rotate: false,
            });
        }
    }
    nesting_parts
}

/// Generate toolpaths for all parts on a single sheet.
pub fn generate_sheet_toolpaths(
    sheet: &cm_nesting::packer::SheetLayout,
    parts: &[&TaggedPart],
    tool: &Tool,
    rpm: f64,
    cam_config: &CamConfig,
) -> Vec<Toolpath> {
    let mut sheet_toolpaths: Vec<Toolpath> = Vec::new();

    for placed in &sheet.parts {
        let base_label = strip_nesting_id(&placed.id);

        let tp_match = parts.iter()
            .find(|tp| tp.part.label == base_label || placed.id.ends_with(&tp.part.label))
            .unwrap_or_else(|| panic!("Part definition not found for '{}'", placed.id));

        let positioned_rect = Rect::new(
            placed.rect.origin,
            placed.rect.width,
            placed.rect.height,
        );

        for op in &tp_match.part.operations {
            if let Some(tp) = generate_operation_toolpath(op, &positioned_rect, tool, rpm, cam_config) {
                sheet_toolpaths.push(tp);
            }
        }

        // Profile cut
        let tp = generate_profile_cut(&positioned_rect, tp_match.part.thickness, tool, rpm, cam_config);
        sheet_toolpaths.push(tp);
    }

    sheet_toolpaths
}

/// Generate toolpath for a single part operation.
pub fn generate_operation_toolpath(
    op: &PartOperation,
    positioned_rect: &Rect,
    tool: &Tool,
    rpm: f64,
    cam_config: &CamConfig,
) -> Option<Toolpath> {
    match op {
        PartOperation::Dado(dado) => {
            let horizontal = dado.orientation == cm_cabinet::part::DadoOrientation::Horizontal;
            Some(generate_dado_toolpath(
                positioned_rect,
                &DadoParams { position: dado.position, width: dado.width, depth: dado.depth, horizontal },
                tool, rpm, cam_config,
            ))
        }
        PartOperation::Rabbet(rabbet) => {
            let edge = match rabbet.edge {
                cm_cabinet::part::Edge::Top => RabbetEdge::Top,
                cm_cabinet::part::Edge::Bottom => RabbetEdge::Bottom,
                cm_cabinet::part::Edge::Left => RabbetEdge::Left,
                cm_cabinet::part::Edge::Right => RabbetEdge::Right,
            };
            Some(generate_rabbet_toolpath(
                positioned_rect, edge, rabbet.width, rabbet.depth,
                tool, rpm, cam_config,
            ))
        }
        PartOperation::Drill(drill) => {
            Some(generate_drill(
                Point2D::new(
                    positioned_rect.min_x() + drill.x,
                    positioned_rect.min_y() + drill.y,
                ),
                drill.depth, tool, rpm, cam_config,
            ))
        }
        PartOperation::PocketHole(ph) => {
            if ph.cnc_operation {
                Some(generate_drill(
                    Point2D::new(
                        positioned_rect.min_x() + ph.x,
                        positioned_rect.min_y() + ph.y,
                    ),
                    0.625, tool, rpm, cam_config,
                ))
            } else {
                None
            }
        }
        PartOperation::Dovetail(dt) => {
            let edge = edge_to_dovetail(dt.edge);
            Some(generate_dovetail_toolpath(
                positioned_rect,
                &DovetailParams { edge, tail_count: dt.tail_count, tail_width: dt.tail_width, pin_width: dt.pin_width, depth: dt.depth },
                tool, rpm, cam_config,
            ))
        }
        PartOperation::BoxJoint(bj) => {
            let edge = edge_to_dovetail(bj.edge);
            Some(generate_box_joint_toolpath(
                positioned_rect,
                &BoxJointParams { edge, finger_width: bj.finger_width, finger_count: bj.finger_count, depth: bj.depth },
                tool, rpm, cam_config,
            ))
        }
        PartOperation::Mortise(m) => {
            Some(generate_mortise_toolpath(
                positioned_rect,
                &MortiseParams { x: m.x, y: m.y, width: m.width, length: m.length, depth: m.depth },
                tool, rpm, cam_config,
            ))
        }
        PartOperation::Tenon(t) => {
            let edge = edge_to_dovetail(t.edge);
            Some(generate_tenon_toolpath(
                positioned_rect,
                &TenonParams { edge, tenon_thickness: t.thickness, tenon_width: t.width, tenon_length: t.length, shoulder_depth: t.shoulder_depth },
                tool, rpm, cam_config,
            ))
        }
        PartOperation::Dowel(d) => {
            let hole_positions: Vec<(f64, f64)> = d.holes
                .iter()
                .map(|h| (h.x, h.y))
                .collect();
            Some(generate_dowel_holes(
                positioned_rect, &hole_positions,
                d.dowel_diameter, d.depth,
                tool, rpm, cam_config,
            ))
        }
    }
}

/// Convert cabinet Edge to CAM DovetailEdge.
fn edge_to_dovetail(edge: cm_cabinet::part::Edge) -> DovetailEdge {
    match edge {
        cm_cabinet::part::Edge::Top => DovetailEdge::Top,
        cm_cabinet::part::Edge::Bottom => DovetailEdge::Bottom,
        cm_cabinet::part::Edge::Left => DovetailEdge::Left,
        cm_cabinet::part::Edge::Right => DovetailEdge::Right,
    }
}

/// Apply post-processing to toolpaths (fillets, arc fitting).
fn post_process_toolpaths(mut toolpaths: Vec<Toolpath>, tool_radius: f64) -> Vec<Toolpath> {
    for tp in &mut toolpaths {
        apply_corner_fillets(tp, tool_radius, FilletStyle::DogBone);
    }
    for tp in &mut toolpaths {
        arc_fit(tp, 0.001);
    }
    toolpaths
}

/// Strip the cabinet prefix and/or quantity suffix from a nesting part ID.
pub fn strip_nesting_id(id: &str) -> String {
    let label = if let Some(slash) = id.find('/') {
        &id[slash + 1..]
    } else {
        id
    };

    if let Some(last_underscore) = label.rfind('_') {
        let suffix = &label[last_underscore + 1..];
        if suffix.parse::<u32>().is_ok() {
            return label[..last_underscore].to_string();
        }
    }
    label.to_string()
}
