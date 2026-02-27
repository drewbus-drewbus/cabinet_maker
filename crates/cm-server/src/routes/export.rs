use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ServerError, lock_err};
use crate::state::AppState;

fn get_session(state: &AppState, id: &Uuid) -> Result<Arc<crate::state::SessionState>, ServerError> {
    state.get_session(id).ok_or(ServerError::SessionNotFound)
}

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

/// Helper to build cut list rows from a session.
fn build_cutlist(session: &crate::state::SessionState) -> Result<Vec<CutlistRow>, ServerError> {
    let guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_ref().ok_or(ServerError::NoProject)?;

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
                    cm_cabinet::part::PartOperation::Dovetail(d) => {
                        format!("Dovetail {:?} {} tails, depth {:.3}\"", d.edge, d.tail_count, d.depth)
                    }
                    cm_cabinet::part::PartOperation::BoxJoint(b) => {
                        format!("Box joint {:?} {} fingers, depth {:.3}\"", b.edge, b.finger_count, b.depth)
                    }
                    cm_cabinet::part::PartOperation::Mortise(m) => {
                        format!("Mortise {:.3}\"w x {:.3}\"l x {:.3}\"d @ ({:.3}\", {:.3}\")", m.width, m.length, m.depth, m.x, m.y)
                    }
                    cm_cabinet::part::PartOperation::Tenon(t) => {
                        format!("Tenon {:?} {:.3}\" x {:.3}\" x {:.3}\"", t.edge, t.thickness, t.width, t.length)
                    }
                    cm_cabinet::part::PartOperation::Dowel(d) => {
                        format!("Dowel {} holes, {:.3}\" dia, depth {:.3}\"", d.holes.len(), d.dowel_diameter, d.depth)
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

/// GET /api/sessions/:id/export/cutlist — Get cut list as JSON.
pub async fn get_cutlist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<CutlistRow>>, ServerError> {
    let session = get_session(&state, &id)?;
    let rows = build_cutlist(&session)?;
    Ok(Json(rows))
}

/// GET /api/sessions/:id/export/csv — Download cut list as CSV.
pub async fn export_csv(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServerError> {
    let session = get_session(&state, &id)?;
    let rows = build_cutlist(&session)?;

    let mut csv = String::from("Cabinet,Label,Material,Width,Height,Thickness,Qty,Grain,Operations\n");
    for row in &rows {
        csv.push_str(&format!(
            "{},{},{},{:.4},{:.4},{:.4},{},{},\"{}\"\n",
            csv_escape(&row.cabinet),
            csv_escape(&row.label),
            csv_escape(&row.material),
            row.width,
            row.height,
            row.thickness,
            row.quantity,
            row.grain,
            row.operations.join("; "),
        ));
    }

    Ok((
        [(header::CONTENT_TYPE, "text/csv"), (header::CONTENT_DISPOSITION, "attachment; filename=\"cutlist.csv\"")],
        csv,
    ))
}

/// GET /api/sessions/:id/export/bom — Download BOM as JSON.
pub async fn export_bom(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServerError> {
    let session = get_session(&state, &id)?;
    let guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_ref().ok_or(ServerError::NoProject)?;

    let tagged = project.generate_all_parts();
    let primary_mat = project.primary_material();
    let cost_per_sheet = primary_mat.and_then(|m| m.cost_per_unit);
    let bom = cm_pipeline::generate_bom(project, &tagged, 0, cost_per_sheet);
    let json = serde_json::to_string_pretty(&bom)
        .map_err(|e| ServerError::Internal(format!("Failed to serialize BOM: {e}")))?;

    Ok((
        [(header::CONTENT_TYPE, "application/json"), (header::CONTENT_DISPOSITION, "attachment; filename=\"bom.json\"")],
        json,
    ))
}

/// GET /api/sessions/:id/export/dxf — Export all parts as a DXF file.
pub async fn export_dxf(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServerError> {
    let session = get_session(&state, &id)?;
    let guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_ref().ok_or(ServerError::NoProject)?;

    let tagged = project.generate_all_parts();

    // Build a sheet-style DXF with all parts laid out sequentially
    let placed: Vec<cm_import::dxf_export::ExportPlacedPart<'_>> = tagged
        .iter()
        .map(|tp| cm_import::dxf_export::ExportPlacedPart {
            id: &tp.part.label,
            rect: &tp.part.rect,
        })
        .collect();

    // Use a virtual sheet that fits all parts
    let max_x = tagged.iter().map(|tp| tp.part.rect.origin.x + tp.part.rect.width).fold(0.0f64, f64::max);
    let max_y = tagged.iter().map(|tp| tp.part.rect.origin.y + tp.part.rect.height).fold(0.0f64, f64::max);
    let sheet_rect = cm_core::geometry::Rect::new(
        cm_core::geometry::Point2D::new(0.0, 0.0),
        max_x.max(1.0),
        max_y.max(1.0),
    );

    let bytes = cm_import::dxf_export::export_sheet_dxf(&sheet_rect, &placed)
        .map_err(|e| ServerError::Internal(format!("DXF export failed: {e}")))?;

    Ok((
        [(header::CONTENT_TYPE, "application/dxf"), (header::CONTENT_DISPOSITION, "attachment; filename=\"parts.dxf\"")],
        bytes,
    ))
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
