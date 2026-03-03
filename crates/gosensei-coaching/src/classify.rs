use crate::types::ErrorClass;

/// Input data for error classification.
pub struct ClassifyInput<'a> {
    pub move_number: u16,
    pub board_size: u8,
    pub player_row: u8,
    pub player_col: u8,
    pub best_row: u8,
    pub best_col: u8,
    pub pv_length: usize,
    pub ownership: Option<&'a [f64]>,
    pub score_loss: f64,
}

/// Richer error classification using KataGo analysis data.
///
/// Runs a 6-classifier cascade:
/// 1. Direction — player and best move far apart or different quadrants
/// 2. Life & Death — ownership flips near the move
/// 3. Reading — PV depth diverges significantly
/// 4. Shape — player move near best move but suboptimal local shape
/// 5. Sente/Gote — best move forces a local response
/// 6. Fallback — game-phase heuristic
pub fn classify_error(input: &ClassifyInput<'_>) -> Option<ErrorClass> {
    let phase = game_phase(input.move_number, input.board_size);

    // 1. Direction: player and best move are far apart (>7 intersections or different quadrants)
    let distance = manhattan_distance(
        input.player_row,
        input.player_col,
        input.best_row,
        input.best_col,
    );
    if distance > 7
        || different_quadrants(
            input.player_row,
            input.player_col,
            input.best_row,
            input.best_col,
            input.board_size,
        )
    {
        return Some(ErrorClass::Direction);
    }

    // 2. Life & Death: ownership map shows territory flipping near the move
    if input.ownership.is_some_and(|own| {
        ownership_flips_near(own, input.player_row, input.player_col, input.board_size)
    }) {
        return Some(ErrorClass::LifeAndDeath);
    }

    // 3. Reading: PV depth diverges significantly (deep tactical sequence)
    if input.pv_length > 6 && input.score_loss > 3.0 {
        return Some(ErrorClass::Reading);
    }

    // 4. Shape: player move within 3 intersections of best — local shape error
    if distance <= 3 && distance > 0 {
        if is_edge(input.player_row, input.player_col, input.board_size)
            && !is_edge(input.best_row, input.best_col, input.board_size)
        {
            return Some(ErrorClass::Shape);
        }
        if input.score_loss >= 1.0 {
            return Some(ErrorClass::Shape);
        }
    }

    // 5. Sente/Gote: short PV with moderate loss suggests timing/initiative error
    if input.pv_length <= 3 && input.score_loss >= 2.0 && phase == GamePhase::Middle {
        return Some(ErrorClass::SenteGote);
    }

    // 6. Fallback: game-phase heuristic
    match phase {
        GamePhase::Opening => Some(ErrorClass::Opening),
        GamePhase::Endgame => Some(ErrorClass::Endgame),
        GamePhase::Middle => {
            if is_edge(input.player_row, input.player_col, input.board_size) {
                Some(ErrorClass::LifeAndDeath)
            } else {
                Some(ErrorClass::Direction)
            }
        }
    }
}

