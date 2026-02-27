use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use cm_cabinet::project::Project;
use cm_nesting::packer::{nest_parts, NestingConfig, NestingPart};
use cm_cam::ops::{generate_profile_cut, CamConfig};
use cm_pipeline::visualize::{ToolpathVisualizationDto, generate_annotated_toolpaths};
use cm_post::gcode::GCodeEmitter;
use cm_post::validate::{self, PartInfo, ValidationResult};

use crate::error::{ServerError, lock_err};
use crate::state::AppState;

fn get_session(state: &AppState, id: &Uuid) -> Result<Arc<crate::state::SessionState>, ServerError> {
    state.get_session(id).ok_or(ServerError::SessionNotFound)
}

/// POST /api/sessions/:id/gcode/validate — Validate the project.
pub async fn validate_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ValidationResult>, ServerError> {
    let session = get_session(&state, &id)?;
    let guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_ref().ok_or(ServerError::NoProject)?;
    let machine = session.machine.lock().map_err(lock_err)?;

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
                    cm_cabinet::part::PartOperation::Dovetail(d) => max.max(d.depth),
                    cm_cabinet::part::PartOperation::BoxJoint(b) => max.max(b.depth),
                    cm_cabinet::part::PartOperation::Mortise(m) => max.max(m.depth),
                    cm_cabinet::part::PartOperation::Tenon(t) => max.max(t.shoulder_depth),
                    cm_cabinet::part::PartOperation::Dowel(d) => max.max(d.depth),
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

    let rpm = 5000.0;
    let primary_mat = project.primary_material();
    let sw = primary_mat.and_then(|m| m.sheet_width);
    let sl = primary_mat.and_then(|m| m.sheet_length);

    Ok(Json(validate::validate_project(&parts, &project.tools, rpm, &machine, sw, sl)))
}

#[derive(Deserialize)]
pub struct GenerateGcodeRequest {
    pub rpm: Option<f64>,
}

/// POST /api/sessions/:id/gcode/generate — Generate G-code for all sheets.
pub async fn generate_gcode(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<Option<GenerateGcodeRequest>>,
) -> Result<Json<Vec<SheetGcode>>, ServerError> {
    let session = get_session(&state, &id)?;
    let guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_ref().ok_or(ServerError::NoProject)?;
    let machine = session.machine.lock().map_err(lock_err)?;

    let tagged = project.generate_all_parts();
    let groups = Project::group_parts_by_material(&tagged);

    let units = project.project.units;
    let emitter = GCodeEmitter::new(&machine, units);
    let cam_config = CamConfig::default();
    let rpm = req.as_ref().and_then(|r| r.rpm).unwrap_or(5000.0);

    let mut result = Vec::new();

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
                .ok_or(ServerError::BadRequest("No tools defined in project".into()))?;

            let toolpaths: Vec<_> = sheet
                .parts
                .iter()
                .map(|placed| {
                    generate_profile_cut(
                        &placed.rect, group.material.thickness,
                        tool, rpm, &cam_config,
                    )
                })
                .collect();

            let gcode = emitter.emit(&toolpaths);
            let safe_name = group.material_name.replace(['/', '\\', '"', ' '], "_");

            result.push(SheetGcode {
                material: group.material_name.clone(),
                sheet_index: sheet.sheet_index,
                filename: format!("{}_sheet_{}.ngc", safe_name, sheet.sheet_index + 1),
                gcode,
            });
        }
    }

    Ok(Json(result))
}

#[derive(serde::Serialize)]
pub struct SheetGcode {
    pub material: String,
    pub sheet_index: usize,
    pub filename: String,
    pub gcode: String,
}

/// GET /api/sessions/:id/gcode/:material_index/:sheet_index — Preview G-code for a sheet.
pub async fn preview_gcode(
    State(state): State<Arc<AppState>>,
    Path((id, material_index, sheet_index)): Path<(Uuid, usize, usize)>,
) -> Result<String, ServerError> {
    let session = get_session(&state, &id)?;
    let guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_ref().ok_or(ServerError::NoProject)?;
    let machine = session.machine.lock().map_err(lock_err)?;

    let tagged = project.generate_all_parts();
    let groups = Project::group_parts_by_material(&tagged);

    let group = groups.get(material_index)
        .ok_or(ServerError::IndexOutOfRange(format!("Material index {material_index}")))?;

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
        .ok_or(ServerError::IndexOutOfRange(format!("Sheet index {sheet_index}")))?;

    let units = project.project.units;
    let emitter = GCodeEmitter::new(&machine, units);
    let cam_config = CamConfig::default();
    let rpm = 5000.0;

    let tool = project.tools.first()
        .ok_or(ServerError::BadRequest("No tools defined".into()))?;

    let toolpaths: Vec<_> = sheet
        .parts
        .iter()
        .map(|placed| {
            generate_profile_cut(
                &placed.rect, group.material.thickness,
                tool, rpm, &cam_config,
            )
        })
        .collect();

    Ok(emitter.emit(&toolpaths))
}

/// GET /api/sessions/:id/gcode/:material_index/:sheet_index/toolpaths — Get annotated toolpaths for visualization.
pub async fn get_toolpaths(
    State(state): State<Arc<AppState>>,
    Path((id, material_index, sheet_index)): Path<(Uuid, usize, usize)>,
) -> Result<Json<ToolpathVisualizationDto>, ServerError> {
    let session = get_session(&state, &id)?;
    let guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_ref().ok_or(ServerError::NoProject)?;
    let machine = session.machine.lock().map_err(lock_err)?;

    let tagged = project.generate_all_parts();
    let groups = Project::group_parts_by_material(&tagged);

    let group = groups.get(material_index)
        .ok_or(ServerError::IndexOutOfRange(format!("Material index {material_index}")))?;

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
        .ok_or(ServerError::IndexOutOfRange(format!("Sheet index {sheet_index}")))?;

    let cam_config = CamConfig {
        safe_z: machine.post.safe_z,
        rapid_z: machine.post.rapid_z,
        ..Default::default()
    };
    let rpm = 5000.0;

    let tool = project.tools.first()
        .ok_or(ServerError::BadRequest("No tools defined".into()))?;

    let dto = generate_annotated_toolpaths(sheet, &group.parts, tool, rpm, &cam_config);

    Ok(Json(dto))
}
