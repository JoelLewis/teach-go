use std::sync::{Arc, Mutex};

use gosensei_core::game::Game;
use gosensei_core::types::Color;
use gosensei_katago::client::KataGoClient;
use rusqlite::Connection;

use crate::review::ReviewSession;
use crate::skill::GameError;
use crate::solver::SolverSession;

pub struct AppState {
    pub game: Mutex<Option<Game>>,
    pub ai_color: Mutex<Option<Color>>,
    pub katago: Arc<tokio::sync::Mutex<Option<KataGoClient>>>,
    pub db: Mutex<Connection>,
    pub review: Arc<tokio::sync::Mutex<Option<ReviewSession>>>,
    pub game_errors: Mutex<Vec<GameError>>,
    pub solver: Mutex<Option<SolverSession>>,
    #[cfg(feature = "llm")]
    pub llm: Arc<tokio::sync::Mutex<Option<gosensei_llm::model::ModelManager>>>,
}

impl AppState {
    pub fn with_db(conn: Connection) -> Self {
        Self {
            game: Mutex::new(None),
            ai_color: Mutex::new(None),
            katago: Arc::new(tokio::sync::Mutex::new(None)),
            db: Mutex::new(conn),
            review: Arc::new(tokio::sync::Mutex::new(None)),
            game_errors: Mutex::new(Vec::new()),
            solver: Mutex::new(None),
            #[cfg(feature = "llm")]
            llm: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }
}
