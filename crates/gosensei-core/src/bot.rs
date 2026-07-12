//! Lightweight heuristic opponent for first-run games while KataGo downloads.
//!
//! This is not an engine — it is a stopgap policy strong enough to give a
//! beginner a sensible first 9x9 game. In priority order it captures
//! opponent stones in atari, rescues its own groups from atari, and
//! otherwise plays near existing stones with a preference for classic
//! opening points, while never filling its own eyes or playing self-atari.

use std::collections::HashSet;

use crate::board::Board;
use crate::game::{Game, GamePhase};
use crate::rules;
use crate::types::{Color, Point};

const OPENING_POINT_BONUS: i32 = 3;
const NEAR_STONES_BONUS: i32 = 2;
const FIRST_LINE_PENALTY: i32 = -3;
const SECOND_LINE_PENALTY: i32 = -1;
const JITTER_RANGE: u64 = 3;

/// Suggest a move for `color` in the current position.
///
/// Deterministic for a given `rng_seed`. Returns `None` when no sensible
/// move remains — the caller should pass.
pub fn suggest_move(game: &Game, color: Color, rng_seed: u64) -> Option<Point> {
    if *game.phase() == GamePhase::Finished {
        return None;
    }

    let board = game.board();
    let ko = game.ko_point();
    let mut rng = XorShift64::new(rng_seed);

    best_capture(board, color, ko)
        .or_else(|| best_rescue(board, color, ko))
        .or_else(|| best_developing_move(board, color, ko, &mut rng))
}

/// Capture the largest opponent group currently in atari, if any capture is legal.
fn best_capture(board: &Board, color: Color, ko: Option<Point>) -> Option<Point> {
    let opponent = color.opponent();
    let mut best: Option<(usize, Point)> = None;

    for (group_size, liberty) in atari_groups(board, opponent) {
        if rules::validate_move(board, liberty, color, ko).is_err() {
            continue;
        }
        if best.is_none_or(|(size, _)| group_size > size) {
            best = Some((group_size, liberty));
        }
    }

    best.map(|(_, point)| point)
}

/// Extend the largest own group in atari, provided the extension actually
/// escapes (the resulting group has at least two liberties).
fn best_rescue(board: &Board, color: Color, ko: Option<Point>) -> Option<Point> {
    let mut best: Option<(usize, Point)> = None;

    for (group_size, liberty) in atari_groups(board, color) {
        if !escapes_atari(board, liberty, color, ko) {
            continue;
        }
        if best.is_none_or(|(size, _)| group_size > size) {
            best = Some((group_size, liberty));
        }
    }

    best.map(|(_, point)| point)
}

/// Otherwise: prefer classic opening points and moves near existing stones,
/// with a little randomness. Excludes self-atari and own-eye fills.
fn best_developing_move(
    board: &Board,
    color: Color,
    ko: Option<Point>,
    rng: &mut XorShift64,
) -> Option<Point> {
    let stones: Vec<Point> = board
        .all_points()
        .filter(|&p| board.get(p).is_some())
        .collect();

    let mut best: Option<(i32, Point)> = None;
    for point in board.all_points() {
        if rules::validate_move(board, point, color, ko).is_err()
            || is_own_eye(board, point, color)
            || is_self_atari(board, point, color, ko)
        {
            continue;
        }

        let score =
            score_developing_move(board, point, &stones) + (rng.next() % JITTER_RANGE) as i32;
        if best.is_none_or(|(s, _)| score > s) {
            best = Some((score, point));
        }
    }

    best.map(|(_, point)| point)
}

fn score_developing_move(board: &Board, point: Point, stones: &[Point]) -> i32 {
    let dim = board.dimension();
    let mut score = 0;

    if is_opening_point(point, dim) {
        score += OPENING_POINT_BONUS;
    }

    if let Some(dist) = stones.iter().map(|s| chebyshev_distance(point, *s)).min()
        && dist <= 2
    {
        score += NEAR_STONES_BONUS;
    }

    match edge_distance(point, dim) {
        0 => score += FIRST_LINE_PENALTY,
        1 => score += SECOND_LINE_PENALTY,
        _ => {}
    }

    score
}

/// All groups of `color` that are in atari, as `(group_size, last_liberty)`.
fn atari_groups(board: &Board, color: Color) -> Vec<(usize, Point)> {
    let mut visited: HashSet<Point> = HashSet::new();
    let mut result = Vec::new();

    for point in board.all_points() {
        if board.get(point) != Some(color) || visited.contains(&point) {
            continue;
        }
        let Some(group) = board.group_at(point) else {
            continue;
        };
        visited.extend(group.stones.iter().copied());
        if group.liberty_count() == 1 {
            let liberty = *group.liberties.iter().next().expect("one liberty");
            result.push((group.stones.len(), liberty));
        }
    }

    result
}

/// Does playing `point` leave the connected group with at least two liberties?
fn escapes_atari(board: &Board, point: Point, color: Color, ko: Option<Point>) -> bool {
    resulting_liberties(board, point, color, ko).is_some_and(|libs| libs >= 2)
}

