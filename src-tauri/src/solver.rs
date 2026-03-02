use std::time::Instant;

use gosensei_core::game::Game;
use gosensei_core::types::Point;
use serde::{Deserialize, Serialize};

use crate::problem::{Problem, SolutionNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolveStatus {
    InProgress,
    Solved,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MoveResult {
    Correct {
        opponent_response: Option<(u8, u8)>,
        solved: bool,
    },
    Wrong {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HintLevel {
    Area,
    Candidates,
    Answer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HintData {
    None,
    Area {
        min_row: u8,
        max_row: u8,
        min_col: u8,
        max_col: u8,
    },
    Candidates {
        points: Vec<(u8, u8)>,
    },
    Answer {
        point: Option<(u8, u8)>,
        message: String,
    },
}

pub struct SolverSession {
    problem_id: i64,
    game: Game,
    status: SolveStatus,
    hints_used: u8,
    attempts: u16,
    start_time: Instant,
    current_branches: Vec<SolutionNode>,
    _player_color: gosensei_core::types::Color,
    prompt: String,
    category: String,
}

impl SolverSession {
    pub fn new(problem: &Problem) -> Result<Self, String> {
        let game = Game::from_sgf_with_setup(&problem.setup_sgf)?;

        Ok(Self {
            problem_id: problem.id,
            game,
            status: SolveStatus::InProgress,
            hints_used: 0,
            attempts: 0,
            start_time: Instant::now(),
            current_branches: problem.solutions.clone(),
            _player_color: problem.player_color,
            prompt: problem.prompt.clone(),
            category: problem.category.as_str().to_string(),
        })
    }

    pub fn problem_id(&self) -> i64 {
        self.problem_id
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn status(&self) -> SolveStatus {
        self.status
    }

    pub fn hints_used(&self) -> u8 {
        self.hints_used
    }

    pub fn attempts(&self) -> u16 {
        self.attempts
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn elapsed_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    pub fn try_move(&mut self, point: Point) -> MoveResult {
        if self.status != SolveStatus::InProgress {
            return MoveResult::Wrong {
                message: "Problem is already finished".into(),
            };
        }

        self.attempts += 1;

        // Check if this point matches any correct move in current branches
        let matched = self
            .current_branches
            .iter()
            .find(|node| node.point == point)
            .cloned();

        match matched {
            Some(node) => {
                // Play the correct move on the board
                let _ = self.game.play(point);

                if node.responses.is_empty() {
                    // Terminal node — problem solved!
                    self.status = SolveStatus::Solved;
                    MoveResult::Correct {
                        opponent_response: None,
                        solved: true,
                    }
                } else {
                    // Pick the first response branch (opponent has one main response)
                    let branch = &node.responses[0];
                    let opp_point = branch.opponent_move;

                    // Play opponent's response
                    let _ = self.game.play(opp_point);

                    // Advance to next level of the tree
                    self.current_branches = branch.correct_moves.clone();

                    if self.current_branches.is_empty() {
                        // No more moves needed — solved
                        self.status = SolveStatus::Solved;
                        MoveResult::Correct {
                            opponent_response: Some((opp_point.row, opp_point.col)),
                            solved: true,
                        }
                    } else {
                        MoveResult::Correct {
                            opponent_response: Some((opp_point.row, opp_point.col)),
                            solved: false,
                        }
                    }
                }
            }
            None => MoveResult::Wrong {
                message: "That's not the best move. Try again!".into(),
            },
        }
    }

    pub fn get_hint(&mut self, level: HintLevel) -> HintData {
        self.hints_used += 1;

        if self.current_branches.is_empty() {
            return HintData::None;
        }

        match level {
            HintLevel::Area => {
                // Give a 5×5 area around the first correct answer
                let p = self.current_branches[0].point;
                let dim = self.game.board().dimension();
                HintData::Area {
                    min_row: p.row.saturating_sub(2),
                    max_row: (p.row + 2).min(dim - 1),
                    min_col: p.col.saturating_sub(2),
                    max_col: (p.col + 2).min(dim - 1),
                }
            }
            HintLevel::Candidates => {
                let points: Vec<(u8, u8)> = self
                    .current_branches
                    .iter()
                    .take(3)
                    .map(|n| (n.point.row, n.point.col))
                    .collect();
                HintData::Candidates { points }
            }
            HintLevel::Answer => {
                let first = &self.current_branches[0];
                HintData::Answer {
                    point: Some((first.point.row, first.point.col)),
                    message: "The correct move is shown".into(),
                }
            }
        }
    }

    pub fn mark_failed(&mut self) {
        self.status = SolveStatus::Failed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::*;
    use gosensei_core::types::{BoardSize, Color};

    fn simple_one_move_problem() -> Problem {
        Problem {
            id: 1,
            setup_sgf: points_to_setup_sgf(
                9,
                &[Point::new(3, 4), Point::new(4, 3), Point::new(5, 4)],
                &[Point::new(4, 4)],
            ),
            board_size: BoardSize::Nine,
            player_color: Color::Black,
            solutions: vec![SolutionNode {
                point: Point::new(4, 5),
                responses: vec![],
            }],
            category: ProblemCategory::Tesuji,
            difficulty: 25.0,
            source: ProblemSource::Seed,
            source_game_id: None,
            prompt: "Black to capture".into(),
            tags: vec![],
        }
    }

    fn two_move_problem() -> Problem {
        Problem {
            id: 2,
            setup_sgf: points_to_setup_sgf(
                9,
                &[Point::new(3, 4), Point::new(4, 3)],
                &[Point::new(4, 4)],
            ),
            board_size: BoardSize::Nine,
            player_color: Color::Black,
            solutions: vec![SolutionNode {
                point: Point::new(5, 4),
                responses: vec![ResponseBranch {
                    opponent_move: Point::new(4, 5),
                    correct_moves: vec![SolutionNode {
                        point: Point::new(5, 5),
                        responses: vec![],
                    }],
                }],
            }],
            category: ProblemCategory::Tesuji,
            difficulty: 24.0,
            source: ProblemSource::Seed,
            source_game_id: None,
            prompt: "Black to capture".into(),
            tags: vec![],
        }
    }

    fn branching_problem() -> Problem {
        Problem {
            id: 3,
            setup_sgf: points_to_setup_sgf(
                9,
                &[Point::new(3, 4), Point::new(4, 3)],
                &[Point::new(4, 4)],
            ),
            board_size: BoardSize::Nine,
            player_color: Color::Black,
            solutions: vec![
                SolutionNode {
                    point: Point::new(5, 4),
                    responses: vec![],
                },
                SolutionNode {
                    point: Point::new(4, 5),
                    responses: vec![],
                },
            ],
            category: ProblemCategory::Tesuji,
            difficulty: 24.0,
            source: ProblemSource::Seed,
            source_game_id: None,
            prompt: "Black to capture".into(),
            tags: vec![],
        }
    }

    #[test]
    fn correct_one_move_solves() {
        let problem = simple_one_move_problem();
        let mut solver = SolverSession::new(&problem).unwrap();
        assert_eq!(solver.status(), SolveStatus::InProgress);

        let result = solver.try_move(Point::new(4, 5));
        assert!(matches!(result, MoveResult::Correct { solved: true, .. }));
        assert_eq!(solver.status(), SolveStatus::Solved);
    }

    #[test]
    fn wrong_move_returns_wrong() {
        let problem = simple_one_move_problem();
        let mut solver = SolverSession::new(&problem).unwrap();

        let result = solver.try_move(Point::new(0, 0));
        assert!(matches!(result, MoveResult::Wrong { .. }));
        assert_eq!(solver.status(), SolveStatus::InProgress);
        assert_eq!(solver.attempts(), 1);
    }

    #[test]
    fn two_move_sequence() {
        let problem = two_move_problem();
        let mut solver = SolverSession::new(&problem).unwrap();

        // First correct move
        let result = solver.try_move(Point::new(5, 4));
        match result {
            MoveResult::Correct {
                opponent_response,
                solved,
            } => {
                assert!(!solved);
                assert_eq!(opponent_response, Some((4, 5)));
            }
            _ => panic!("expected Correct"),
        }
        assert_eq!(solver.status(), SolveStatus::InProgress);

        // Second correct move
        let result = solver.try_move(Point::new(5, 5));
        assert!(matches!(result, MoveResult::Correct { solved: true, .. }));
        assert_eq!(solver.status(), SolveStatus::Solved);
    }

    #[test]
    fn branching_solutions_accept_either() {
        let problem = branching_problem();
        let mut solver = SolverSession::new(&problem).unwrap();

        // Either branch should work
        let result = solver.try_move(Point::new(4, 5));
        assert!(matches!(result, MoveResult::Correct { solved: true, .. }));
    }

    #[test]
    fn hint_area_gives_bounded_region() {
        let problem = simple_one_move_problem();
        let mut solver = SolverSession::new(&problem).unwrap();

        let hint = solver.get_hint(HintLevel::Area);
        match hint {
            HintData::Area {
                min_row,
                max_row,
                min_col,
                max_col,
            } => {
                // Answer is (4,5), area should be roughly 2-6, 3-7
                assert!(min_row <= 4 && max_row >= 4);
                assert!(min_col <= 5 && max_col >= 5);
            }
            _ => panic!("expected Area hint"),
        }
        assert_eq!(solver.hints_used(), 1);
    }

    #[test]
    fn hint_candidates_gives_points() {
        let problem = branching_problem();
        let mut solver = SolverSession::new(&problem).unwrap();

        let hint = solver.get_hint(HintLevel::Candidates);
        match hint {
            HintData::Candidates { points } => {
                assert_eq!(points.len(), 2);
            }
            _ => panic!("expected Candidates hint"),
        }
    }

    #[test]
    fn hint_answer_reveals_move() {
        let problem = simple_one_move_problem();
        let mut solver = SolverSession::new(&problem).unwrap();

        let hint = solver.get_hint(HintLevel::Answer);
        match hint {
            HintData::Answer { point, .. } => {
                assert_eq!(point, Some((4, 5)));
            }
            _ => panic!("expected Answer hint"),
        }
    }

    #[test]
    fn hints_used_counter_increments() {
        let problem = simple_one_move_problem();
        let mut solver = SolverSession::new(&problem).unwrap();
        assert_eq!(solver.hints_used(), 0);
        solver.get_hint(HintLevel::Area);
        assert_eq!(solver.hints_used(), 1);
        solver.get_hint(HintLevel::Candidates);
        assert_eq!(solver.hints_used(), 2);
    }
}
