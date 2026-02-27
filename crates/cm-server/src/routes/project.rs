use std::fs;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use cm_cabinet::project::Project;

use crate::error::{ServerError, lock_err};
use crate::state::AppState;

/// Helper to get a session or return an error.
fn get_session(state: &AppState, id: &Uuid) -> Result<Arc<crate::state::SessionState>, ServerError> {
    state.get_session(id).ok_or(ServerError::SessionNotFound)
}

/// POST /api/sessions/:id/project/new — Create a new empty project.
pub async fn new_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Project>, ServerError> {
    let session = get_session(&state, &id)?;

    let project = Project {
        project: cm_cabinet::project::ProjectMeta {
            name: "Untitled Project".into(),
            units: cm_core::units::Unit::Inches,
        },
        material: None,
        back_material: None,
        cabinet: None,
        materials: vec![cm_core::material::Material::plywood_3_4()],
        cabinets: Vec::new(),
        tools: vec![cm_core::tool::Tool::quarter_inch_endmill()],
    };

    *session.project.lock().map_err(lock_err)? = Some(project.clone());
    *session.project_filename.lock().map_err(lock_err)? = None;
    *session.cached_parts.lock().map_err(lock_err)? = None;

    Ok(Json(project))
}

/// GET /api/sessions/:id/project — Get the current project.
pub async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Option<Project>>, ServerError> {
    let session = get_session(&state, &id)?;
    let guard = session.project.lock().map_err(lock_err)?;
    Ok(Json(guard.clone()))
}

/// PUT /api/sessions/:id/project — Update the entire project.
pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(project): Json<Project>,
) -> Result<Json<()>, ServerError> {
    let session = get_session(&state, &id)?;
    *session.project.lock().map_err(lock_err)? = Some(project);
    *session.cached_parts.lock().map_err(lock_err)? = None;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct OpenProjectRequest {
    pub toml_content: String,
    pub filename: Option<String>,
}

/// POST /api/sessions/:id/project/open — Open a project from TOML content.
pub async fn open_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<OpenProjectRequest>,
) -> Result<Json<Project>, ServerError> {
    let session = get_session(&state, &id)?;

    let project = Project::from_toml(&req.toml_content)
        .map_err(|e| ServerError::BadRequest(format!("Failed to parse TOML: {e}")))?;

    *session.project.lock().map_err(lock_err)? = Some(project.clone());
    *session.project_filename.lock().map_err(lock_err)? = req.filename;
    *session.cached_parts.lock().map_err(lock_err)? = None;

    Ok(Json(project))
}

/// POST /api/sessions/:id/project/save — Save the project as TOML (returns TOML string).
pub async fn save_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<String, ServerError> {
    let session = get_session(&state, &id)?;
    let guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_ref().ok_or(ServerError::NoProject)?;

    let toml_str = project.to_toml()
        .map_err(|e| ServerError::Internal(format!("Failed to serialize project: {e}")))?;

    Ok(toml_str)
}

/// POST /api/sessions/:id/project/template/*name — Load a template.
/// Supports nested paths like "sets/l-shaped-kitchen".
pub async fn load_template(
    State(state): State<Arc<AppState>>,
    Path((id, name)): Path<(Uuid, String)>,
) -> Result<Json<Project>, ServerError> {
    let session = get_session(&state, &id)?;

    // Prevent path traversal
    if name.contains("..") {
        return Err(ServerError::BadRequest("Invalid template name".into()));
    }

    let filename = format!("{name}.toml");
    let path = state.templates_dir.join(&filename);

    // Ensure resolved path stays within templates_dir
    let canonical = path.canonicalize()
        .map_err(|_| ServerError::NotFound(format!("Template '{name}' not found")))?;
    let base = state.templates_dir.canonicalize()
        .map_err(|e| ServerError::Internal(format!("Templates dir error: {e}")))?;
    if !canonical.starts_with(&base) {
        return Err(ServerError::BadRequest("Invalid template path".into()));
    }

    let contents = fs::read_to_string(&canonical)?;
    let project = Project::from_toml(&contents)
        .map_err(|e| ServerError::BadRequest(format!("Failed to parse template: {e}")))?;

    *session.project.lock().map_err(lock_err)? = Some(project.clone());
    *session.project_filename.lock().map_err(lock_err)? = None;
    *session.cached_parts.lock().map_err(lock_err)? = None;

    Ok(Json(project))
}

/// GET /api/templates — List available template names (session-independent).
/// Returns paths relative to templates dir, e.g. "bookshelf", "sets/l-shaped-kitchen".
pub async fn list_templates(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<String>>, ServerError> {
    let mut templates = Vec::new();
    collect_templates(&state.templates_dir, &state.templates_dir, &mut templates);
    templates.sort();
    Ok(Json(templates))
}

fn collect_templates(base: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_templates(base, &path, out);
        } else if path.extension().is_some_and(|ext| ext == "toml")
            && let Ok(rel) = path.strip_prefix(base)
        {
            let name = rel.with_extension("");
            if let Some(s) = name.to_str() {
                out.push(s.to_string());
            }
        }
    }
}
