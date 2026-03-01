use std::sync::{Arc, Mutex};

use gosensei_core::game::Game;
use gosensei_core::types::Color;
use gosensei_katago::client::KataGoClient;

pub struct AppState {
    pub game: Mutex<Option<Game>>,
    pub ai_color: Mutex<Option<Color>>,
    pub katago: Arc<tokio::sync::Mutex<Option<KataGoClient>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            game: Mutex::new(None),
            ai_color: Mutex::new(None),
            katago: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
