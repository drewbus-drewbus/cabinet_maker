use std::fs;
use std::path::PathBuf;

use tauri::State;

use cm_cam::ops::{generate_profile_cut, CamConfig};
use cm_nesting::packer::{nest_parts, NestingConfig, NestingPart};
use cm_post::validate::{validate_project, PartInfo, ValidationResult};
use cm_post::GCodeEmitter;
use cm_cabinet::project::Project;

use crate::state::AppState;

/// Validate the current project against the machine profile.
#[tauri::command]
pub fn validate_project_cmd(state: State<'_, AppState>) -> Result<ValidationResult, String> {
    let guard = state.project.lock().map_err(|e| e.to_string())?;
    let project = guard.as_ref().ok_or("No project loaded")?;
    let machine = state.machine.lock().map_err(|e| e.to_string())?;

    let tagged = project.generate_all_parts();
    let parts: Vec<PartInfo> = tagged
        .iter()
        .map(|tp| {
            let max_op_depth = tp.part.operations.iter().fold(0.0_f64, |max, op| {
                match op {
                    cm_cabinet::part::PartOperation::Dado(d) => max.max(d.depth),
                    cm_cabinet::part::PartOperation::Rabbet(r) => max.max(r.depth),
                    cm_cabinet::part::PartOperation::Drill(d) => max.max(d.depth),
                    cm_cabinet::part::PartOperation::PocketHole(_) => max,
                }
            });
            PartInfo {
                label: format!("{}:{}", tp.cabinet_name, tp.part.label),
                width: tp.part.rect.width,
                height: tp.part.rect.height,
                thickness: tp.material.thickness,
                max_operation_depth: max_op_depth,
            }
        })
        .collect();

    let tools = &project.tools;
    let rpm = 5000.0; // Default RPM — could come from project settings
    let primary_mat = project.primary_material();
    let sw = primary_mat.and_then(|m| m.sheet_width);
    let sl = primary_mat.and_then(|m| m.sheet_length);

    Ok(validate_project(&parts, tools, rpm, &machine, sw, sl))
}

/// Generate G-code for all nested sheets and write to output directory.
#[tauri::command]
pub fn generate_gcode(
    output_dir: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let guard = state.project.lock().map_err(|e| e.to_string())?;
    let project = guard.as_ref().ok_or("No project loaded")?;
    let machine = state.machine.lock().map_err(|e| e.to_string())?;

    let tagged = project.generate_all_parts();
    let groups = Project::group_parts_by_material(&tagged);

    let units = project.project.units;
    let emitter = GCodeEmitter::new(&machine, units);
    let cam_config = CamConfig::default();
    let rpm = 5000.0;

    let out_path = PathBuf::from(&output_dir);
    fs::create_dir_all(&out_path)
        .map_err(|e| format!("Failed to create output directory: {e}"))?;

    let mut written_files = Vec::new();

    for group in &groups {
        let nesting_config = NestingConfig {
            sheet_width: group.material.sheet_width.unwrap_or(48.0),
            sheet_length: group.material.sheet_length.unwrap_or(96.0),
            ..NestingConfig::default()
        };

        let nesting_parts: Vec<NestingPart> = group
            .parts
            .iter()
            .flat_map(|tp| {
                (0..tp.part.quantity).map(move |_| NestingPart {
                    id: format!("{}:{}", tp.cabinet_name, tp.part.label),
                    width: tp.part.rect.width,
                    height: tp.part.rect.height,
                    can_rotate: false,
                })
            })
            .collect();

        let nesting_result = nest_parts(&nesting_parts, &nesting_config);

        for sheet in &nesting_result.sheets {
            let tool = project.tools.first()
                .ok_or("No tools defined in project")?;

            let toolpaths: Vec<_> = sheet
                .parts
                .iter()
                .map(|placed| {
                    generate_profile_cut(
                        &placed.rect,
                        group.material.thickness,
                        tool,
                        rpm,
                        &cam_config,
                    )
                })
                .collect();

            let gcode = emitter.emit(&toolpaths);

            let safe_name = group.material_name.replace(['/', '\\', '"', ' '], "_");
            let filename = format!(
                "{}_sheet_{}.ngc",
                safe_name,
                sheet.sheet_index + 1,
            );
            let file_path = out_path.join(&filename);
            fs::write(&file_path, &gcode)
                .map_err(|e| format!("Failed to write {filename}: {e}"))?;

            written_files.push(filename);
        }
    }

    Ok(written_files)
}

/// Preview G-code for a specific sheet (returns the G-code string).
#[tauri::command]
pub fn preview_gcode(
    material_index: usize,
    sheet_index: usize,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let guard = state.project.lock().map_err(|e| e.to_string())?;
    let project = guard.as_ref().ok_or("No project loaded")?;
    let machine = state.machine.lock().map_err(|e| e.to_string())?;

    let tagged = project.generate_all_parts();
    let groups = Project::group_parts_by_material(&tagged);

    let group = groups.get(material_index)
        .ok_or(format!("Material index {material_index} out of range"))?;

    let nesting_config = NestingConfig {
        sheet_width: group.material.sheet_width.unwrap_or(48.0),
        sheet_length: group.material.sheet_length.unwrap_or(96.0),
        ..NestingConfig::default()
    };

    let nesting_parts: Vec<NestingPart> = group
        .parts
        .iter()
        .flat_map(|tp| {
            (0..tp.part.quantity).map(move |_| NestingPart {
                id: format!("{}:{}", tp.cabinet_name, tp.part.label),
                width: tp.part.rect.width,
                height: tp.part.rect.height,
                can_rotate: false,
            })
        })
        .collect();

    let nesting_result = nest_parts(&nesting_parts, &nesting_config);

    let sheet = nesting_result.sheets.get(sheet_index)
        .ok_or(format!("Sheet index {sheet_index} out of range"))?;

    let units = project.project.units;
    let emitter = GCodeEmitter::new(&machine, units);
    let cam_config = CamConfig::default();
    let rpm = 5000.0;

    let tool = project.tools.first()
        .ok_or("No tools defined in project")?;

    let toolpaths: Vec<_> = sheet
        .parts
        .iter()
        .map(|placed| {
            generate_profile_cut(
                &placed.rect,
                group.material.thickness,
                tool,
                rpm,
                &cam_config,
            )
        })
        .collect();

    Ok(emitter.emit(&toolpaths))
}
