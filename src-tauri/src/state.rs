use std::sync::{Arc, Mutex};

use gosensei_core::game::Game;
use gosensei_core::types::Color;
use gosensei_katago::client::KataGoClient;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::review::ReviewSession;
use crate::skill::GameError;
use crate::solver::SolverSession;

/// Which opponent drives AI moves for the active game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum AiEngine {
    #[serde(rename = "katago")]
    KataGo,
    PracticeBot,
}

/// Engine choice is made once when a game starts and stays fixed for that
/// game (no mid-game engine swap): KataGo when its download is complete,
/// otherwise the built-in practice bot.
pub fn select_engine(katago_ready: bool) -> AiEngine {
    if katago_ready {
        AiEngine::KataGo
    } else {
        AiEngine::PracticeBot
    }
}

pub struct AppState {
    pub game: Mutex<Option<Game>>,
    pub ai_color: Mutex<Option<Color>>,
    pub ai_engine: Mutex<AiEngine>,
    pub bot_seed: Mutex<u64>,
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
            ai_engine: Mutex::new(AiEngine::KataGo),
            bot_seed: Mutex::new(0),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn katago_ready_selects_katago() {
        assert_eq!(select_engine(true), AiEngine::KataGo);
    }

    #[test]
    fn katago_absent_selects_practice_bot() {
        assert_eq!(select_engine(false), AiEngine::PracticeBot);
    }
}
