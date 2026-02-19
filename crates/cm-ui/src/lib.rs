pub mod state;
pub mod commands;

use state::AppState;

/// Build and configure the Tauri application.
///
/// This is separated from `main.rs` so it can be tested and reused.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            // Project
            commands::project::new_project,
            commands::project::get_project,
            commands::project::update_project,
            commands::project::open_project,
            commands::project::save_project,
            commands::project::load_template,
            commands::project::list_templates,
            commands::project::get_project_path,
            // Cabinet
            commands::cabinet::generate_parts,
            commands::cabinet::add_cabinet,
            commands::cabinet::update_cabinet,
            commands::cabinet::remove_cabinet,
            commands::cabinet::preview_cabinet_parts,
            commands::cabinet::get_3d_assembly,
            // Nesting
            commands::nesting::nest_all,
            commands::nesting::get_nesting_svg,
            // G-code
            commands::gcode::validate_project_cmd,
            commands::gcode::generate_gcode,
            commands::gcode::preview_gcode,
            // Machine
            commands::machine::get_machine,
            commands::machine::set_machine,
            commands::machine::load_machine_profile,
            // Export
            commands::export::get_cutlist,
            commands::export::export_csv,
            commands::export::export_bom_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
