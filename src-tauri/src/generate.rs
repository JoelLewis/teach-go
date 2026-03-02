use gosensei_core::game::Game;
use gosensei_core::types::{BoardSize, Color, Point};

use crate::convert::gtp_to_point;
use crate::problem::{
    points_to_setup_sgf, Problem, ProblemCategory, ProblemSource, ResponseBranch, SolutionNode,
};
use crate::review::MoveAnalysis;

/// Generate problems from positions where the player made mistakes.
pub fn generate_from_review(
    game_sgf: &str,
    board_size: u8,
    analyses: &[MoveAnalysis],
    game_id: i64,
    threshold: f64,
) -> Result<Vec<Problem>, String> {
    let bs = BoardSize::try_from(board_size).map_err(|_| "invalid board size")?;
    let mut problems = Vec::new();

    for analysis in analyses {
        // Skip positions without significant errors
        if analysis.score_loss < threshold {
            continue;
        }
        // Skip pass moves and initial position
        if analysis.player_move.is_none() || analysis.move_number == 0 {
            continue;
        }
        if let Some(ref mv) = analysis.player_move
            && mv.eq_ignore_ascii_case("pass")
        {
            continue;
        }
        // Need best_move to build a solution
        let best_move_gtp = match &analysis.best_move {
            Some(m) => m.clone(),
            None => continue,
        };

        // Reconstruct the board at the position BEFORE the mistake
        let position = analysis.move_number.saturating_sub(1);
        let game = match Game::from_sgf_partial(game_sgf, Some(position)) {
            Ok(g) => g,
            Err(_) => continue,
        };

        // Convert board state to setup SGF
        let setup_sgf = board_to_setup_sgf(&game, board_size);

        // Build solution from the engine's best variation
        let solutions = build_solution_from_pv(
            &best_move_gtp,
            &analysis.best_variation,
            board_size,
        );

        if solutions.is_empty() {
            continue;
        }

        // Determine category from error classification
        let category = analysis
            .coaching_message
            .as_ref()
            .and_then(|_| error_class_from_severity(analysis.score_loss))
            .unwrap_or(ProblemCategory::Tesuji);

        // Player color at that position
        let player_color = if position.is_multiple_of(2) {
            Color::Black
        } else {
            Color::White
        };

        let prompt = if player_color == Color::Black {
            "Black to find the best move"
        } else {
            "White to find the best move"
        };

        problems.push(Problem {
            id: 0,
            setup_sgf,
            board_size: bs,
            player_color,
            solutions,
            category,
            difficulty: 20.0, // Will be refined by player's skill level
            source: ProblemSource::Generated,
            source_game_id: Some(game_id),
            prompt: prompt.into(),
            tags: vec!["from-game".into()],
        });
    }

    Ok(problems)
}

/// Convert a game's board state to SGF with AB/AW setup properties.
pub fn board_to_setup_sgf(game: &Game, board_size: u8) -> String {
    let board = game.board();
    let mut black_stones = Vec::new();
    let mut white_stones = Vec::new();

    for point in board.all_points() {
        match board.get(point) {
            Some(Color::Black) => black_stones.push(point),
            Some(Color::White) => white_stones.push(point),
            None => {}
        }
    }

    points_to_setup_sgf(board_size, &black_stones, &white_stones)
}

/// Build a solution tree from the engine's best move and principal variation.
/// The PV alternates player/opponent moves starting from the best move.
pub fn build_solution_from_pv(
    first_move_gtp: &str,
    pv: &[String],
    board_size: u8,
) -> Vec<SolutionNode> {
    let first_point = match gtp_to_point(first_move_gtp, board_size) {
        Some(p) => p,
        None => return vec![],
    };

    // Build PV points (skip the first move since we handle it separately)
    let pv_points: Vec<Option<Point>> = pv.iter().map(|m| gtp_to_point(m, board_size)).collect();

    // Recursively build the tree from alternating moves, depth limited to 7
    let node = build_tree_recursive(first_point, &pv_points, 0, 7);
    vec![node]
}

fn build_tree_recursive(
    player_move: Point,
    remaining_pv: &[Option<Point>],
    depth: usize,
    max_depth: usize,
) -> SolutionNode {
    if depth >= max_depth || remaining_pv.is_empty() {
        return SolutionNode {
            point: player_move,
            responses: vec![],
        };
    }

    // Next move in PV is opponent's response
    let opponent_point = match remaining_pv.first() {
        Some(Some(p)) => *p,
        _ => {
            return SolutionNode {
                point: player_move,
                responses: vec![],
            };
        }
    };

    // Move after that is player's next correct move
    if remaining_pv.len() < 2 {
        // Opponent responds but no more player moves needed
        return SolutionNode {
            point: player_move,
            responses: vec![ResponseBranch {
                opponent_move: opponent_point,
                correct_moves: vec![],
            }],
        };
    }

    let next_player_point = match remaining_pv.get(1) {
        Some(Some(p)) => *p,
        _ => {
            return SolutionNode {
                point: player_move,
                responses: vec![ResponseBranch {
                    opponent_move: opponent_point,
                    correct_moves: vec![],
                }],
            };
        }
    };

    // Recurse for deeper PV
    let child = build_tree_recursive(next_player_point, &remaining_pv[2..], depth + 1, max_depth);

    SolutionNode {
        point: player_move,
        responses: vec![ResponseBranch {
            opponent_move: opponent_point,
            correct_moves: vec![child],
        }],
    }
}

