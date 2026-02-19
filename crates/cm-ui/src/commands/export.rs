use std::fs;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// A row in the cut list table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutlistRow {
    pub cabinet: String,
    pub label: String,
    pub material: String,
    pub width: f64,
    pub height: f64,
    pub thickness: f64,
    pub quantity: u32,
    pub grain: String,
    pub operations: Vec<String>,
}

/// Get the cut list as structured data for the frontend table.
#[tauri::command]
pub fn get_cutlist(state: State<'_, AppState>) -> Result<Vec<CutlistRow>, String> {
    let guard = state.project.lock().map_err(|e| e.to_string())?;
    let project = guard.as_ref().ok_or("No project loaded")?;

    let tagged = project.generate_all_parts();
    let rows: Vec<CutlistRow> = tagged
        .iter()
        .map(|tp| {
            let ops: Vec<String> = tp.part.operations.iter().map(|op| {
                match op {
                    cm_cabinet::part::PartOperation::Dado(d) => {
                        format!("Dado {:.3}\"w x {:.3}\"d @ {:.3}\"", d.width, d.depth, d.position)
                    }
                    cm_cabinet::part::PartOperation::Rabbet(r) => {
                        format!("Rabbet {:?} {:.3}\"w x {:.3}\"d", r.edge, r.width, r.depth)
                    }
                    cm_cabinet::part::PartOperation::Drill(d) => {
                        format!("Drill {:.3}\"dia x {:.3}\"d @ ({:.3}\", {:.3}\")", d.diameter, d.depth, d.x, d.y)
                    }
                    cm_cabinet::part::PartOperation::PocketHole(p) => {
                        format!("Pocket hole {:?} @ ({:.3}\", {:.3}\")", p.edge, p.x, p.y)
                    }
                }
            }).collect();

            CutlistRow {
                cabinet: tp.cabinet_name.clone(),
                label: tp.part.label.clone(),
                material: tp.material_name.clone(),
                width: tp.part.rect.width,
                height: tp.part.rect.height,
                thickness: tp.material.thickness,
                quantity: tp.part.quantity,
                grain: format!("{:?}", tp.part.grain_direction),
                operations: ops,
            }
        })
        .collect();

    Ok(rows)
}

/// Export cut list as CSV.
#[tauri::command]
pub fn export_csv(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let rows = get_cutlist(state)?;

    let mut csv = String::from("Cabinet,Label,Material,Width,Height,Thickness,Qty,Grain,Operations\n");
    for row in &rows {
        csv.push_str(&format!(
            "{},{},{},{:.4},{:.4},{:.4},{},{},\"{}\"\n",
            row.cabinet,
            row.label,
            row.material,
            row.width,
            row.height,
            row.thickness,
            row.quantity,
            row.grain,
            row.operations.join("; "),
        ));
    }

    fs::write(&path, &csv)
        .map_err(|e| format!("Failed to write CSV: {e}"))?;

    Ok(())
}

/// Export BOM as JSON.
#[tauri::command]
pub fn export_bom_json(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let rows = get_cutlist(state)?;
    let json = serde_json::to_string_pretty(&rows)
        .map_err(|e| format!("Failed to serialize BOM: {e}"))?;

    fs::write(&path, &json)
        .map_err(|e| format!("Failed to write BOM JSON: {e}"))?;

    Ok(())
}
