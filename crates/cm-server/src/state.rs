use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use cm_cabinet::project::{Project, TaggedPart};
use cm_post::machine::MachineProfile;
use uuid::Uuid;

/// Per-session state (supports multiple browser tabs/users).
pub struct SessionState {
    pub project: Mutex<Option<Project>>,
    pub project_filename: Mutex<Option<String>>,
    pub machine: Mutex<MachineProfile>,
    pub cached_parts: Mutex<Option<Vec<TaggedPart>>>,
    pub last_activity: Mutex<Instant>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            project: Mutex::new(None),
            project_filename: Mutex::new(None),
            machine: Mutex::new(MachineProfile::tormach_pcnc1100()),
            cached_parts: Mutex::new(None),
            last_activity: Mutex::new(Instant::now()),
        }
    }

    /// Touch the session to update last activity time.
    pub fn touch(&self) {
        if let Ok(mut last) = self.last_activity.lock() {
            *last = Instant::now();
        }
    }
}

/// Global application state shared across all requests.
pub struct AppState {
    pub sessions: RwLock<HashMap<Uuid, Arc<SessionState>>>,
    pub templates_dir: PathBuf,
    pub session_timeout: Duration,
}

impl AppState {
    pub fn new(templates_dir: PathBuf) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            templates_dir,
            session_timeout: Duration::from_secs(30 * 60), // 30 minutes
        }
    }

    /// Get a session by ID, returning an error if not found.
    pub fn get_session(&self, id: &Uuid) -> Option<Arc<SessionState>> {
        let sessions = self.sessions.read().ok()?;
        let session = sessions.get(id)?.clone();
        session.touch();
        Some(session)
    }

    /// Create a new session and return its ID.
    pub fn create_session(&self) -> Option<(Uuid, Arc<SessionState>)> {
        let id = Uuid::new_v4();
        let session = Arc::new(SessionState::new());
        let mut sessions = self.sessions.write().ok()?;
        sessions.insert(id, session.clone());
        Some((id, session))
    }

    /// Remove a session.
    pub fn remove_session(&self, id: &Uuid) -> bool {
        if let Ok(mut sessions) = self.sessions.write() {
            sessions.remove(id).is_some()
        } else {
            false
        }
    }

    /// Sweep idle sessions older than the timeout.
    pub fn sweep_idle_sessions(&self) {
        if let Ok(mut sessions) = self.sessions.write() {
            let timeout = self.session_timeout;
            sessions.retain(|_id, session| {
                if let Ok(last) = session.last_activity.lock() {
                    last.elapsed() < timeout
                } else {
                    false
                }
            });
        }
    }
}
