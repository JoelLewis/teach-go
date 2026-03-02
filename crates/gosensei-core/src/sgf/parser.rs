use crate::types::{BoardSize, Color, Move, Point};

#[derive(Debug, Clone)]
pub struct SgfGame {
    pub board_size: BoardSize,
    pub komi: f32,
    pub moves: Vec<(Color, Move)>,
    pub player_black: Option<String>,
    pub player_white: Option<String>,
    pub result: Option<String>,
    /// Setup stones placed via AB[] (Add Black) properties.
    pub setup_black: Vec<Point>,
    /// Setup stones placed via AW[] (Add White) properties.
    pub setup_white: Vec<Point>,
    /// Color to play first, from PL[] property.
    pub player_to_move: Option<Color>,
}

#[derive(Debug, thiserror::Error)]
pub enum SgfParseError {
    #[error("invalid SGF format: {0}")]
    InvalidFormat(String),

    #[error("unsupported board size: {0}")]
    UnsupportedBoardSize(u8),
}

/// Parse a minimal SGF string into a game record.
pub fn parse(input: &str) -> Result<SgfGame, SgfParseError> {
    let input = input.trim();
    if !input.starts_with("(;") {
        return Err(SgfParseError::InvalidFormat(
            "SGF must start with '(;'".into(),
        ));
    }

    let mut board_size = BoardSize::Nineteen;
    let mut komi = 6.5;
    let mut moves = Vec::new();
    let mut player_black = None;
    let mut player_white = None;
    let mut result = None;
    let mut setup_black = Vec::new();
    let mut setup_white = Vec::new();
    let mut player_to_move = None;

    // Simple property parser — handles SZ, KM, PB, PW, RE, B, W
    let mut chars = input[2..].chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            ')' => break,
            ';' => {
                chars.next();
            }
            c if c.is_ascii_uppercase() => {
                let mut prop = String::new();
                while let Some(&pc) = chars.peek() {
                    if pc.is_ascii_uppercase() {
                        prop.push(pc);
                        chars.next();
                    } else {
                        break;
                    }
                }

                // Read value(s) in brackets
                let mut values = Vec::new();
                while chars.peek() == Some(&'[') {
                    chars.next(); // skip '['
                    let mut val = String::new();
                    let mut escaped = false;
                    for vc in chars.by_ref() {
                        if escaped {
                            val.push(vc);
                            escaped = false;
                        } else if vc == '\\' {
                            escaped = true;
                        } else if vc == ']' {
                            break;
                        } else {
                            val.push(vc);
                        }
                    }
                    values.push(val);
                }

                match prop.as_str() {
                    "SZ" => {
                        if let Some(val) = values.first() {
                            let size: u8 = val.parse().unwrap_or(19);
                            board_size = BoardSize::try_from(size)
                                .map_err(|_| SgfParseError::UnsupportedBoardSize(size))?;
                        }
                    }
                    "KM" => {
                        if let Some(val) = values.first() {
                            komi = val.parse().unwrap_or(6.5);
                        }
                    }
                    "PB" => player_black = values.into_iter().next(),
                    "PW" => player_white = values.into_iter().next(),
                    "RE" => result = values.into_iter().next(),
                    "B" | "W" => {
                        let color = if prop == "B" {
                            Color::Black
                        } else {
                            Color::White
                        };
                        if let Some(val) = values.first() {
                            let mv = parse_move(val);
                            moves.push((color, mv));
                        }
                    }
                    "AB" => {
                        for val in &values {
                            if let Move::Play(p) = parse_move(val) {
                                setup_black.push(p);
                            }
                        }
                    }
                    "AW" => {
                        for val in &values {
                            if let Move::Play(p) = parse_move(val) {
                                setup_white.push(p);
                            }
                        }
                    }
                    "PL" => {
                        if let Some(val) = values.first() {
                            player_to_move = match val.as_str() {
                                "B" => Some(Color::Black),
                                "W" => Some(Color::White),
                                _ => None,
                            };
                        }
                    }
                    _ => {} // Ignore unknown properties
                }
            }
            _ => {
                chars.next();
            }
        }
    }

    Ok(SgfGame {
        board_size,
        komi,
        moves,
        player_black,
        player_white,
        result,
        setup_black,
        setup_white,
        player_to_move,
    })
}

fn parse_move(coord: &str) -> Move {
    if coord.is_empty() || coord == "tt" {
        return Move::Pass;
    }
    let bytes = coord.as_bytes();
    if bytes.len() >= 2 {
        let col = bytes[0] - b'a';
        let row = bytes[1] - b'a';
        Move::Play(Point::new(row, col))
    } else {
        Move::Pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_game() {
        let sgf = "(;SZ[9]KM[5.5]PB[Player1]PW[Player2];B[ee];W[cc])";
        let game = parse(sgf).unwrap();
        assert_eq!(game.board_size, BoardSize::Nine);
        assert_eq!(game.komi, 5.5);
        assert_eq!(game.moves.len(), 2);
        assert_eq!(game.player_black, Some("Player1".into()));
    }

    #[test]
    fn parse_pass_move() {
        let sgf = "(;SZ[9];B[ee];W[])";
        let game = parse(sgf).unwrap();
        assert_eq!(game.moves.len(), 2);
        assert_eq!(game.moves[1].1, Move::Pass);
    }

    #[test]
    fn parse_setup_stones() {
        // SGF coords: first char=col, second char=row
        // aa = col 0, row 0; ab = col 0, row 1; ac = col 0, row 2
        // ba = col 1, row 0; bb = col 1, row 1
        let sgf = "(;SZ[9]AB[aa][ab][ac]AW[ba][bb])";
        let game = parse(sgf).unwrap();
        assert_eq!(game.setup_black.len(), 3);
        assert_eq!(game.setup_white.len(), 2);
        assert_eq!(game.setup_black[0], Point::new(0, 0));
        assert_eq!(game.setup_black[1], Point::new(1, 0));
        assert_eq!(game.setup_black[2], Point::new(2, 0));
        assert_eq!(game.setup_white[0], Point::new(0, 1));
        assert_eq!(game.setup_white[1], Point::new(1, 1));
        assert_eq!(game.moves.len(), 0);
    }

    #[test]
    fn parse_setup_with_moves_and_pl() {
        let sgf = "(;SZ[9]AB[dd][de]AW[ed][ee]PL[B];B[df])";
        let game = parse(sgf).unwrap();
        assert_eq!(game.setup_black.len(), 2);
        assert_eq!(game.setup_white.len(), 2);
        assert_eq!(game.player_to_move, Some(Color::Black));
        assert_eq!(game.moves.len(), 1);
    }
}
