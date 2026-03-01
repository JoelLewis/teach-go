use gosensei_core::types::{Color, Move, MoveRecord, Point};
use gosensei_katago::protocol::AnalysisQuery;

/// GTP column letters: A–T, skipping I.
const GTP_COLUMNS: [char; 25] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

/// Convert a Point (0-indexed, top-left origin) to a GTP coordinate string.
///
/// GTP columns are A–T (skipping I), rows count from the bottom (1-indexed).
/// On a 9×9 board: Point(0,0) → "A9", Point(8,8) → "J1".
pub fn point_to_gtp(point: Point, board_size: u8) -> String {
    let col_letter = GTP_COLUMNS[point.col as usize];
    let row_number = board_size - point.row;
    format!("{col_letter}{row_number}")
}

/// Convert a GTP coordinate string to a Point. Returns None for "pass".
pub fn gtp_to_point(gtp: &str, board_size: u8) -> Option<Point> {
    let gtp = gtp.trim();
    if gtp.eq_ignore_ascii_case("pass") {
        return None;
    }

    let mut chars = gtp.chars();
    let col_char = chars.next()?.to_ascii_uppercase();
    let row_str: String = chars.collect();
    let row_number: u8 = row_str.parse().ok()?;

    let col = GTP_COLUMNS.iter().position(|&c| c == col_char)? as u8;
    if col >= board_size || row_number == 0 || row_number > board_size {
        return None;
    }

    let row = board_size - row_number;
    Some(Point::new(row, col))
}

fn color_to_gtp(color: Color) -> String {
    match color {
        Color::Black => "B".to_string(),
        Color::White => "W".to_string(),
    }
}

/// Convert game move history to KataGo's moves format: `[("B","D4"), ("W","Q16")]`.
/// Skips Resign moves.
pub fn history_to_katago_moves(history: &[MoveRecord], board_size: u8) -> Vec<(String, String)> {
    history
        .iter()
        .filter_map(|record| match record.mv {
            Move::Play(point) => {
                Some((color_to_gtp(record.color), point_to_gtp(point, board_size)))
            }
            Move::Pass => Some((color_to_gtp(record.color), "pass".to_string())),
            Move::Resign => None,
        })
        .collect()
}

/// Build a full KataGo AnalysisQuery from game state.
pub fn build_query(
    id: String,
    history: &[MoveRecord],
    board_size: u8,
    komi: f32,
    max_visits: u32,
) -> AnalysisQuery {
    AnalysisQuery {
        id,
        moves: history_to_katago_moves(history, board_size),
        rules: "tromp-taylor".to_string(),
        komi,
        board_x_size: board_size,
        board_y_size: board_size,
        max_visits,
        include_ownership: None,
        include_policy: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_to_gtp_top_left() {
        assert_eq!(point_to_gtp(Point::new(0, 0), 9), "A9");
    }

    #[test]
    fn point_to_gtp_bottom_right_9x9() {
        assert_eq!(point_to_gtp(Point::new(8, 8), 9), "J1");
    }

    #[test]
    fn point_to_gtp_center_9x9() {
        // Center of 9×9 is (4,4) → E5
        assert_eq!(point_to_gtp(Point::new(4, 4), 9), "E5");
    }

    #[test]
    fn point_to_gtp_skips_i() {
        // col=8 should be 'J' (since I is skipped)
        assert_eq!(point_to_gtp(Point::new(0, 8), 9), "J9");
    }

    #[test]
    fn gtp_to_point_roundtrip() {
        for row in 0..9u8 {
            for col in 0..9u8 {
                let point = Point::new(row, col);
                let gtp = point_to_gtp(point, 9);
                let back = gtp_to_point(&gtp, 9).expect("should parse");
                assert_eq!(back, point, "round-trip failed for {gtp}");
            }
        }
    }

    #[test]
    fn gtp_to_point_pass() {
        assert_eq!(gtp_to_point("pass", 9), None);
        assert_eq!(gtp_to_point("PASS", 9), None);
    }

    #[test]
    fn gtp_to_point_invalid() {
        assert_eq!(gtp_to_point("Z9", 9), None); // column out of range
        assert_eq!(gtp_to_point("A0", 9), None); // row 0 invalid
        assert_eq!(gtp_to_point("A10", 9), None); // row > board_size
    }

    #[test]
    fn history_to_katago_moves_basic() {
        let history = vec![
            MoveRecord {
                color: Color::Black,
                mv: Move::Play(Point::new(4, 4)),
                move_number: 1,
            },
            MoveRecord {
                color: Color::White,
                mv: Move::Play(Point::new(3, 3)),
                move_number: 2,
            },
        ];
        let moves = history_to_katago_moves(&history, 9);
        assert_eq!(moves, vec![
            ("B".to_string(), "E5".to_string()),
            ("W".to_string(), "D6".to_string()),
        ]);
    }

    #[test]
    fn history_skips_resign() {
        let history = vec![
            MoveRecord {
                color: Color::Black,
                mv: Move::Play(Point::new(4, 4)),
                move_number: 1,
            },
            MoveRecord {
                color: Color::White,
                mv: Move::Resign,
                move_number: 2,
            },
        ];
        let moves = history_to_katago_moves(&history, 9);
        assert_eq!(moves.len(), 1);
    }

    #[test]
    fn history_includes_pass() {
        let history = vec![MoveRecord {
            color: Color::Black,
            mv: Move::Pass,
            move_number: 1,
        }];
        let moves = history_to_katago_moves(&history, 9);
        assert_eq!(moves, vec![("B".to_string(), "pass".to_string())]);
    }

    #[test]
    fn build_query_sets_fields() {
        let history = vec![MoveRecord {
            color: Color::Black,
            mv: Move::Play(Point::new(4, 4)),
            move_number: 1,
        }];
        let query = build_query("q1".to_string(), &history, 9, 6.5, 100);
        assert_eq!(query.id, "q1");
        assert_eq!(query.board_x_size, 9);
        assert_eq!(query.board_y_size, 9);
        assert_eq!(query.komi, 6.5);
        assert_eq!(query.max_visits, 100);
        assert_eq!(query.rules, "tromp-taylor");
        assert_eq!(query.moves.len(), 1);
    }

    #[test]
    fn roundtrip_19x19() {
        for row in 0..19u8 {
            for col in 0..19u8 {
                let point = Point::new(row, col);
                let gtp = point_to_gtp(point, 19);
                let back = gtp_to_point(&gtp, 19).expect("should parse");
                assert_eq!(back, point, "19×19 round-trip failed for {gtp}");
            }
        }
    }
}
