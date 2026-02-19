use std::sync::Mutex;

use cm_cabinet::project::{Project, TaggedPart};
use cm_post::MachineProfile;

/// Application state shared across all Tauri commands.
///
/// Each field is wrapped in a `Mutex` so commands can read/write independently
/// without holding a lock on the entire state.
pub struct AppState {
    /// The current project (None if no project is loaded).
    pub project: Mutex<Option<Project>>,

    /// File path of the currently open project (None if unsaved/new).
    pub project_path: Mutex<Option<String>>,

    /// Active machine profile for validation and G-code generation.
    pub machine: Mutex<MachineProfile>,

    /// Cached generated parts (invalidated when the project changes).
    pub cached_parts: Mutex<Option<Vec<TaggedPart>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            project: Mutex::new(None),
            project_path: Mutex::new(None),
            machine: Mutex::new(MachineProfile::tormach_pcnc1100()),
            cached_parts: Mutex::new(None),
        }
    }
}
