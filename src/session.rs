use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, Duration};

use lazy_static::lazy_static;

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub data: HashMap<String, String>,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
}

pub struct SessionManager {
    pub sessions: HashMap<String, Session>,
    pub session_duration: Duration,
}

lazy_static! {
    static ref GLOBAL_SESSION_MANAGER: Mutex<SessionManager> = Mutex::new(SessionManager::new(Duration::from_secs(60 * 60)));
}

impl SessionManager {

    pub fn global() -> &'static Mutex<SessionManager> {
        &GLOBAL_SESSION_MANAGER
    }

    pub fn new(session_duration: Duration) -> Self {
        Self {
            sessions: HashMap::new(),
            session_duration,
        }
    }

    pub fn create_session(&mut self) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let session = Session {
            id: id.clone(),
            data: HashMap::new(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + self.session_duration,
        };
        self.sessions.insert(id.clone(), session);
        id
    }

    pub fn get_session(&self, id: &str) -> Option<&Session> {
        self.sessions.get(id)
    }

    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(id)
    }

    pub fn remove_session(&mut self, id: &str) {
        self.sessions.remove(id);
    }
}