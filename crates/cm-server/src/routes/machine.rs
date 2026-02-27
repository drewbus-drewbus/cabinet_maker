use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use cm_post::machine::MachineProfile;

use crate::error::{ServerError, lock_err};
use crate::state::AppState;

fn get_session(state: &AppState, id: &Uuid) -> Result<Arc<crate::state::SessionState>, ServerError> {
    state.get_session(id).ok_or(ServerError::SessionNotFound)
}

/// GET /api/sessions/:id/machine — Get the machine profile.
pub async fn get_machine(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<MachineProfile>, ServerError> {
    let session = get_session(&state, &id)?;
    let guard = session.machine.lock().map_err(lock_err)?;
    Ok(Json(guard.clone()))
}

/// PUT /api/sessions/:id/machine — Set the machine profile.
pub async fn set_machine(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(profile): Json<MachineProfile>,
) -> Result<Json<()>, ServerError> {
    let session = get_session(&state, &id)?;
    *session.machine.lock().map_err(lock_err)? = profile;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct UploadMachineRequest {
    pub toml_content: String,
}

/// POST /api/sessions/:id/machine/upload — Upload a machine profile TOML.
pub async fn upload_machine_profile(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UploadMachineRequest>,
) -> Result<Json<MachineProfile>, ServerError> {
    let session = get_session(&state, &id)?;

    let profile = MachineProfile::from_toml(&req.toml_content)
        .map_err(|e| ServerError::BadRequest(format!("Failed to parse machine profile: {e}")))?;

    *session.machine.lock().map_err(lock_err)? = profile.clone();

    Ok(Json(profile))
}
