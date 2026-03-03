use rusqlite::Connection;

use crate::error::AppError;

/// A coaching event recorded during a game session.
pub struct CoachingEvent {
    pub move_number: u16,
    pub error_class: Option<String>,
    pub severity: String,
    pub score_loss: f64,
    pub llm_used: bool,
}

/// Insert a coaching event for the current session.
pub fn insert_event(conn: &Connection, event: &CoachingEvent) -> Result<i64, AppError> {
    conn.execute(
        "INSERT INTO coaching_events (move_number, error_class, severity, score_loss, llm_used)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            event.move_number,
            event.error_class,
            event.severity,
            event.score_loss,
            event.llm_used as i32,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Count coaching events with a specific error class in the current session.
#[cfg_attr(not(feature = "llm"), allow(dead_code))]
pub fn count_class_this_session(conn: &Connection, error_class: &str) -> Result<u32, AppError> {
    let count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM coaching_events WHERE error_class = ?1",
        [error_class],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Count all mistake-level+ coaching events in the current session.
#[cfg_attr(not(feature = "llm"), allow(dead_code))]
pub fn count_mistakes_this_session(conn: &Connection) -> Result<u32, AppError> {
    let count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM coaching_events WHERE severity IN ('Inaccuracy', 'Mistake', 'Blunder')",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Clear all coaching events — called at the start of each new game.
pub fn clear_session(conn: &Connection) -> Result<(), AppError> {
    conn.execute("DELETE FROM coaching_events", [])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_schema;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn insert_and_count() {
        let conn = test_conn();
        insert_event(&conn, &CoachingEvent {
            move_number: 10,
            error_class: Some("Direction".to_string()),
            severity: "Mistake".to_string(),
            score_loss: 3.5,
            llm_used: false,
        }).unwrap();

        insert_event(&conn, &CoachingEvent {
            move_number: 20,
            error_class: Some("Direction".to_string()),
            severity: "Inaccuracy".to_string(),
            score_loss: 1.2,
            llm_used: true,
        }).unwrap();

        insert_event(&conn, &CoachingEvent {
            move_number: 30,
            error_class: Some("Shape".to_string()),
            severity: "Blunder".to_string(),
            score_loss: 8.0,
            llm_used: false,
        }).unwrap();

        assert_eq!(count_class_this_session(&conn, "Direction").unwrap(), 2);
        assert_eq!(count_class_this_session(&conn, "Shape").unwrap(), 1);
        assert_eq!(count_class_this_session(&conn, "Reading").unwrap(), 0);
        assert_eq!(count_mistakes_this_session(&conn).unwrap(), 3);
    }

    #[test]
    fn clear_session_removes_all() {
        let conn = test_conn();
        insert_event(&conn, &CoachingEvent {
            move_number: 5,
            error_class: None,
            severity: "Mistake".to_string(),
            score_loss: 2.0,
            llm_used: false,
        }).unwrap();

        assert_eq!(count_mistakes_this_session(&conn).unwrap(), 1);
        clear_session(&conn).unwrap();
        assert_eq!(count_mistakes_this_session(&conn).unwrap(), 0);
    }

    #[test]
    fn schema_idempotent() {
        let conn = test_conn();
        // Call init_schema again — should not error
        init_schema(&conn).unwrap();
        assert_eq!(count_mistakes_this_session(&conn).unwrap(), 0);
    }

    #[test]
    fn insert_returns_id() {
        let conn = test_conn();
        let id1 = insert_event(&conn, &CoachingEvent {
            move_number: 1,
            error_class: None,
            severity: "Inaccuracy".to_string(),
            score_loss: 1.0,
            llm_used: false,
        }).unwrap();

        let id2 = insert_event(&conn, &CoachingEvent {
            move_number: 2,
            error_class: None,
            severity: "Mistake".to_string(),
            score_loss: 2.0,
            llm_used: false,
        }).unwrap();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }
}
