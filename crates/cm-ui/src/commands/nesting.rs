use serde::{Deserialize, Serialize};
use tauri::State;

use cm_cabinet::project::Project;
use cm_nesting::packer::{nest_parts, NestingConfig, NestingPart, NestingResult};

use crate::state::AppState;

/// Owned material group DTO (no lifetimes) for sending to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialGroupDto {
    pub material_name: String,
    pub thickness: f64,
    pub nesting_result: NestingResult,
}

/// Nest all parts grouped by material.
#[tauri::command]
pub fn nest_all(
    config: Option<NestingConfig>,
    state: State<'_, AppState>,
) -> Result<Vec<MaterialGroupDto>, String> {
    let guard = state.project.lock().map_err(|e| e.to_string())?;
    let project = guard.as_ref().ok_or("No project loaded")?;

    // Generate parts if not cached
    let parts_guard = state.cached_parts.lock().map_err(|e| e.to_string())?;
    let tagged = if let Some(ref cached) = *parts_guard {
        cached.clone()
    } else {
        drop(parts_guard);
        let tagged = project.generate_all_parts();
        *state.cached_parts.lock().map_err(|e| e.to_string())? = Some(tagged.clone());
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

/// Generate SVG for a specific sheet layout.
#[tauri::command]
pub fn get_nesting_svg(
    material_index: usize,
    sheet_index: usize,
    config: Option<NestingConfig>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let groups = nest_all(config, state)?;

    let group = groups.get(material_index)
        .ok_or(format!("Material index {material_index} out of range"))?;
    let sheet = group.nesting_result.sheets.get(sheet_index)
        .ok_or(format!("Sheet index {sheet_index} out of range"))?;

    let sw = sheet.sheet_rect.width;
    let sh = sheet.sheet_rect.height;
    let scale = 8.0; // pixels per inch

    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"##,
        sw * scale,
        sh * scale,
        sw * scale,
        sh * scale,
    );

    // Sheet background
    svg.push_str(&format!(
        r##"<rect x="0" y="0" width="{}" height="{}" fill="#f5f0e8" stroke="#999" stroke-width="1"/>"##,
        sw * scale,
        sh * scale,
    ));

    // Colors for different parts
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

        // Label
        let font_size = (w.min(h) * 0.15).max(8.0).min(14.0);
        svg.push_str(&format!(
            r##"<text x="{}" y="{}" font-size="{font_size}" fill="#222" text-anchor="middle" dominant-baseline="central" font-family="sans-serif">{}</text>"##,
            x + w / 2.0,
            y + h / 2.0,
            part.id,
        ));
    }

    svg.push_str("</svg>");
    Ok(svg)
}
