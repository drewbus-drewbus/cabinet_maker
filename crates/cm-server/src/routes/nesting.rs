use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use cm_cabinet::project::Project;
use cm_nesting::packer::{nest_parts, NestingConfig, NestingPart, NestingResult};
use cm_nesting::validate::{ManualPlacement, PlacementValidation, validate_manual_placement};

use crate::error::{ServerError, lock_err};
use crate::state::AppState;

fn get_session(state: &AppState, id: &Uuid) -> Result<Arc<crate::state::SessionState>, ServerError> {
    state.get_session(id).ok_or(ServerError::SessionNotFound)
}

/// Owned material group DTO for JSON responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialGroupDto {
    pub material_name: String,
    pub thickness: f64,
    pub nesting_result: NestingResult,
}

/// Request body for nest_all.
#[derive(Deserialize)]
pub struct NestAllRequest {
    pub config: Option<NestingConfig>,
}

/// Helper: run nesting for a session, returning material group DTOs.
fn run_nesting(
    session: &crate::state::SessionState,
    config: Option<NestingConfig>,
) -> Result<Vec<MaterialGroupDto>, ServerError> {
    let guard = session.project.lock().map_err(lock_err)?;
    let project = guard.as_ref().ok_or(ServerError::NoProject)?;

    // Use cached parts or generate
    let parts_guard = session.cached_parts.lock().map_err(lock_err)?;
    let tagged = if let Some(ref cached) = *parts_guard {
        cached.clone()
    } else {
        drop(parts_guard);
        let tagged = project.generate_all_parts();
        *session.cached_parts.lock().map_err(lock_err)? = Some(tagged.clone());
        tagged
    };

    let groups = Project::group_parts_by_material(&tagged);
    let mut results = Vec::new();

    for group in groups {
        let nesting_config = config.clone().unwrap_or_else(|| NestingConfig {
            sheet_width: group.material.sheet_width.unwrap_or(48.0),
            sheet_length: group.material.sheet_length.unwrap_or(96.0),
            ..NestingConfig::default()
        });

        let nesting_parts: Vec<NestingPart> = group
            .parts
            .iter()
            .flat_map(|tp| {
                (0..tp.part.quantity).map(move |i| {
                    let id = if tp.part.quantity > 1 {
                        format!("{}:{}:{}", tp.cabinet_name, tp.part.label, i)
                    } else {
                        format!("{}:{}", tp.cabinet_name, tp.part.label)
                    };
                    NestingPart {
                        id,
                        width: tp.part.rect.width,
                        height: tp.part.rect.height,
                        can_rotate: false,
                    }
                })
            })
            .collect();

        let nesting_result = nest_parts(&nesting_parts, &nesting_config);

        results.push(MaterialGroupDto {
            material_name: group.material_name,
            thickness: group.material.thickness,
            nesting_result,
        });
    }

    Ok(results)
}

/// POST /api/sessions/:id/nesting — Nest all parts.
pub async fn nest_all(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<NestAllRequest>,
) -> Result<Json<Vec<MaterialGroupDto>>, ServerError> {
    let session = get_session(&state, &id)?;
    let results = run_nesting(&session, req.config)?;
    Ok(Json(results))
}

/// GET /api/sessions/:id/nesting/:material_index/:sheet_index/svg — Get nesting SVG.
pub async fn get_nesting_svg(
    State(state): State<Arc<AppState>>,
    Path((id, material_index, sheet_index)): Path<(Uuid, usize, usize)>,
) -> Result<axum::response::Html<String>, ServerError> {
    let session = get_session(&state, &id)?;
    let groups = run_nesting(&session, None)?;

    let group = groups.get(material_index)
        .ok_or(ServerError::IndexOutOfRange(format!("Material index {material_index}")))?;
    let sheet = group.nesting_result.sheets.get(sheet_index)
        .ok_or(ServerError::IndexOutOfRange(format!("Sheet index {sheet_index}")))?;

    let sw = sheet.sheet_rect.width;
    let sh = sheet.sheet_rect.height;
    let scale = 8.0;

    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"##,
        sw * scale, sh * scale, sw * scale, sh * scale,
    );

    svg.push_str(&format!(
        r##"<rect x="0" y="0" width="{}" height="{}" fill="#f5f0e8" stroke="#999" stroke-width="1"/>"##,
        sw * scale, sh * scale,
    ));

    let colors = ["#6b9bd2", "#e8a838", "#5cb85c", "#d9534f", "#9b59b6", "#1abc9c", "#e67e22", "#3498db"];

    for (i, part) in sheet.parts.iter().enumerate() {
        let color = colors[i % colors.len()];
        let x = part.rect.origin.x * scale;
        let y = part.rect.origin.y * scale;
        let w = part.rect.width * scale;
        let h = part.rect.height * scale;

        svg.push_str(&format!(
            r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{color}" fill-opacity="0.7" stroke="#333" stroke-width="0.5"/>"##
        ));

        let font_size = (w.min(h) * 0.15).clamp(8.0, 14.0);
        svg.push_str(&format!(
            r##"<text x="{}" y="{}" font-size="{font_size}" fill="#222" text-anchor="middle" dominant-baseline="central" font-family="sans-serif">{}</text>"##,
            x + w / 2.0, y + h / 2.0, part.id,
        ));
    }

    svg.push_str("</svg>");
    Ok(axum::response::Html(svg))
}

/// Request body for validate_placement.
#[derive(Deserialize)]
pub struct ValidatePlacementRequest {
    pub placements: Vec<ManualPlacement>,
    pub config: NestingConfig,
}

/// POST /api/sessions/:id/nesting/validate — Validate a manual placement.
pub async fn validate_placement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ValidatePlacementRequest>,
) -> Result<Json<PlacementValidation>, ServerError> {
    let _session = get_session(&state, &id)?;
    let result = validate_manual_placement(&req.placements, &req.config);
    Ok(Json(result))
}
