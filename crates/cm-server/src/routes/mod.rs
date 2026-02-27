pub mod cabinet;
pub mod estimate;
pub mod export;
pub mod gcode;
pub mod machine;
pub mod nesting;
pub mod project;
pub mod session;

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post, put};

use crate::state::AppState;

/// Build the complete API router.
pub fn api_router() -> Router<Arc<AppState>> {
    let session_routes = Router::new()
        // Project
        .route("/project/new", post(project::new_project))
        .route("/project", get(project::get_project).put(project::update_project))
        .route("/project/open", post(project::open_project))
        .route("/project/save", post(project::save_project))
        .route("/project/template/{*name}", post(project::load_template))
        // Cabinets
        .route("/parts", post(cabinet::generate_parts))
        .route("/cabinets", post(cabinet::add_cabinet))
        .route("/cabinets/{index}", put(cabinet::update_cabinet).delete(cabinet::remove_cabinet))
        .route("/cabinets/preview", post(cabinet::preview_cabinet_parts))
        .route("/cabinets/{index}/3d", get(cabinet::get_3d_assembly))
        // Nesting
        .route("/nesting", post(nesting::nest_all))
        .route("/nesting/validate", post(nesting::validate_placement))
        .route("/nesting/{material_index}/{sheet_index}/svg", get(nesting::get_nesting_svg))
        // G-code
        .route("/gcode/validate", post(gcode::validate_project))
        .route("/gcode/generate", post(gcode::generate_gcode))
        .route("/gcode/{material_index}/{sheet_index}", get(gcode::preview_gcode))
        .route("/gcode/{material_index}/{sheet_index}/toolpaths", get(gcode::get_toolpaths))
        // Machine
        .route("/machine", get(machine::get_machine).put(machine::set_machine))
        .route("/machine/upload", post(machine::upload_machine_profile))
        // Export
        .route("/export/cutlist", get(export::get_cutlist))
        .route("/export/csv", get(export::export_csv))
        .route("/export/bom", get(export::export_bom))
        // Estimate
        .route("/estimate", get(estimate::get_estimate))
        // DXF export
        .route("/export/dxf", get(export::export_dxf));

    Router::new()
        .route("/api/sessions", post(session::create_session))
        .route("/api/sessions/{id}", get(session::get_session_info).delete(session::delete_session))
        .nest("/api/sessions/{id}", session_routes)
        .route("/api/templates", get(project::list_templates))
}