/// A move is self-atari if it neither captures nor leaves its group with
/// more than one liberty.
fn is_self_atari(board: &Board, point: Point, color: Color, ko: Option<Point>) -> bool {
    let mut test_board = board.clone();
    match rules::apply_move(&mut test_board, point, color, ko) {
        Ok(captured) => {
            captured.is_empty()
                && test_board
                    .group_at(point)
                    .is_some_and(|g| g.liberty_count() <= 1)
        }
        Err(_) => true,
    }
}

/// Liberties of the group containing `point` after playing there, or `None`
/// if the move is illegal.
fn resulting_liberties(
    board: &Board,
    point: Point,
    color: Color,
    ko: Option<Point>,
) -> Option<usize> {
    let mut test_board = board.clone();
    rules::apply_move(&mut test_board, point, color, ko).ok()?;
    test_board.group_at(point).map(|g| g.liberty_count())
}

/// Simple eye heuristic: every orthogonal neighbor is our own stone, and the
/// diagonals are not dominated by the opponent (no enemy diagonal on the
/// edge/corner, at most one in the center).
fn is_own_eye(board: &Board, point: Point, color: Color) -> bool {
    let dim = board.dimension();
    if !point.neighbors(dim).all(|n| board.get(n) == Some(color)) {
        return false;
    }

    let opponent = color.opponent();
    let mut diagonals = 0;
    let mut enemy_diagonals = 0;
    let (r, c) = (point.row as i16, point.col as i16);
    for (dr, dc) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        let (nr, nc) = (r + dr, c + dc);
        if nr < 0 || nc < 0 || nr >= dim as i16 || nc >= dim as i16 {
            continue;
        }
        diagonals += 1;
        if board.get(Point::new(nr as u8, nc as u8)) == Some(opponent) {
            enemy_diagonals += 1;
        }
    }

    if diagonals < 4 {
        enemy_diagonals == 0
    } else {
        enemy_diagonals <= 1
    }
}

/// Classic opening intersections: third line and the middle line
/// (for 9x9 that is rows/cols 2, 4, 6 — the star points and sides).
fn is_opening_point(point: Point, dim: u8) -> bool {
    let lines = [2, dim / 2, dim - 3];
    lines.contains(&point.row) && lines.contains(&point.col)
}

fn chebyshev_distance(a: Point, b: Point) -> u8 {
    let dr = a.row.abs_diff(b.row);
    let dc = a.col.abs_diff(b.col);
    dr.max(dc)
}

fn edge_distance(point: Point, dim: u8) -> u8 {
    point
        .row
        .min(point.col)
        .min(dim - 1 - point.row)
        .min(dim - 1 - point.col)
}

