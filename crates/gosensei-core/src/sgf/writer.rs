use crate::game::Game;
use crate::types::{Color, Move, Point};

/// Serialize a game to SGF format.
pub fn write(game: &Game) -> String {
    let mut sgf = String::from("(;");

    // Header properties
    let dim = game.board().dimension();
    sgf.push_str(&format!("SZ[{dim}]"));
    sgf.push_str("GM[1]FF[4]");
    sgf.push_str("AP[GoSensei:0.1]");

    // Moves
    for record in game.move_history() {
        let prop = match record.color {
            Color::Black => "B",
            Color::White => "W",
        };
        let coord = match record.mv {
            Move::Play(p) => format_point(p),
            Move::Pass => String::new(),
            Move::Resign => continue,
        };
        sgf.push_str(&format!(";{prop}[{coord}]"));
    }

    sgf.push(')');
    sgf
}

pub fn format_point(p: Point) -> String {
    let col = (b'a' + p.col) as char;
    let row = (b'a' + p.row) as char;
    format!("{col}{row}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BoardSize;

    #[test]
    fn write_simple_game() {
        let mut game = Game::new(BoardSize::Nine, 6.5);
        game.play(Point::new(4, 4)).unwrap();
        game.play(Point::new(2, 2)).unwrap();
        let sgf = write(&game);
        assert!(sgf.starts_with("(;SZ[9]"));
        assert!(sgf.contains(";B[ee]"));
        assert!(sgf.contains(";W[cc]"));
        assert!(sgf.ends_with(')'));
    }
}
