use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ServerError;
use crate::state::AppState;

#[derive(Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
}

/// POST /api/sessions — Create a new session.
pub async fn create_session(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SessionInfo>, ServerError> {
    let (id, _session) = state.create_session()
        .ok_or(ServerError::Internal("Failed to create session".into()))?;
    Ok(Json(SessionInfo { id: id.to_string() }))
}

/// GET /api/sessions/:id — Check if a session exists.
pub async fn get_session_info(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SessionInfo>, ServerError> {
    state.get_session(&id).ok_or(ServerError::SessionNotFound)?;
    Ok(Json(SessionInfo { id: id.to_string() }))
}

/// DELETE /api/sessions/:id — Delete a session.
pub async fn delete_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SessionInfo>, ServerError> {
    if state.remove_session(&id) {
        Ok(Json(SessionInfo { id: id.to_string() }))
    } else {
        Err(ServerError::SessionNotFound)
    }
}
