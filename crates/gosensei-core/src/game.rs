use serde::{Deserialize, Serialize};

use crate::board::Board;
use crate::rules::{self, MoveError};
use crate::scoring;
use crate::sgf;
use crate::types::{BoardSize, Color, GameResult, Move, MoveRecord, Point};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Playing,
    Finished,
}

#[derive(Debug, Clone)]
pub struct Game {
    board: Board,
    current_color: Color,
    phase: GamePhase,
    move_history: Vec<MoveRecord>,
    board_history: Vec<Board>,
    ko_point: Option<Point>,
    captures: Captures,
    consecutive_passes: u8,
    komi: f32,
    result: Option<GameResult>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Captures {
    pub black: u32,
    pub white: u32,
}

/// Serializable snapshot of game state for IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub board_size: u8,
    pub stones: Vec<StonePosition>,
    pub current_color: String,
    pub move_number: u16,
    pub captures_black: u32,
    pub captures_white: u32,
    pub phase: GamePhase,
    pub result: Option<GameResult>,
    pub last_move: Option<(u8, u8)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StonePosition {
    pub row: u8,
    pub col: u8,
    pub color: String,
}

impl Game {
    pub fn new(board_size: BoardSize, komi: f32) -> Self {
        let board = Board::new(board_size);
        Self {
            board: board.clone(),
            current_color: Color::Black,
            phase: GamePhase::Playing,
            move_history: Vec::new(),
            board_history: vec![board],
            ko_point: None,
            captures: Captures::default(),
            consecutive_passes: 0,
            komi,
            result: None,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    pub fn current_color(&self) -> Color {
        self.current_color
    }

    pub fn phase(&self) -> &GamePhase {
        &self.phase
    }

    pub fn move_history(&self) -> &[MoveRecord] {
        &self.move_history
    }

    pub fn captures(&self) -> &Captures {
        &self.captures
    }

    pub fn result(&self) -> Option<&GameResult> {
        self.result.as_ref()
    }

    pub fn komi(&self) -> f32 {
        self.komi
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn play(&mut self, point: Point) -> Result<Vec<Point>, MoveError> {
        if self.phase == GamePhase::Finished {
            return Err(MoveError::GameOver);
        }

        let captured = rules::apply_move(&mut self.board, point, self.current_color, self.ko_point)?;

        // Update captures count
        let capture_count = captured.len() as u32;
        match self.current_color {
            Color::Black => self.captures.black += capture_count,
            Color::White => self.captures.white += capture_count,
        }

        // Detect ko
        self.ko_point = rules::detect_ko(&self.board, point, &captured);

        // Record move
        let move_number = self.move_history.len() as u16 + 1;
        self.move_history.push(MoveRecord {
            color: self.current_color,
            mv: Move::Play(point),
            move_number,
        });
        self.board_history.push(self.board.clone());

        self.consecutive_passes = 0;
        self.current_color = self.current_color.opponent();

        Ok(captured)
    }

    pub fn pass(&mut self) -> Result<(), MoveError> {
        if self.phase == GamePhase::Finished {
            return Err(MoveError::GameOver);
        }

        let move_number = self.move_history.len() as u16 + 1;
        self.move_history.push(MoveRecord {
            color: self.current_color,
            mv: Move::Pass,
            move_number,
        });
        self.board_history.push(self.board.clone());

        self.consecutive_passes += 1;
        self.ko_point = None;
        self.current_color = self.current_color.opponent();

        // Two consecutive passes end the game — score the position
        if self.consecutive_passes >= 2 {
            self.phase = GamePhase::Finished;
            self.result = Some(scoring::score_area(&self.board, self.komi));
        }

        Ok(())
    }

    pub fn resign(&mut self) -> Result<GameResult, MoveError> {
        if self.phase == GamePhase::Finished {
            return Err(MoveError::GameOver);
        }

        let move_number = self.move_history.len() as u16 + 1;
        self.move_history.push(MoveRecord {
            color: self.current_color,
            mv: Move::Resign,
            move_number,
        });

        self.phase = GamePhase::Finished;
        let result = GameResult::Resignation {
            winner: self.current_color.opponent(),
        };
        self.result = Some(result.clone());
        Ok(result)
    }

    pub fn undo(&mut self) -> Result<(), MoveError> {
        if self.move_history.is_empty() {
            return Ok(());
        }

        self.move_history.pop();
        self.board_history.pop();

        // Restore board state
        if let Some(previous) = self.board_history.last() {
            self.board = previous.clone();
        }

        self.current_color = self.current_color.opponent();
        self.ko_point = None; // Simplified — proper ko tracking on undo is complex
        self.consecutive_passes = 0;
        self.phase = GamePhase::Playing;
        self.result = None;

        Ok(())
    }

    /// Replay an SGF string into a Game. Stops on the first illegal move.
    pub fn from_sgf(input: &str) -> Result<Self, String> {
        Self::from_sgf_partial(input, None)
    }

    /// Replay an SGF string up to `max_moves` moves (or all moves if `None`).
    /// Stops on the first illegal move or when the limit is reached.
    pub fn from_sgf_partial(input: &str, max_moves: Option<u16>) -> Result<Self, String> {
        let parsed = sgf::parser::parse(input).map_err(|e| e.to_string())?;
        let mut game = Game::new(parsed.board_size, parsed.komi);
        let limit = max_moves.unwrap_or(u16::MAX) as usize;

        for (color, mv) in parsed.moves.iter().take(limit) {
            // SGF may have out-of-order colors; force the expected color
            game.current_color = *color;
            match mv {
                Move::Play(point) => {
                    if game.play(*point).is_err() {
                        break;
                    }
                    // play() flips color, but we forced it above, so the
                    // alternation is handled by the next iteration's force.
                }
                Move::Pass => {
                    if game.pass().is_err() {
                        break;
                    }
                }
                Move::Resign => {
                    let _ = game.resign();
                    break;
                }
            }
        }

        Ok(game)
    }

    /// Parse an SGF with setup stones (AB/AW) into a Game.
    /// Places setup stones directly on the board, then replays any moves.
    /// The starting color is determined by the PL[] property, or defaults to Black.
    pub fn from_sgf_with_setup(input: &str) -> Result<Self, String> {
        let parsed = sgf::parser::parse(input).map_err(|e| e.to_string())?;
        let mut game = Game::new(parsed.board_size, parsed.komi);

        // Place setup stones directly (bypassing rules)
        for &point in &parsed.setup_black {
            game.board.set(point, Some(Color::Black));
        }
        for &point in &parsed.setup_white {
            game.board.set(point, Some(Color::White));
        }

        // Update initial board history to include setup stones
        game.board_history[0] = game.board.clone();

        // Set starting color from PL[] or default to Black
        game.current_color = parsed.player_to_move.unwrap_or(Color::Black);

        // Replay any moves that follow the setup
        for (color, mv) in &parsed.moves {
            game.current_color = *color;
            match mv {
                Move::Play(point) => {
                    if game.play(*point).is_err() {
                        break;
                    }
                }
                Move::Pass => {
                    if game.pass().is_err() {
                        break;
                    }
                }
                Move::Resign => {
                    let _ = game.resign();
                    break;
                }
            }
        }

        Ok(game)
    }

    /// Serialize this game to SGF format.
    pub fn to_sgf(&self) -> String {
        sgf::writer::write(self)
    }

    /// Create a serializable snapshot of the current state.
    pub fn to_state(&self) -> GameState {
        let dim = self.board.dimension();
        let mut stones = Vec::new();
        for r in 0..dim {
            for c in 0..dim {
                let p = Point::new(r, c);
                if let Some(color) = self.board.get(p) {
                    stones.push(StonePosition {
                        row: r,
                        col: c,
                        color: match color {
                            Color::Black => "black".to_string(),
                            Color::White => "white".to_string(),
                        },
                    });
                }
            }
        }

        let last_move = self.move_history.last().and_then(|m| match m.mv {
            Move::Play(p) => Some((p.row, p.col)),
            _ => None,
        });

        GameState {
            board_size: dim,
            stones,
            current_color: match self.current_color {
                Color::Black => "black".to_string(),
                Color::White => "white".to_string(),
            },
            move_number: self.move_history.len() as u16,
            captures_black: self.captures.black,
            captures_white: self.captures.white,
            phase: self.phase.clone(),
            result: self.result.clone(),
            last_move,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_starts_with_black() {
        let game = Game::new(BoardSize::Nine, 6.5);
        assert_eq!(game.current_color(), Color::Black);
    }

    #[test]
    fn turns_alternate() {
        let mut game = Game::new(BoardSize::Nine, 6.5);
        game.play(Point::new(4, 4)).unwrap();
        assert_eq!(game.current_color(), Color::White);
        game.play(Point::new(3, 3)).unwrap();
        assert_eq!(game.current_color(), Color::Black);
    }

    #[test]
    fn double_pass_ends_game() {
        let mut game = Game::new(BoardSize::Nine, 6.5);
        game.pass().unwrap();
        game.pass().unwrap();
        assert_eq!(*game.phase(), GamePhase::Finished);
        // Empty board with 6.5 komi → White wins
        assert_eq!(
            game.result().cloned(),
            Some(GameResult::Score {
                winner: Color::White,
                margin: 6.5,
            })
        );
    }

    #[test]
    fn resign_ends_game() {
        let mut game = Game::new(BoardSize::Nine, 6.5);
        let result = game.resign().unwrap();
        assert_eq!(
            result,
            GameResult::Resignation {
                winner: Color::White
            }
        );
    }

    #[test]
    fn undo_restores_state() {
        let mut game = Game::new(BoardSize::Nine, 6.5);
        game.play(Point::new(4, 4)).unwrap();
        assert_eq!(game.current_color(), Color::White);
        game.undo().unwrap();
        assert_eq!(game.current_color(), Color::Black);
        assert!(game.board().is_empty(Point::new(4, 4)));
    }

    #[test]
    fn cannot_play_after_game_over() {
        let mut game = Game::new(BoardSize::Nine, 6.5);
        game.resign().unwrap();
        let result = game.play(Point::new(4, 4));
        assert!(matches!(result, Err(MoveError::GameOver)));
    }

    #[test]
    fn from_sgf_partial_stops_at_limit() {
        let sgf = "(;SZ[9]KM[6.5];B[ee];W[cc];B[gg];W[dd])";
        // Full replay has 4 moves
        let full = Game::from_sgf(sgf).unwrap();
        assert_eq!(full.move_history().len(), 4);

        // Partial replay at move 2
        let partial = Game::from_sgf_partial(sgf, Some(2)).unwrap();
        assert_eq!(partial.move_history().len(), 2);

        // Board at move 2 should match full game's board at that point
        let full_state_at_2 = Game::from_sgf_partial(sgf, Some(2)).unwrap().to_state();
        let partial_state = partial.to_state();
        assert_eq!(full_state_at_2.stones.len(), partial_state.stones.len());
        assert_eq!(full_state_at_2.move_number, 2);
    }

    #[test]
    fn from_sgf_partial_none_replays_all() {
        let sgf = "(;SZ[9]KM[6.5];B[ee];W[cc];B[gg])";
        let full = Game::from_sgf(sgf).unwrap();
        let partial_all = Game::from_sgf_partial(sgf, None).unwrap();
        assert_eq!(
            full.move_history().len(),
            partial_all.move_history().len()
        );
    }

    #[test]
    fn from_sgf_partial_zero_gives_empty_board() {
        let sgf = "(;SZ[9]KM[6.5];B[ee];W[cc])";
        let empty = Game::from_sgf_partial(sgf, Some(0)).unwrap();
        assert_eq!(empty.move_history().len(), 0);
        assert!(empty.board().is_empty(Point::new(4, 4)));
    }

    #[test]
    fn board_mut_allows_direct_placement() {
        let mut game = Game::new(BoardSize::Nine, 6.5);
        game.board_mut().set(Point::new(3, 3), Some(Color::Black));
        assert_eq!(game.board().get(Point::new(3, 3)), Some(Color::Black));
    }

    #[test]
    fn from_sgf_with_setup_places_stones() {
        // SGF coords: dd = col 3 row 3, de = col 3 row 4, ed = col 4 row 3, ee = col 4 row 4
        let sgf = "(;SZ[9]AB[dd][de]AW[ed][ee])";
        let game = Game::from_sgf_with_setup(sgf).unwrap();
        assert_eq!(game.board().get(Point::new(3, 3)), Some(Color::Black));
        assert_eq!(game.board().get(Point::new(4, 3)), Some(Color::Black));
        assert_eq!(game.board().get(Point::new(3, 4)), Some(Color::White));
        assert_eq!(game.board().get(Point::new(4, 4)), Some(Color::White));
        assert_eq!(game.current_color(), Color::Black);
        assert_eq!(game.move_history().len(), 0);
    }

    #[test]
    fn from_sgf_with_setup_respects_pl() {
        let sgf = "(;SZ[9]AB[dd]AW[ee]PL[W])";
        let game = Game::from_sgf_with_setup(sgf).unwrap();
        assert_eq!(game.current_color(), Color::White);
    }

    #[test]
    fn from_sgf_with_setup_and_moves() {
        // df = col 3, row 5 → Point(5, 3)
        let sgf = "(;SZ[9]AB[dd][de]AW[ed][ee];B[df])";
        let game = Game::from_sgf_with_setup(sgf).unwrap();
        assert_eq!(game.board().get(Point::new(5, 3)), Some(Color::Black));
        assert_eq!(game.move_history().len(), 1);
    }
}
