use serde::{Deserialize, Serialize};
use tauri::State;

use cm_cabinet::cabinet::Cabinet;
use cm_cabinet::project::{CabinetEntry, TaggedPart};
use cm_cabinet::Part;

use crate::state::AppState;

/// Generate all parts from the current project. Caches the result.
#[tauri::command]
pub fn generate_parts(state: State<'_, AppState>) -> Result<Vec<TaggedPart>, String> {
    let guard = state.project.lock().map_err(|e| e.to_string())?;
    let project = guard.as_ref().ok_or("No project loaded")?;
    let parts = project.generate_all_parts();

    drop(guard);
    *state.cached_parts.lock().map_err(|e| e.to_string())? = Some(parts.clone());

    Ok(parts)
}

/// Add a new cabinet entry to the project.
#[tauri::command]
pub fn add_cabinet(entry: CabinetEntry, state: State<'_, AppState>) -> Result<usize, String> {
    let mut guard = state.project.lock().map_err(|e| e.to_string())?;
    let project = guard.as_mut().ok_or("No project loaded")?;
    project.cabinets.push(entry);
    let index = project.cabinets.len() - 1;

    drop(guard);
    *state.cached_parts.lock().map_err(|e| e.to_string())? = None;

    Ok(index)
}

/// Update a cabinet entry at a given index.
#[tauri::command]
pub fn update_cabinet(
    index: usize,
    entry: CabinetEntry,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut guard = state.project.lock().map_err(|e| e.to_string())?;
    let project = guard.as_mut().ok_or("No project loaded")?;

    if index >= project.cabinets.len() {
        return Err(format!("Cabinet index {index} out of range"));
    }
    project.cabinets[index] = entry;

    drop(guard);
    *state.cached_parts.lock().map_err(|e| e.to_string())? = None;

    Ok(())
}

/// Remove a cabinet entry at a given index.
#[tauri::command]
pub fn remove_cabinet(index: usize, state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.project.lock().map_err(|e| e.to_string())?;
    let project = guard.as_mut().ok_or("No project loaded")?;

    if index >= project.cabinets.len() {
        return Err(format!("Cabinet index {index} out of range"));
    }
    project.cabinets.remove(index);

    drop(guard);
    *state.cached_parts.lock().map_err(|e| e.to_string())? = None;

    Ok(())
}

/// Preview parts for a single cabinet (without saving to project).
#[tauri::command]
pub fn preview_cabinet_parts(cabinet: Cabinet) -> Result<Vec<Part>, String> {
    Ok(cabinet.generate_parts())
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

/// Get 3D assembly data for a cabinet (panel positions for Three.js).
#[tauri::command]
pub fn get_3d_assembly(
    cabinet_index: usize,
    state: State<'_, AppState>,
) -> Result<Vec<Panel3D>, String> {
    let guard = state.project.lock().map_err(|e| e.to_string())?;
    let project = guard.as_ref().ok_or("No project loaded")?;

    let cabinet = if cabinet_index < project.cabinets.len() {
        &project.cabinets[cabinet_index].cabinet
    } else if cabinet_index == 0 && project.cabinet.is_some() {
        project.cabinet.as_ref().unwrap()
    } else {
        return Err(format!("Cabinet index {cabinet_index} out of range"));
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
                width: mt,
                height: part.rect.height,
                depth: d,
                x: mt / 2.0,
                y: toe_h + part.rect.height / 2.0,
                z: d / 2.0,
                color: "#c4a882".into(),
            },
            "right_side" => Panel3D {
                label: part.label.clone(),
                width: mt,
                height: part.rect.height,
                depth: d,
                x: w - mt / 2.0,
                y: toe_h + part.rect.height / 2.0,
                z: d / 2.0,
                color: "#c4a882".into(),
            },
            "top" => Panel3D {
                label: part.label.clone(),
                width: part.rect.width,
                height: mt,
                depth: d,
                x: w / 2.0,
                y: h - mt / 2.0,
                z: d / 2.0,
                color: "#b89b72".into(),
            },
            "bottom" => Panel3D {
                label: part.label.clone(),
                width: part.rect.width,
                height: mt,
                depth: d,
                x: w / 2.0,
                y: toe_h + mt / 2.0,
                z: d / 2.0,
                color: "#b89b72".into(),
            },
            "back" => Panel3D {
                label: part.label.clone(),
                width: part.rect.width,
                height: part.rect.height,
                depth: bt,
                x: w / 2.0,
                y: toe_h + part.rect.height / 2.0,
                z: bt / 2.0,
                color: "#d4c4a8".into(),
            },
            label if label.starts_with("shelf") => {
                // Distribute shelves evenly in the interior
                let interior_h = h - toe_h - 2.0 * mt;
                let shelf_count = parts.iter().filter(|p| p.label.starts_with("shelf")).count();
                let shelf_idx: usize = label.strip_prefix("shelf_")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let spacing = interior_h / (shelf_count as f64 + 1.0);
                let y_pos = toe_h + mt + spacing * (shelf_idx as f64 + 1.0);

                Panel3D {
                    label: part.label.clone(),
                    width: part.rect.width,
                    height: mt,
                    depth: d - bt,
                    x: w / 2.0,
                    y: y_pos,
                    z: (d - bt) / 2.0 + bt,
                    color: "#a88b62".into(),
                }
            }
            _ => Panel3D {
                label: part.label.clone(),
                width: part.rect.width,
                height: part.rect.height,
                depth: mt,
                x: w / 2.0,
                y: h / 2.0,
                z: d / 2.0,
                color: "#ccc".into(),
            },
        };
        panels.push(panel);
    }

    Ok(panels)
}
