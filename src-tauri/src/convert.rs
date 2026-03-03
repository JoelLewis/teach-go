use gosensei_core::types::{Color, Move, MoveRecord, Point};
use gosensei_katago::protocol::AnalysisQuery;

/// GTP column letters: A–T, skipping I.
const GTP_COLUMNS: [char; 25] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
    'U', 'V', 'W', 'X', 'Y', 'Z',
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
    human_sl_profile: Option<String>,
    include_ownership: Option<bool>,
) -> AnalysisQuery {
    AnalysisQuery {
        id,
        moves: history_to_katago_moves(history, board_size),
        rules: "tromp-taylor".to_string(),
        komi,
        board_x_size: board_size,
        board_y_size: board_size,
        max_visits,
        include_ownership,
        include_policy: None,
        human_sl_profile,
    }
}

/// Map a GoSensei AI strength setting to a KataGo humanSLProfile name.
pub fn strength_to_profile(strength: &str) -> Option<String> {
    match strength {
        "beginner" => Some("preaz_18k".to_string()),
        "intermediate" => Some("preaz_9k".to_string()),
        "advanced" => Some("preaz_3k".to_string()),
        "dan" => None, // Full strength — no profile
        _ => None,
    }
}

/// Map a numeric player rank to the closest KataGo humanSLProfile.
/// Used for Human SL policy queries to understand what a human at this rank would play.
#[cfg_attr(not(feature = "llm"), allow(dead_code))]
pub fn rank_to_human_profile(rank: f64) -> String {
    match rank as u32 {
        20..=u32::MAX => "preaz_18k",
        15..=19 => "preaz_15k",
        10..=14 => "preaz_9k",
        5..=9 => "preaz_5k",
        1..=4 => "preaz_3k",
        _ => "preaz_1d",
    }
    .to_string()
}

