use std::sync::Mutex;

use gosensei_core::game::Game;

pub struct AppState {
    pub game: Mutex<Option<Game>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            game: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
