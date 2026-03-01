use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub fn opponent(self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub row: u8,
    pub col: u8,
}

impl Point {
    pub fn new(row: u8, col: u8) -> Self {
        Self { row, col }
    }

    pub fn neighbors(self, board_size: u8) -> impl Iterator<Item = Point> {
        let (r, c, s) = (self.row as i8, self.col as i8, board_size as i8);
        [(r - 1, c), (r + 1, c), (r, c - 1), (r, c + 1)]
            .into_iter()
            .filter(move |&(nr, nc)| nr >= 0 && nc >= 0 && nr < s && nc < s)
            .map(|(nr, nc)| Point::new(nr as u8, nc as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Move {
    Play(Point),
    Pass,
    Resign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardSize {
    Nine,
    Thirteen,
    Nineteen,
}

impl BoardSize {
    pub fn size(self) -> u8 {
        match self {
            BoardSize::Nine => 9,
            BoardSize::Thirteen => 13,
            BoardSize::Nineteen => 19,
        }
    }
}

impl TryFrom<u8> for BoardSize {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            9 => Ok(BoardSize::Nine),
            13 => Ok(BoardSize::Thirteen),
            19 => Ok(BoardSize::Nineteen),
            _ => Err(format!("Invalid board size: {value}. Must be 9, 13, or 19")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameResult {
    Score { winner: Color, margin: f32 },
    Resignation { winner: Color },
    Draw,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoveRecord {
    pub color: Color,
    pub mv: Move,
    pub move_number: u16,
}
