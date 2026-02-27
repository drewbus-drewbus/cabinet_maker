use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use cm_pipeline::QuickEstimate;

use crate::error::{ServerError, lock_err};
use crate::state::AppState;

/// GET /api/sessions/:id/estimate — Quick design-time cost estimate.
pub async fn get_estimate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<QuickEstimate>, ServerError> {
    let session = state.get_session(&id).ok_or(ServerError::SessionNotFound)?;
    let guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_ref().ok_or(ServerError::NoProject)?;

    let estimate = cm_pipeline::quick_estimate(project);
    Ok(Json(estimate))
}
