use std::fs;
use std::path::PathBuf;

use tauri::State;

use cm_cabinet::project::Project;

use crate::state::AppState;

/// Create a new empty project.
#[tauri::command]
pub fn new_project(state: State<'_, AppState>) -> Result<Project, String> {
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

    *state.project.lock().map_err(|e| e.to_string())? = Some(project.clone());
    *state.project_path.lock().map_err(|e| e.to_string())? = None;
    *state.cached_parts.lock().map_err(|e| e.to_string())? = None;

    Ok(project)
}

/// Get the current project.
#[tauri::command]
pub fn get_project(state: State<'_, AppState>) -> Result<Option<Project>, String> {
    let guard = state.project.lock().map_err(|e| e.to_string())?;
    Ok(guard.clone())
}

/// Update the entire project from the frontend.
#[tauri::command]
pub fn update_project(project: Project, state: State<'_, AppState>) -> Result<(), String> {
    *state.project.lock().map_err(|e| e.to_string())? = Some(project);
    *state.cached_parts.lock().map_err(|e| e.to_string())? = None;
    Ok(())
}

/// Open a project from a TOML file path.
#[tauri::command]
pub fn open_project(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {e}"))?;
    let project = Project::from_toml(&contents)
        .map_err(|e| format!("Failed to parse TOML: {e}"))?;

    *state.project.lock().map_err(|e| e.to_string())? = Some(project.clone());
    *state.project_path.lock().map_err(|e| e.to_string())? = Some(path);
    *state.cached_parts.lock().map_err(|e| e.to_string())? = None;

    Ok(project)
}

/// Save the current project to a TOML file.
/// If `path` is None, uses the previously saved path.
#[tauri::command]
pub fn save_project(path: Option<String>, state: State<'_, AppState>) -> Result<String, String> {
    let guard = state.project.lock().map_err(|e| e.to_string())?;
    let project = guard.as_ref().ok_or("No project loaded")?;

    let save_path = if let Some(p) = path {
        p
    } else {
        state.project_path.lock().map_err(|e| e.to_string())?
            .clone()
            .ok_or("No file path set — use Save As")?
    };

    let toml_str = project.to_toml()
        .map_err(|e| format!("Failed to serialize project: {e}"))?;
    fs::write(&save_path, &toml_str)
        .map_err(|e| format!("Failed to write file: {e}"))?;

    drop(guard);
    *state.project_path.lock().map_err(|e| e.to_string())? = Some(save_path.clone());

    Ok(save_path)
}

/// Load a built-in template by name.
#[tauri::command]
pub fn load_template(name: String, state: State<'_, AppState>) -> Result<Project, String> {
    // Look for templates relative to the executable or in a well-known location
    let template_dirs = [
        PathBuf::from("templates"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates"),
    ];

    let filename = format!("{name}.toml");

    for dir in &template_dirs {
        let path = dir.join(&filename);
        if path.exists() {
            let contents = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read template: {e}"))?;
            let project = Project::from_toml(&contents)
                .map_err(|e| format!("Failed to parse template: {e}"))?;

            *state.project.lock().map_err(|e| e.to_string())? = Some(project.clone());
            *state.project_path.lock().map_err(|e| e.to_string())? = None;
            *state.cached_parts.lock().map_err(|e| e.to_string())? = None;

            return Ok(project);
        }
    }

    Err(format!("Template '{name}' not found"))
}

/// List available template names.
#[tauri::command]
pub fn list_templates() -> Result<Vec<String>, String> {
    let template_dirs = [
        PathBuf::from("templates"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates"),
    ];

    let mut templates = Vec::new();

    for dir in &template_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "toml") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if !templates.contains(&stem.to_string()) {
                            templates.push(stem.to_string());
                        }
                    }
                }
            }
        }
    }

    templates.sort();
    Ok(templates)
}

/// Get the current project file path.
#[tauri::command]
pub fn get_project_path(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let guard = state.project_path.lock().map_err(|e| e.to_string())?;
    Ok(guard.clone())
}
