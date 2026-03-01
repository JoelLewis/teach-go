use crate::board::Board;
use crate::types::{Color, Point};

#[derive(Debug, Clone, thiserror::Error)]
pub enum MoveError {
    #[error("point ({row}, {col}) is occupied")]
    Occupied { row: u8, col: u8 },

    #[error("point ({row}, {col}) is out of bounds")]
    OutOfBounds { row: u8, col: u8 },

    #[error("move is suicidal (no liberties after placement)")]
    Suicide,

    #[error("move violates ko rule (recreates previous board position)")]
    Ko,

    #[error("game is already over")]
    GameOver,
}

/// Check if placing `color` at `point` on `board` is legal.
/// `ko_point` is the single point forbidden by simple ko.
/// Returns the set of captured stones if the move is legal.
pub fn validate_move(
    board: &Board,
    point: Point,
    color: Color,
    ko_point: Option<Point>,
) -> Result<Vec<Point>, MoveError> {
    let dim = board.dimension();
    if point.row >= dim || point.col >= dim {
        return Err(MoveError::OutOfBounds {
            row: point.row,
            col: point.col,
        });
    }

    if !board.is_empty(point) {
        return Err(MoveError::Occupied {
            row: point.row,
            col: point.col,
        });
    }

    if ko_point == Some(point) {
        return Err(MoveError::Ko);
    }

    // Simulate placement to check for captures and suicide
    let mut test_board = board.clone();
    test_board.set(point, Some(color));

    // Check for captures of opponent groups
    let opponent = color.opponent();
    let mut captured = Vec::new();
    for neighbor in point.neighbors(dim) {
        if test_board.get(neighbor) == Some(opponent) {
            if let Some(group) = test_board.group_at(neighbor) {
                if group.is_captured() {
                    for &stone in &group.stones {
                        captured.push(stone);
                    }
                    test_board.remove_group(&group);
                }
            }
        }
    }

    // Check for suicide (no captures and placed stone has no liberties)
    if captured.is_empty() {
        if let Some(group) = test_board.group_at(point) {
            if group.is_captured() {
                return Err(MoveError::Suicide);
            }
        }
    }

    Ok(captured)
}

/// Apply a move to the board, returning captured stones.
pub fn apply_move(
    board: &mut Board,
    point: Point,
    color: Color,
    ko_point: Option<Point>,
) -> Result<Vec<Point>, MoveError> {
    let captured = validate_move(board, point, color, ko_point)?;

    board.set(point, Some(color));

    // Remove captured stones
    for &cap in &captured {
        board.set(cap, None);
    }

    Ok(captured)
}

/// Determine the ko point after a move (if exactly one stone was captured
/// and the placed stone has exactly one liberty remaining).
pub fn detect_ko(board: &Board, point: Point, captured: &[Point]) -> Option<Point> {
    if captured.len() != 1 {
        return None;
    }
    let group = board.group_at(point)?;
    if group.stones.len() == 1 && group.liberty_count() == 1 {
        Some(captured[0])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BoardSize;

    #[test]
    fn reject_occupied_point() {
        let mut board = Board::new(BoardSize::Nine);
        let p = Point::new(4, 4);
        board.set(p, Some(Color::Black));
        let result = validate_move(&board, p, Color::White, None);
        assert!(matches!(result, Err(MoveError::Occupied { .. })));
    }

    #[test]
    fn simple_capture() {
        // Place white stone surrounded by black on three sides
        // Then black plays the fourth side to capture
        let mut board = Board::new(BoardSize::Nine);
        let target = Point::new(0, 1);
        board.set(target, Some(Color::White));
        board.set(Point::new(0, 0), Some(Color::Black));
        board.set(Point::new(1, 1), Some(Color::Black));

        let captured = apply_move(&mut board, Point::new(0, 2), Color::Black, None).unwrap();
        assert_eq!(captured.len(), 1);
        assert!(captured.contains(&target));
        assert!(board.is_empty(target));
    }

    #[test]
    fn reject_suicide() {
        let mut board = Board::new(BoardSize::Nine);
        // Surround a corner point with opponent stones
        board.set(Point::new(0, 1), Some(Color::White));
        board.set(Point::new(1, 0), Some(Color::White));
        let result = validate_move(&board, Point::new(0, 0), Color::Black, None);
        assert!(matches!(result, Err(MoveError::Suicide)));
    }

    #[test]
    fn ko_detection() {
        // Classic ko shape:
        //   0 1 2
        // 0 . B .
        // 1 B W B
        // 2 . B .
        // White at (1,1) has 0 liberties when Black plays at any remaining spot.
        // But for ko we need: capture exactly 1 stone, capturer has 1 liberty.
        //
        // Proper ko setup:
        //   0 1 2
        // 0 B . W
        // 1 . B W
        // 2 B W .
        // Black captures white at (0,1)... no, let's use a direct unit test.

        // Direct test of detect_ko: single stone captured, capturer has 1 liberty
        let mut board = Board::new(BoardSize::Nine);
        //   0 1
        // 0 B .
        // 1 . .
        // Place black at corner — it has 2 liberties, so ko won't trigger
        board.set(Point::new(0, 0), Some(Color::Black));
        let ko = detect_ko(&board, Point::new(0, 0), &[Point::new(0, 1)]);
        assert!(ko.is_none(), "2 liberties means no ko");

        // Set up a position where the capturer has exactly 1 liberty
        let mut board = Board::new(BoardSize::Nine);
        //   0 1 2
        // 0 W B .
        // 1 B . .
        // Black captures white at (0,0): Black stone at (0,0) has 1 liberty (0,1 occupied by B)
        // Wait, (0,1) already has Black so (0,0) after capture would have liberty at...
        // Let's just set it up after the capture:
        //   0 1
        // 0 B W    <- Black just played at (0,0), captured the stone that was there
        // 1 W .    <- White surrounds, so Black at (0,0) has 1 liberty: (0,1)...
        //           but (0,1) is W. Hmm.

        // Simplest ko: Black stone with 1 liberty after capturing 1 stone
        board.set(Point::new(0, 0), Some(Color::Black));
        board.set(Point::new(1, 0), Some(Color::White)); // below
        // (0,0) now has 1 liberty: (0,1). Captured 1 stone -> ko!
        let ko = detect_ko(&board, Point::new(0, 0), &[Point::new(0, 1)]);
        // Stone at (0,0) has 1 liberty (only (0,1) is free since (1,0) is white)
        // Wait: neighbors of (0,0) are (0,1) and (1,0). (1,0) is White, (0,1) is empty -> 1 liberty
        assert_eq!(ko, Some(Point::new(0, 1)));
    }

    #[test]
    fn ko_point_prevents_recapture() {
        let board = Board::new(BoardSize::Nine);
        let p = Point::new(2, 2);
        // Point is empty and legal, but marked as ko
        let result = validate_move(&board, p, Color::Black, Some(p));
        assert!(matches!(result, Err(MoveError::Ko)));
    }
}
