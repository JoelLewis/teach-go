use gosensei_coaching::types::Severity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveAnalysis {
    pub move_number: u16,
    /// "black" | "white", None for initial position (move 0)
    pub color: Option<String>,
    /// GTP notation of the played move (None for initial position or pass)
    pub player_move: Option<String>,
    /// Win probability from Black's perspective, 0.0–1.0
    pub winrate_black: f64,
    /// Score lead from Black's perspective (positive = Black leads)
    pub score_lead: f64,
    /// Engine's top choice in GTP notation
    pub best_move: Option<String>,
    /// Score loss in points (0.0 if best move or initial position)
    pub score_loss: f64,
    pub severity: Severity,
    pub coaching_message: Option<String>,
    /// Principal variation from the engine
    pub best_variation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewData {
    pub board_size: u8,
    pub total_moves: u16,
    pub komi: f32,
    pub move_analyses: Vec<MoveAnalysis>,
    /// Up to 5 move numbers with the highest score_loss, sorted descending
    pub top_mistakes: Vec<u16>,
}

/// In-memory session tracking review progress.
pub struct ReviewSession {
    pub game_sgf: String,
    pub board_size: u8,
    pub komi: f32,
    pub total_positions: u16,
    pub results: Vec<Option<MoveAnalysis>>,
    pub is_complete: bool,
}
