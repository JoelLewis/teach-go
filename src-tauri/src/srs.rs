use chrono::{DateTime, NaiveDateTime, Utc};
use rs_fsrs::{Card, FSRS, Rating, State};
use rusqlite::Connection;

use crate::error::AppError;

/// Map solve outcome to FSRS rating.
pub fn solve_to_rating(solved: bool, hints_used: u8) -> Rating {
    if !solved {
        Rating::Again
    } else if hints_used >= 2 {
        Rating::Hard
    } else if hints_used == 1 {
        Rating::Good
    } else {
        Rating::Easy
    }
}

/// Read an SRS card from the database, or return a new card if none exists.
pub fn get_card(conn: &Connection, problem_id: i64) -> Result<Card, AppError> {
    let mut stmt = conn.prepare(
        "SELECT due, stability, difficulty_fsrs, elapsed_days, scheduled_days,
                reps, lapses, state, last_review
         FROM srs_cards WHERE problem_id = ?1",
    )?;

    let card = stmt.query_row([problem_id], |row| {
        let due_str: String = row.get(0)?;
        let last_review_str: String = row.get(8)?;

        Ok(Card {
            due: parse_datetime(&due_str),
            stability: row.get(1)?,
            difficulty: row.get(2)?,
            elapsed_days: row.get(3)?,
            scheduled_days: row.get(4)?,
            reps: row.get(5)?,
            lapses: row.get(6)?,
            state: int_to_state(row.get::<_, i32>(7)?),
            last_review: parse_datetime(&last_review_str),
        })
    });

    match card {
        Ok(c) => Ok(c),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Card::new()),
        Err(e) => Err(e.into()),
    }
}

/// Apply a rating to a card and upsert the result into the database.
pub fn update_card(conn: &Connection, problem_id: i64, rating: Rating) -> Result<Card, AppError> {
    let card = get_card(conn, problem_id)?;
    let fsrs = FSRS::new(Default::default());
    let now = Utc::now();
    let info = fsrs.next(card, now, rating);
    let updated = info.card;

    conn.execute(
        "INSERT INTO srs_cards (problem_id, due, stability, difficulty_fsrs, elapsed_days,
                                scheduled_days, reps, lapses, state, last_review)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(problem_id) DO UPDATE SET
             due = ?2, stability = ?3, difficulty_fsrs = ?4, elapsed_days = ?5,
             scheduled_days = ?6, reps = ?7, lapses = ?8, state = ?9, last_review = ?10",
        rusqlite::params![
            problem_id,
            updated.due.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated.stability,
            updated.difficulty,
            updated.elapsed_days,
            updated.scheduled_days,
            updated.reps,
            updated.lapses,
            state_to_int(updated.state),
            updated.last_review.format("%Y-%m-%d %H:%M:%S").to_string(),
        ],
    )?;

    Ok(updated)
}

/// Get problem IDs that are due for review (past their scheduled date).
pub fn get_due_problems(conn: &Connection, limit: u32) -> Result<Vec<i64>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT problem_id FROM srs_cards
         WHERE due <= datetime('now')
         ORDER BY due ASC
         LIMIT ?1",
    )?;

    let rows = stmt.query_map([limit], |row| row.get(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

/// Get problem IDs that have never been reviewed (no SRS card entry).
pub fn get_unseen_problems(conn: &Connection, limit: u32) -> Result<Vec<i64>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT p.id FROM problems p
         LEFT JOIN srs_cards s ON p.id = s.problem_id
         WHERE s.problem_id IS NULL
         LIMIT ?1",
    )?;

    let rows = stmt.query_map([limit], |row| row.get(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }
    Ok(ids)
}

fn parse_datetime(s: &str) -> DateTime<Utc> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|_| Utc::now())
}

fn state_to_int(state: State) -> i32 {
    match state {
        State::New => 0,
        State::Learning => 1,
        State::Review => 2,
        State::Relearning => 3,
    }
}

fn int_to_state(i: i32) -> State {
    match i {
        1 => State::Learning,
        2 => State::Review,
        3 => State::Relearning,
        _ => State::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_schema;
    use crate::problem;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        problem::seed_problems_if_empty(&conn).unwrap();
        conn
    }

    #[test]
    fn solve_to_rating_mapping() {
        assert_eq!(solve_to_rating(false, 0) as u8, Rating::Again as u8);
        assert_eq!(solve_to_rating(true, 2) as u8, Rating::Hard as u8);
        assert_eq!(solve_to_rating(true, 3) as u8, Rating::Hard as u8);
        assert_eq!(solve_to_rating(true, 1) as u8, Rating::Good as u8);
        assert_eq!(solve_to_rating(true, 0) as u8, Rating::Easy as u8);
    }

    #[test]
    fn card_creation_and_retrieval() {
        let conn = test_db();
        let card = get_card(&conn, 1).unwrap();
        assert_eq!(card.reps, 0);
        assert!(matches!(card.state, State::New));
    }

    #[test]
    fn update_card_advances_due_date() {
        let conn = test_db();

        let card = update_card(&conn, 1, Rating::Easy).unwrap();
        assert!(card.scheduled_days > 0);
        assert!(card.reps > 0);

        let card2 = update_card(&conn, 2, Rating::Again).unwrap();
        assert!(card2.scheduled_days <= card.scheduled_days);
    }

    #[test]
    fn get_due_problems_returns_past_due() {
        let conn = test_db();
        update_card(&conn, 1, Rating::Again).unwrap();

        conn.execute(
            "UPDATE srs_cards SET due = datetime('now', '-1 hour') WHERE problem_id = 1",
            [],
        )
        .unwrap();

        let due = get_due_problems(&conn, 50).unwrap();
        assert!(due.contains(&1));
    }

    #[test]
    fn get_unseen_problems_excludes_reviewed() {
        let conn = test_db();

        update_card(&conn, 1, Rating::Good).unwrap();

        // Problem 1 was reviewed, so it should not appear in unseen results
        let unseen_after = get_unseen_problems(&conn, 5000).unwrap();
        assert!(
            !unseen_after.contains(&1),
            "reviewed problem should be excluded"
        );
    }
}
