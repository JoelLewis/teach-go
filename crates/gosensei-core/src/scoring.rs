use std::collections::HashSet;

use crate::board::Board;
use crate::types::{Color, GameResult, Point};

/// Tromp-Taylor area scoring.
///
/// Each point on the board is assigned to the color that exclusively
/// reaches it through empty intersections, or left neutral if both
/// colors (or neither) can reach it.
pub fn score_area(board: &Board, komi: f32) -> GameResult {
    let dim = board.dimension();
    let mut visited: HashSet<Point> = HashSet::new();
    let mut black_score: f32 = 0.0;
    let mut white_score: f32 = komi;

    for point in board.all_points() {
        match board.get(point) {
            Some(Color::Black) => black_score += 1.0,
            Some(Color::White) => white_score += 1.0,
            None if !visited.contains(&point) => {
                let (region, owner) = flood_empty(board, point, dim);
                visited.extend(&region);
                let territory = region.len() as f32;
                match owner {
                    Some(Color::Black) => black_score += territory,
                    Some(Color::White) => white_score += territory,
                    None => {} // dame — no points
                }
            }
            _ => {}
        }
    }

    if black_score > white_score {
        GameResult::Score {
            winner: Color::Black,
            margin: black_score - white_score,
        }
    } else if white_score > black_score {
        GameResult::Score {
            winner: Color::White,
            margin: white_score - black_score,
        }
    } else {
        GameResult::Draw
    }
}

/// Flood-fill from an empty point. Returns the set of connected empty
/// points and the unique neighboring color (if exactly one color
/// borders the region).
fn flood_empty(board: &Board, start: Point, dim: u8) -> (HashSet<Point>, Option<Color>) {
    let mut region = HashSet::new();
    let mut stack = vec![start];
    let mut bordering_colors: HashSet<Color> = HashSet::new();

    while let Some(p) = stack.pop() {
        if !region.insert(p) {
            continue;
        }
        for neighbor in p.neighbors(dim) {
            match board.get(neighbor) {
                None => {
                    if !region.contains(&neighbor) {
                        stack.push(neighbor);
                    }
                }
                Some(color) => {
                    bordering_colors.insert(color);
                }
            }
        }
    }

    let owner = if bordering_colors.len() == 1 {
        bordering_colors.into_iter().next()
    } else {
        None
    };

    (region, owner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BoardSize;

    #[test]
    fn empty_board_white_wins_by_komi() {
        let board = Board::new(BoardSize::Nine);
        // Empty board: all 81 points are dame (touch neither color).
        // White gets komi only.
        // Actually, an empty board has no stones bordering empty regions,
        // so all points are neutral. Black=0, White=6.5.
        let result = score_area(&board, 6.5);
        assert_eq!(
            result,
            GameResult::Score {
                winner: Color::White,
                margin: 6.5,
            }
        );
    }

    #[test]
    fn single_stone_no_territory() {
        // A lone black stone on a 9x9 board: the surrounding empty area
        // touches only black, so black claims the entire empty region.
        // Black = 1 (stone) + 80 (territory) = 81, White = 6.5 (komi)
        let mut board = Board::new(BoardSize::Nine);
        board.set(Point::new(4, 4), Some(Color::Black));
        let result = score_area(&board, 6.5);
        assert_eq!(
            result,
            GameResult::Score {
                winner: Color::Black,
                margin: 81.0 - 6.5,
            }
        );
    }

    #[test]
    fn surrounded_territory() {
        // On a 9x9 board, white surrounds the top-left corner.
        // White stones at (0,1), (1,0), (1,1) — the point (0,0) is
        // empty and only touches white → territory for white.
        let mut board = Board::new(BoardSize::Nine);
        board.set(Point::new(0, 1), Some(Color::White));
        board.set(Point::new(1, 0), Some(Color::White));
        board.set(Point::new(1, 1), Some(Color::White));
        // Also place a black stone so the rest of the board isn't all-white territory.
        board.set(Point::new(2, 0), Some(Color::Black));
        board.set(Point::new(0, 2), Some(Color::Black));

        let result = score_area(&board, 0.0);
        // (0,0) is surrounded by white → white territory
        // Remaining empty points touch both colors → dame
        // Actually let's verify: (0,0) neighbors are (0,1)=White, (1,0)=White
        // So (0,0) region = {(0,0)}, owner = White.
        // Rest of empty board: connected region touching both colors → dame.
        // Black: 2 stones + 0 territory = 2
        // White: 3 stones + 1 territory = 4
        // With komi=0: White wins by 2.0
        assert_eq!(
            result,
            GameResult::Score {
                winner: Color::White,
                margin: 2.0,
            }
        );
    }

    #[test]
    fn dame_scores_for_nobody() {
        // Black at (0,0), White at (0,2) on a 9x9 board.
        // The empty points between them touch both colors → dame.
        let mut board = Board::new(BoardSize::Nine);
        board.set(Point::new(0, 0), Some(Color::Black));
        board.set(Point::new(0, 2), Some(Color::White));

        // All empty points are connected and touch both colors → all dame.
        // Black = 1, White = 1 + 6.5 komi = 7.5
        let result = score_area(&board, 6.5);
        assert_eq!(
            result,
            GameResult::Score {
                winner: Color::White,
                margin: 6.5,
            }
        );
    }

    #[test]
    fn black_surrounds_large_territory() {
        // Construct a scenario where black surrounds a 2x2 interior.
        // On a 9x9 board, black forms a ring around (3,3)-(4,4).
        let mut board = Board::new(BoardSize::Nine);

        // Black wall around rows 2-5, cols 2-5 (the border of a 4x4 square)
        for r in 2..=5 {
            for c in 2..=5 {
                if r == 2 || r == 5 || c == 2 || c == 5 {
                    board.set(Point::new(r, c), Some(Color::Black));
                }
            }
        }
        // Place a white stone outside so exterior isn't all-black territory
        board.set(Point::new(0, 0), Some(Color::White));

        let result = score_area(&board, 6.5);
        // Interior empty: (3,3), (3,4), (4,3), (4,4) = 4 points, surrounded by black.
        // Exterior empty: connected, touches both colors → dame.
        // Black stones: 12 (the ring). White stones: 1.
        // Black = 12 + 4 = 16. White = 1 + 6.5 = 7.5. Black wins by 8.5.
        assert_eq!(
            result,
            GameResult::Score {
                winner: Color::Black,
                margin: 8.5,
            }
        );
    }

    #[test]
    fn exact_tie_is_draw() {
        // Contrive a position where scores are exactly equal.
        // On 9x9 with komi=0: place 1 black and 1 white stone, rest is dame.
        let mut board = Board::new(BoardSize::Nine);
        board.set(Point::new(0, 0), Some(Color::Black));
        board.set(Point::new(0, 8), Some(Color::White));

        let result = score_area(&board, 0.0);
        // All empty points touch both colors → dame.
        // Black = 1, White = 1 → Draw.
        assert_eq!(result, GameResult::Draw);
    }
}
