use crate::types::ErrorClass;

/// Heuristic error classification based on KataGo analysis data.
///
/// Alpha implementation: simple heuristics based on move position and game phase.
/// These will be refined with real game data during Alpha-2.
pub fn classify_error(
    move_number: u16,
    board_size: u8,
    row: u8,
    col: u8,
    _score_loss: f64,
) -> Option<ErrorClass> {
    let game_phase = game_phase(move_number, board_size);

    match game_phase {
        GamePhase::Opening => Some(ErrorClass::Opening),
        GamePhase::Endgame => Some(ErrorClass::Endgame),
        GamePhase::Middle => {
            // Basic heuristic: edge moves are often about life & death
            if is_edge(row, col, board_size) {
                Some(ErrorClass::LifeAndDeath)
            } else {
                Some(ErrorClass::Direction)
            }
        }
    }
}

enum GamePhase {
    Opening,
    Middle,
    Endgame,
}

fn game_phase(move_number: u16, board_size: u8) -> GamePhase {
    let total_intersections = board_size as u16 * board_size as u16;
    let ratio = move_number as f32 / total_intersections as f32;

    if ratio < 0.15 {
        GamePhase::Opening
    } else if ratio > 0.6 {
        GamePhase::Endgame
    } else {
        GamePhase::Middle
    }
}

fn is_edge(row: u8, col: u8, board_size: u8) -> bool {
    row <= 1 || col <= 1 || row >= board_size - 2 || col >= board_size - 2
}
