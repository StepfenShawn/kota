/// Session-aware hook placeholder retained for API compatibility.
#[derive(Clone)]
pub struct SessionIdHook {
    pub session_id: String,
}

impl SessionIdHook {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}