/// Minimal xorshift64 PRNG so the bot stays dependency-free and
/// deterministic for a given seed.
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        // A zero state would be a fixed point; nudge it.
        Self { state: seed.max(1) }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BoardSize;

    const SEED: u64 = 42;

    fn empty_game() -> Game {
        Game::new(BoardSize::Nine, 6.5)
    }

    fn set(game: &mut Game, color: Color, points: &[(u8, u8)]) {
        for &(r, c) in points {
            game.board_mut().set(Point::new(r, c), Some(color));
        }
    }

    #[test]
    fn captures_stone_in_atari() {
        let mut game = empty_game();
        // White at (0,1) with black at (0,0) and (1,1): last liberty is (0,2)
        set(&mut game, Color::White, &[(0, 1)]);
        set(&mut game, Color::Black, &[(0, 0), (1, 1)]);

        let mv = suggest_move(&game, Color::Black, SEED);
        assert_eq!(mv, Some(Point::new(0, 2)));
    }

    #[test]
    fn prefers_larger_capture() {
        let mut game = empty_game();
        // One-stone white group in atari at (0,1); last liberty (0,2)
        set(&mut game, Color::White, &[(0, 1)]);
        set(&mut game, Color::Black, &[(0, 0), (1, 1)]);
        // Two-stone white group in atari at (8,4),(8,5); last liberty (8,6)
        set(&mut game, Color::White, &[(8, 4), (8, 5)]);
        set(&mut game, Color::Black, &[(8, 3), (7, 4), (7, 5)]);

        let mv = suggest_move(&game, Color::Black, SEED);
        assert_eq!(mv, Some(Point::new(8, 6)));
    }

    #[test]
    fn saves_own_group_in_atari_by_extending() {
        let mut game = empty_game();
        // Black at (4,4) hemmed by white on three sides; last liberty (4,5).
        // Extending there gives the group three liberties.
        set(&mut game, Color::Black, &[(4, 4)]);
        set(&mut game, Color::White, &[(3, 4), (5, 4), (4, 3)]);

        let mv = suggest_move(&game, Color::Black, SEED);
        assert_eq!(mv, Some(Point::new(4, 5)));
    }

    #[test]
    fn capture_beats_rescue() {
        let mut game = empty_game();
        // Black group in atari at (4,4), rescue at (4,5)...
        set(&mut game, Color::Black, &[(4, 4)]);
        set(&mut game, Color::White, &[(3, 4), (5, 4), (4, 3)]);
        // ...but the white stone at (4,3) is itself in atari: capturing at
        // (4,2) both saves the black stone and takes a prisoner.
        set(&mut game, Color::Black, &[(3, 3), (5, 3)]);

        let mv = suggest_move(&game, Color::Black, SEED);
        assert_eq!(mv, Some(Point::new(4, 2)));
    }

    #[test]
    fn does_not_extend_into_a_dead_end() {
        let mut game = empty_game();
        // Black at (0,0) in atari; the "escape" at (0,1) is legal but leaves
        // the group with a single liberty at (0,2), so the bot should not
        // throw good stones after bad.
        set(&mut game, Color::Black, &[(0, 0)]);
        set(&mut game, Color::White, &[(1, 0), (1, 1), (0, 3)]);

        let mv = suggest_move(&game, Color::Black, SEED);
        assert_ne!(mv, Some(Point::new(0, 1)));
    }

    #[test]
    fn avoids_self_atari() {
        let mut game = empty_game();
        // Black wall leaves (0,0) and (1,0) as a two-point corner pocket.
        // A white stone on either point would have at most one liberty.
        set(&mut game, Color::Black, &[(0, 1), (1, 1), (2, 0)]);

        let mv = suggest_move(&game, Color::White, SEED).expect("board is mostly empty");
        assert_ne!(mv, Point::new(0, 0));
        assert_ne!(mv, Point::new(1, 0));
    }

    #[test]
    fn does_not_fill_own_eye() {
        let mut game = empty_game();
        // Black eye at (0,0): neighbors (0,1), (1,0) and diagonal (1,1) black.
        set(&mut game, Color::Black, &[(0, 1), (1, 0), (1, 1)]);

        let mv = suggest_move(&game, Color::Black, SEED).expect("board is mostly empty");
        assert_ne!(mv, Point::new(0, 0));
    }

    #[test]
    fn passes_when_only_own_eyes_remain() {
        let mut game = empty_game();
        // Fill the whole board with black except two eyes.
        let eyes = [Point::new(0, 0), Point::new(8, 8)];
        for p in game.board().all_points().collect::<Vec<_>>() {
            if !eyes.contains(&p) {
                game.board_mut().set(p, Some(Color::Black));
            }
        }

        // Black must not fill its own eyes; White has only suicide moves.
        assert_eq!(suggest_move(&game, Color::Black, SEED), None);
        assert_eq!(suggest_move(&game, Color::White, SEED), None);
    }

    #[test]
    fn no_move_after_game_over() {
        let mut game = empty_game();
        game.resign().unwrap();
        assert_eq!(suggest_move(&game, Color::Black, SEED), None);
    }

    #[test]
    fn first_move_on_empty_board_is_an_opening_point() {
        for seed in 1..=20 {
            let game = empty_game();
            let mv = suggest_move(&game, Color::Black, seed).expect("empty board has moves");
            assert!(
                [2u8, 4, 6].contains(&mv.row) && [2u8, 4, 6].contains(&mv.col),
                "seed {seed} gave non-opening move {mv:?}"
            );
        }
    }

    #[test]
    fn deterministic_for_a_given_seed() {
        let mut game = empty_game();
        set(&mut game, Color::Black, &[(4, 4), (2, 6)]);
        set(&mut game, Color::White, &[(6, 2)]);

        let first = suggest_move(&game, Color::White, SEED);
        let second = suggest_move(&game, Color::White, SEED);
        assert_eq!(first, second);
    }

    #[test]
    fn respects_ko() {
        let mut game = empty_game();
        // Black diamond around (1,1) minus one stone; white wall to its right.
        set(&mut game, Color::Black, &[(0, 1), (1, 0), (2, 1)]);
        set(&mut game, Color::White, &[(0, 2), (2, 2), (1, 3), (1, 1)]);

        // Black captures the white stone at (1,1) by playing (1,2) — a ko.
        game.set_current_color(Color::Black);
        game.play(Point::new(1, 2)).unwrap();
        assert_eq!(game.ko_point(), Some(Point::new(1, 1)));

        // White may not recapture at the ko point immediately, even though
        // the black stone at (1,2) is in atari.
        let mv = suggest_move(&game, Color::White, SEED);
        assert_ne!(mv, Some(Point::new(1, 1)));
        if let Some(p) = mv {
            assert!(rules::validate_move(game.board(), p, Color::White, game.ko_point()).is_ok());
        }
    }

    #[test]
    fn self_play_yields_only_legal_moves_and_finishes() {
        let mut game = empty_game();
        for turn in 0..300u64 {
            if *game.phase() == GamePhase::Finished {
                return; // Game ended by consecutive passes — success.
            }
            let color = game.current_color();
            match suggest_move(&game, color, 0xDEAD_BEEF ^ turn) {
                Some(point) => {
                    game.play(point).unwrap_or_else(|e| {
                        panic!("illegal suggestion {point:?} on turn {turn}: {e}")
                    });
                }
                None => game.pass().unwrap(),
            }
        }
        panic!("self-play did not finish within 300 turns");
    }
}