/// Legacy signature for backward compatibility with callers that don't have
/// best-move or PV data yet. Delegates to the richer classifier with
/// the player's own coordinates as "best" (so distance = 0 → falls through
/// to phase heuristic).
pub fn classify_error_simple(
    move_number: u16,
    board_size: u8,
    row: u8,
    col: u8,
    score_loss: f64,
) -> Option<ErrorClass> {
    classify_error(&ClassifyInput {
        move_number,
        board_size,
        player_row: row,
        player_col: col,
        best_row: row,
        best_col: col,
        pv_length: 0,
        ownership: None,
        score_loss,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

fn manhattan_distance(r1: u8, c1: u8, r2: u8, c2: u8) -> u8 {
    r1.abs_diff(r2) + c1.abs_diff(c2)
}

fn different_quadrants(r1: u8, c1: u8, r2: u8, c2: u8, board_size: u8) -> bool {
    let mid = board_size / 2;
    let q1 = (r1 < mid, c1 < mid);
    let q2 = (r2 < mid, c2 < mid);
    q1 != q2
}

/// Check if ownership values near a move show a sign flip (territory contested).
/// Looks at a 5x5 neighborhood around the move; if there are significant positive
/// AND negative ownership values, the area is contested → life & death.
fn ownership_flips_near(ownership: &[f64], row: u8, col: u8, board_size: u8) -> bool {
    let bs = board_size as i16;
    let r = row as i16;
    let c = col as i16;

    let mut has_positive = false;
    let mut has_negative = false;
    let threshold = 0.3;

    for dr in -2..=2_i16 {
        for dc in -2..=2_i16 {
            let nr = r + dr;
            let nc = c + dc;
            if nr >= 0 && nr < bs && nc >= 0 && nc < bs {
                let idx = nr as usize * board_size as usize + nc as usize;
                if idx < ownership.len() {
                    let val = ownership[idx];
                    if val > threshold {
                        has_positive = true;
                    }
                    if val < -threshold {
                        has_negative = true;
                    }
                }
            }
        }
    }

    has_positive && has_negative
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! ci {
        ($mn:expr, $bs:expr, $pr:expr, $pc:expr, $br:expr, $bc:expr, $pv:expr, $own:expr, $loss:expr) => {
            ClassifyInput {
                move_number: $mn,
                board_size: $bs,
                player_row: $pr,
                player_col: $pc,
                best_row: $br,
                best_col: $bc,
                pv_length: $pv,
                ownership: $own,
                score_loss: $loss,
            }
        };
    }

    #[test]
    fn direction_far_apart_moves() {
        let result = classify_error(&ci!(50, 19, 0, 0, 15, 15, 5, None, 5.0));
        assert_eq!(result, Some(ErrorClass::Direction));
    }

    #[test]
    fn direction_different_quadrants() {
        let result = classify_error(&ci!(50, 19, 3, 3, 15, 15, 3, None, 3.0));
        assert_eq!(result, Some(ErrorClass::Direction));
    }

    #[test]
    fn shape_close_moves_with_loss() {
        let result = classify_error(&ci!(50, 19, 4, 3, 3, 3, 3, None, 2.0));
        assert_eq!(result, Some(ErrorClass::Shape));
    }

    #[test]
    fn reading_deep_pv_high_loss() {
        let result = classify_error(&ci!(50, 19, 4, 3, 5, 4, 10, None, 5.0));
        assert_eq!(result, Some(ErrorClass::Reading));
    }

    #[test]
    fn life_and_death_ownership_flip() {
        let mut ownership = vec![0.0; 81];
        ownership[3 * 9 + 3] = 0.6;
        ownership[3 * 9 + 4] = 0.5;
        ownership[5 * 9 + 4] = -0.5;
        ownership[5 * 9 + 5] = -0.4;

        let result = classify_error(&ci!(30, 9, 4, 4, 4, 5, 4, Some(&ownership), 4.0));
        assert_eq!(result, Some(ErrorClass::LifeAndDeath));
    }

    #[test]
    fn sente_gote_short_pv_moderate_loss() {
        let result = classify_error(&ci!(80, 19, 10, 14, 10, 10, 2, None, 3.0));
        assert_eq!(result, Some(ErrorClass::SenteGote));
    }

    #[test]
    fn opening_phase_fallback() {
        let result = classify_error(&ci!(5, 19, 3, 3, 3, 3, 0, None, 1.0));
        assert_eq!(result, Some(ErrorClass::Opening));
    }

    #[test]
    fn endgame_phase_fallback() {
        let result = classify_error(&ci!(250, 19, 3, 3, 3, 3, 0, None, 0.5));
        assert_eq!(result, Some(ErrorClass::Endgame));
    }

    #[test]
    fn edge_middle_game_life_and_death() {
        let result = classify_error(&ci!(80, 19, 0, 5, 0, 5, 0, None, 1.0));
        assert_eq!(result, Some(ErrorClass::LifeAndDeath));
    }

    #[test]
    fn simple_legacy_api_works() {
        let result = classify_error_simple(50, 19, 9, 9, 3.0);
        assert!(result.is_some());
    }

    #[test]
    fn ownership_no_flip_not_life_and_death() {
        let ownership = vec![0.5; 81];
        let result = classify_error(&ci!(50, 9, 4, 4, 4, 5, 3, Some(&ownership), 2.0));
        assert_eq!(result, Some(ErrorClass::Shape));
    }

    #[test]
    fn empty_ownership_slice_handled() {
        let result = classify_error(&ci!(50, 9, 4, 4, 4, 5, 3, Some(&[]), 2.0));
        assert_eq!(result, Some(ErrorClass::Shape));
    }
}
