use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use cm_cabinet::cabinet::Cabinet;
use cm_cabinet::part::Part;
use cm_cabinet::project::{CabinetEntry, TaggedPart};

use crate::error::{ServerError, lock_err};
use crate::state::AppState;

fn get_session(state: &AppState, id: &Uuid) -> Result<Arc<crate::state::SessionState>, ServerError> {
    state.get_session(id).ok_or(ServerError::SessionNotFound)
}

/// POST /api/sessions/:id/parts — Generate all parts.
pub async fn generate_parts(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<TaggedPart>>, ServerError> {
    let session = get_session(&state, &id)?;
    let guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_ref().ok_or(ServerError::NoProject)?;
    let parts = project.generate_all_parts();

    drop(guard);
    *session.cached_parts.lock().map_err(lock_err)? = Some(parts.clone());

    Ok(Json(parts))
}

/// POST /api/sessions/:id/cabinets — Add a cabinet.
pub async fn add_cabinet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(entry): Json<CabinetEntry>,
) -> Result<Json<usize>, ServerError> {
    let session = get_session(&state, &id)?;
    let mut guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_mut().ok_or(ServerError::NoProject)?;
    project.cabinets.push(entry);
    let index = project.cabinets.len() - 1;

    drop(guard);
    *session.cached_parts.lock().map_err(lock_err)? = None;

    Ok(Json(index))
}

/// PUT /api/sessions/:id/cabinets/:index — Update a cabinet.
pub async fn update_cabinet(
    State(state): State<Arc<AppState>>,
    Path((id, index)): Path<(Uuid, usize)>,
    Json(entry): Json<CabinetEntry>,
) -> Result<Json<()>, ServerError> {
    let session = get_session(&state, &id)?;
    let mut guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_mut().ok_or(ServerError::NoProject)?;

    if index >= project.cabinets.len() {
        return Err(ServerError::IndexOutOfRange(format!("Cabinet index {index}")));
    }
    project.cabinets[index] = entry;

    drop(guard);
    *session.cached_parts.lock().map_err(lock_err)? = None;

    Ok(Json(()))
}

/// DELETE /api/sessions/:id/cabinets/:index — Remove a cabinet.
pub async fn remove_cabinet(
    State(state): State<Arc<AppState>>,
    Path((id, index)): Path<(Uuid, usize)>,
) -> Result<Json<()>, ServerError> {
    let session = get_session(&state, &id)?;
    let mut guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_mut().ok_or(ServerError::NoProject)?;

    if index >= project.cabinets.len() {
        return Err(ServerError::IndexOutOfRange(format!("Cabinet index {index}")));
    }
    project.cabinets.remove(index);

    drop(guard);
    *session.cached_parts.lock().map_err(lock_err)? = None;

    Ok(Json(()))
}

/// POST /api/sessions/:id/cabinets/preview — Preview parts for a single cabinet.
pub async fn preview_cabinet_parts(
    Json(cabinet): Json<Cabinet>,
) -> Result<Json<Vec<Part>>, ServerError> {
    Ok(Json(cabinet.generate_parts()))
}

/// 3D panel data for Three.js rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panel3D {
    pub label: String,
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub color: String,
}

/// GET /api/sessions/:id/cabinets/:index/3d — Get 3D assembly data.
pub async fn get_3d_assembly(
    State(state): State<Arc<AppState>>,
    Path((id, cabinet_index)): Path<(Uuid, usize)>,
) -> Result<Json<Vec<Panel3D>>, ServerError> {
    let session = get_session(&state, &id)?;
    let guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_ref().ok_or(ServerError::NoProject)?;

    let cabinet = if cabinet_index < project.cabinets.len() {
        &project.cabinets[cabinet_index].cabinet
    } else if cabinet_index == 0 && project.cabinet.is_some() {
        project.cabinet.as_ref().unwrap()
    } else {
        return Err(ServerError::IndexOutOfRange(format!("Cabinet index {cabinet_index}")));
    };

    let parts = cabinet.generate_parts();
    let w = cabinet.width;
    let h = cabinet.height;
    let d = cabinet.depth;
    let mt = cabinet.material_thickness;
    let bt = cabinet.back_thickness;
    let toe_h = cabinet.toe_kick.as_ref().map_or(0.0, |tk| tk.height);

    let mut panels = Vec::new();

    for part in &parts {
        let panel = match part.label.as_str() {
            "left_side" => Panel3D {
                label: part.label.clone(),
                width: mt, height: part.rect.height, depth: d,
                x: mt / 2.0,
                y: toe_h + part.rect.height / 2.0,
                z: d / 2.0,
                color: "#c4a882".into(),
            },
            "right_side" => Panel3D {
                label: part.label.clone(),
                width: mt, height: part.rect.height, depth: d,
                x: w - mt / 2.0,
                y: toe_h + part.rect.height / 2.0,
                z: d / 2.0,
                color: "#c4a882".into(),
            },
            "top" => Panel3D {
                label: part.label.clone(),
                width: part.rect.width, height: mt, depth: d,
                x: w / 2.0,
                y: h - mt / 2.0,
                z: d / 2.0,
                color: "#b89b72".into(),
            },
            "bottom" => Panel3D {
                label: part.label.clone(),
                width: part.rect.width, height: mt, depth: d,
                x: w / 2.0,
                y: toe_h + mt / 2.0,
                z: d / 2.0,
                color: "#b89b72".into(),
            },
            "back" => Panel3D {
                label: part.label.clone(),
                width: part.rect.width, height: part.rect.height, depth: bt,
                x: w / 2.0,
                y: toe_h + part.rect.height / 2.0,
                z: bt / 2.0,
                color: "#d4c4a8".into(),
            },
            label if label.starts_with("shelf") => {
                let interior_h = h - toe_h - 2.0 * mt;
                let shelf_count = parts.iter().filter(|p| p.label.starts_with("shelf")).count();
                let shelf_idx: usize = label.strip_prefix("shelf_")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let spacing = interior_h / (shelf_count as f64 + 1.0);
                let y_pos = toe_h + mt + spacing * (shelf_idx as f64 + 1.0);

                Panel3D {
                    label: part.label.clone(),
                    width: part.rect.width, height: mt, depth: d - bt,
                    x: w / 2.0,
                    y: y_pos,
                    z: (d - bt) / 2.0 + bt,
                    color: "#a88b62".into(),
                }
            }
            _ => Panel3D {
                label: part.label.clone(),
                width: part.rect.width, height: part.rect.height, depth: mt,
                x: w / 2.0,
                y: h / 2.0,
                z: d / 2.0,
                color: "#ccc".into(),
            },
        };
        panels.push(panel);
    }

    Ok(Json(panels))
}
