use std::fs;

use tauri::State;

use cm_post::MachineProfile;

use crate::state::AppState;

/// Get the current machine profile.
#[tauri::command]
pub fn get_machine(state: State<'_, AppState>) -> Result<MachineProfile, String> {
    let guard = state.machine.lock().map_err(|e| e.to_string())?;
    Ok(guard.clone())
}

/// Set the machine profile.
#[tauri::command]
pub fn set_machine(profile: MachineProfile, state: State<'_, AppState>) -> Result<(), String> {
    *state.machine.lock().map_err(|e| e.to_string())? = profile;
    Ok(())
}

/// Load a machine profile from a TOML file.
#[tauri::command]
pub fn load_machine_profile(path: String, state: State<'_, AppState>) -> Result<MachineProfile, String> {
    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {e}"))?;
    let profile = MachineProfile::from_toml(&contents)
        .map_err(|e| format!("Failed to parse machine profile: {e}"))?;

    *state.machine.lock().map_err(|e| e.to_string())? = profile.clone();

    Ok(profile)
}
