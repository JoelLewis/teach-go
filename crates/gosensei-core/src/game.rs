use serde::{Deserialize, Serialize};

use crate::board::Board;
use crate::rules::{self, MoveError};
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

        // Two consecutive passes end the game
        if self.consecutive_passes >= 2 {
            self.phase = GamePhase::Finished;
            self.result = Some(GameResult::Draw); // Simplified — real scoring TBD
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
}