fn error_class_from_severity(score_loss: f64) -> Option<ProblemCategory> {
    // Default categorization based on score loss magnitude
    if score_loss >= 10.0 {
        Some(ProblemCategory::LifeDeath)
    } else if score_loss >= 5.0 {
        Some(ProblemCategory::Tesuji)
    } else {
        Some(ProblemCategory::Shape)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_to_setup_sgf_produces_valid_sgf() {
        let sgf = "(;SZ[9];B[ee];W[dd])";
        let game = Game::from_sgf(sgf).unwrap();
        let setup = board_to_setup_sgf(&game, 9);

        assert!(setup.starts_with("(;SZ[9]"));
        assert!(setup.contains("AB"));
        assert!(setup.contains("AW"));

        // Should parse back correctly
        let parsed = gosensei_core::sgf::parser::parse(&setup).unwrap();
        assert_eq!(parsed.setup_black.len(), 1);
        assert_eq!(parsed.setup_white.len(), 1);
    }

    #[test]
    fn build_solution_from_pv_one_move() {
        let solutions = build_solution_from_pv("E5", &[], 9);
        assert_eq!(solutions.len(), 1);
        assert_eq!(solutions[0].point, Point::new(4, 4));
        assert!(solutions[0].responses.is_empty());
    }

    #[test]
    fn build_solution_from_pv_three_moves() {
        let pv = vec!["D5".to_string(), "E6".to_string()];
        let solutions = build_solution_from_pv("E5", &pv, 9);
        assert_eq!(solutions.len(), 1);

        let node = &solutions[0];
        assert_eq!(node.point, Point::new(4, 4)); // E5
        assert_eq!(node.responses.len(), 1);

        let branch = &node.responses[0];
        assert_eq!(branch.opponent_move, Point::new(4, 3)); // D5
        assert_eq!(branch.correct_moves.len(), 1);
        assert_eq!(branch.correct_moves[0].point, Point::new(3, 4)); // E6
    }

    #[test]
    fn build_solution_from_pv_five_moves() {
        let pv = vec![
            "D5".to_string(),
            "E6".to_string(),
            "D6".to_string(),
            "E7".to_string(),
        ];
        let solutions = build_solution_from_pv("E5", &pv, 9);
        let node = &solutions[0];
        // Depth 0: player E5, opponent D5
        assert_eq!(node.responses[0].opponent_move, Point::new(4, 3));
        // Depth 1: player E6, opponent D6
        let next = &node.responses[0].correct_moves[0];
        assert_eq!(next.point, Point::new(3, 4));
        assert_eq!(next.responses[0].opponent_move, Point::new(3, 3));
        // Depth 2: player E7
        let final_node = &next.responses[0].correct_moves[0];
        assert_eq!(final_node.point, Point::new(2, 4));
    }

    #[test]
    fn generate_from_review_filters_by_threshold() {
        use gosensei_coaching::types::Severity;

        let sgf = "(;SZ[9]KM[6.5];B[ee];W[dd];B[cc];W[ff])";
        let analyses = vec![
            MoveAnalysis {
                move_number: 1,
                color: Some("black".into()),
                player_move: Some("E5".into()),
                winrate_black: 0.5,
                score_lead: 0.0,
                best_move: Some("D5".into()),
                score_loss: 2.0, // Below threshold
                severity: Severity::Inaccuracy,
                coaching_message: None,
                best_variation: vec![],
            },
            MoveAnalysis {
                move_number: 3,
                color: Some("black".into()),
                player_move: Some("C7".into()),
                winrate_black: 0.4,
                score_lead: -2.0,
                best_move: Some("F5".into()),
                score_loss: 5.0, // Above threshold
                severity: Severity::Mistake,
                coaching_message: Some("Shape error".into()),
                best_variation: vec!["E4".to_string()],
            },
        ];

        let problems = generate_from_review(sgf, 9, &analyses, 1, 3.0).unwrap();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].source_game_id == Some(1));
    }

    #[test]
    fn generate_from_review_skips_pass_moves() {
        use gosensei_coaching::types::Severity;

        let sgf = "(;SZ[9]KM[6.5];B[ee];W[])";
        let analyses = vec![MoveAnalysis {
            move_number: 2,
            color: Some("white".into()),
            player_move: Some("pass".into()),
            winrate_black: 0.5,
            score_lead: 0.0,
            best_move: Some("D5".into()),
            score_loss: 10.0,
            severity: Severity::Blunder,
            coaching_message: None,
            best_variation: vec![],
        }];

        let problems = generate_from_review(sgf, 9, &analyses, 1, 3.0).unwrap();
        assert_eq!(problems.len(), 0);
    }

    #[test]
    fn generated_problem_sgf_loads_correctly() {
        let sgf = "(;SZ[9]KM[6.5];B[ee];W[dd];B[cc])";
        let game = Game::from_sgf_partial(sgf, Some(2)).unwrap();
        let setup = board_to_setup_sgf(&game, 9);

        // Should load via from_sgf_with_setup
        let loaded = Game::from_sgf_with_setup(&setup).unwrap();
        // Board should have 2 stones
        let state = loaded.to_state();
        assert_eq!(state.stones.len(), 2);
    }
}