/// Map a rank to the profile of a player ~5 stones stronger.
/// For comparing what a slightly better player would do.
#[cfg_attr(not(feature = "llm"), allow(dead_code))]
pub fn rank_one_up_profile(rank: f64) -> String {
    rank_to_human_profile((rank - 5.0).max(0.0))
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
        assert_eq!(
            moves,
            vec![
                ("B".to_string(), "E5".to_string()),
                ("W".to_string(), "D6".to_string()),
            ]
        );
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
        let query = build_query("q1".to_string(), &history, 9, 6.5, 100, None, None);
        assert_eq!(query.id, "q1");
        assert_eq!(query.board_x_size, 9);
        assert_eq!(query.board_y_size, 9);
        assert_eq!(query.komi, 6.5);
        assert_eq!(query.max_visits, 100);
        assert_eq!(query.rules, "tromp-taylor");
        assert_eq!(query.moves.len(), 1);
    }

    #[test]
    fn query_serializes_to_valid_json() {
        let history = vec![
            MoveRecord {
                color: Color::Black,
                mv: Move::Play(Point::new(4, 4)),
                move_number: 1,
            },
            MoveRecord {
                color: Color::White,
                mv: Move::Play(Point::new(2, 6)),
                move_number: 2,
            },
        ];
        let query = build_query(
            "test-q1".to_string(),
            &history,
            9,
            6.5,
            200,
            Some("preaz_18k".to_string()),
            None,
        );
        let json = serde_json::to_string(&query).unwrap();

        // Should be valid JSON with expected fields
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["id"], "test-q1");
        assert_eq!(parsed["boardXSize"], 9);
        assert_eq!(parsed["rules"], "tromp-taylor");
        assert_eq!(parsed["moves"][0][0], "B");
        assert_eq!(parsed["moves"][0][1], "E5");
        assert_eq!(parsed["moves"][1][0], "W");
    }

    #[test]
    fn mock_response_parses_correctly() {
        use gosensei_katago::protocol::AnalysisResponse;

        let mock_response = r#"{
            "id": "test-q1",
            "moveInfos": [
                {
                    "move": "D4",
                    "visits": 150,
                    "winrate": 0.55,
                    "scoreLead": 2.5,
                    "prior": 0.12,
                    "order": 0,
                    "pv": ["D4", "C3", "E5"]
                },
                {
                    "move": "Q16",
                    "visits": 50,
                    "winrate": 0.52,
                    "scoreLead": 1.2,
                    "prior": 0.08,
                    "order": 1,
                    "pv": ["Q16", "R14"]
                }
            ],
            "rootInfo": {
                "winrate": 0.55,
                "scoreLead": 2.5,
                "visits": 200
            },
            "ownership": []
        }"#;

        let response: AnalysisResponse = serde_json::from_str(mock_response).unwrap();
        assert_eq!(response.id, "test-q1");
        assert_eq!(response.move_infos.len(), 2);

        let best = &response.move_infos[0];
        assert_eq!(best.mv, "D4");
        assert_eq!(best.visits, 150);
        assert!(best.winrate > 0.5);

        // Convert the best move back to a Point
        let point = gtp_to_point(&best.mv, 9).unwrap();
        assert_eq!(point, Point::new(5, 3)); // D4 on a 9x9 board
    }

    #[test]
    fn mock_pass_response() {
        use gosensei_katago::protocol::AnalysisResponse;

        let mock_response = r#"{
            "id": "endgame-q",
            "moveInfos": [
                {
                    "move": "pass",
                    "visits": 200,
                    "winrate": 0.99,
                    "scoreLead": 50.0,
                    "prior": 0.95,
                    "order": 0,
                    "pv": ["pass"]
                }
            ],
            "rootInfo": {
                "winrate": 0.99,
                "scoreLead": 50.0,
                "visits": 200
            }
        }"#;

        let response: AnalysisResponse = serde_json::from_str(mock_response).unwrap();
        let best = &response.move_infos[0];

        // "pass" should return None from gtp_to_point
        assert!(gtp_to_point(&best.mv, 9).is_none());
    }

    #[test]
    fn strength_mapping() {
        assert_eq!(
            strength_to_profile("beginner"),
            Some("preaz_18k".to_string())
        );
        assert_eq!(
            strength_to_profile("intermediate"),
            Some("preaz_9k".to_string())
        );
        assert_eq!(
            strength_to_profile("advanced"),
            Some("preaz_3k".to_string())
        );
        assert_eq!(strength_to_profile("dan"), None);
        assert_eq!(strength_to_profile("unknown"), None);
    }

    #[test]
    fn query_with_profile_serializes() {
        let query = build_query(
            "profile-test".to_string(),
            &[],
            9,
            6.5,
            100,
            Some("preaz_18k".to_string()),
            None,
        );
        let json = serde_json::to_string(&query).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["humanSlProfile"], "preaz_18k");
    }

    #[test]
    fn query_without_profile_omits_field() {
        let query = build_query("no-profile".to_string(), &[], 9, 6.5, 100, None, None);
        let json = serde_json::to_string(&query).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("humanSlProfile").is_none());
    }

    #[test]
    fn query_with_ownership_serializes() {
        let query = build_query("own-test".to_string(), &[], 9, 6.5, 50, None, Some(true));
        let json = serde_json::to_string(&query).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["includeOwnership"], true);
    }

    #[test]
    fn query_without_ownership_omits_field() {
        let query = build_query("no-own".to_string(), &[], 9, 6.5, 50, None, None);
        let json = serde_json::to_string(&query).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("includeOwnership").is_none());
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

    #[test]
    fn rank_to_human_profile_mappings() {
        assert_eq!(rank_to_human_profile(25.0), "preaz_18k");
        assert_eq!(rank_to_human_profile(20.0), "preaz_18k");
        assert_eq!(rank_to_human_profile(15.0), "preaz_15k");
        assert_eq!(rank_to_human_profile(12.0), "preaz_9k");
        assert_eq!(rank_to_human_profile(7.0), "preaz_5k");
        assert_eq!(rank_to_human_profile(3.0), "preaz_3k");
        assert_eq!(rank_to_human_profile(0.5), "preaz_1d");
    }

    #[test]
    fn rank_one_up_advances_band() {
        // 25k → one_up is 20k → still preaz_18k band
        assert_eq!(rank_one_up_profile(25.0), "preaz_18k");
        // 15k → one_up is 10k → preaz_9k band
        assert_eq!(rank_one_up_profile(15.0), "preaz_9k");
        // 10k → one_up is 5k → preaz_5k band
        assert_eq!(rank_one_up_profile(10.0), "preaz_5k");
        // 5k → one_up is 0 → preaz_1d (clamped)
        assert_eq!(rank_one_up_profile(5.0), "preaz_1d");
        // 3k → one_up is 0 → preaz_1d (clamped at 0.0)
        assert_eq!(rank_one_up_profile(3.0), "preaz_1d");
    }
}
