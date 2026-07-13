use rusqlite::Connection;
use std::collections::HashMap;

use crate::error::AppError;

/// Apply the application schema to an open connection.
/// Safe to call multiple times — all tables use IF NOT EXISTS.
pub fn init_schema(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS players (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            rank TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            player_id INTEGER REFERENCES players(id),
            board_size INTEGER NOT NULL,
            sgf TEXT,
            result TEXT,
            player_color TEXT NOT NULL DEFAULT 'black',
            played_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS skill_profiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            player_id INTEGER DEFAULT 1,
            overall_rank REAL NOT NULL DEFAULT 25.0,
            reading_mu REAL NOT NULL DEFAULT 25.0,
            reading_sigma REAL NOT NULL DEFAULT 8.0,
            shape_mu REAL NOT NULL DEFAULT 25.0,
            shape_sigma REAL NOT NULL DEFAULT 8.0,
            direction_mu REAL NOT NULL DEFAULT 25.0,
            direction_sigma REAL NOT NULL DEFAULT 8.0,
            endgame_mu REAL NOT NULL DEFAULT 25.0,
            endgame_sigma REAL NOT NULL DEFAULT 8.0,
            life_death_mu REAL NOT NULL DEFAULT 25.0,
            life_death_sigma REAL NOT NULL DEFAULT 8.0,
            fighting_mu REAL NOT NULL DEFAULT 25.0,
            fighting_sigma REAL NOT NULL DEFAULT 8.0,
            games_played INTEGER NOT NULL DEFAULT 0,
            last_updated TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS problems (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            setup_sgf TEXT NOT NULL,
            board_size INTEGER NOT NULL,
            player_color TEXT NOT NULL DEFAULT 'black',
            solutions_json TEXT NOT NULL,
            category TEXT NOT NULL,
            difficulty REAL NOT NULL DEFAULT 20.0,
            source TEXT NOT NULL DEFAULT 'seed',
            source_game_id INTEGER REFERENCES games(id),
            prompt TEXT NOT NULL DEFAULT 'Black to play',
            tags_json TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS problem_attempts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            problem_id INTEGER NOT NULL REFERENCES problems(id),
            player_id INTEGER DEFAULT 1,
            status TEXT NOT NULL,
            hints_used INTEGER NOT NULL DEFAULT 0,
            attempts INTEGER NOT NULL DEFAULT 1,
            time_seconds INTEGER NOT NULL DEFAULT 0,
            attempted_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS srs_cards (
            problem_id INTEGER PRIMARY KEY REFERENCES problems(id),
            player_id INTEGER DEFAULT 1,
            due TEXT NOT NULL DEFAULT (datetime('now')),
            stability REAL NOT NULL DEFAULT 0.0,
            difficulty_fsrs REAL NOT NULL DEFAULT 0.0,
            elapsed_days INTEGER NOT NULL DEFAULT 0,
            scheduled_days INTEGER NOT NULL DEFAULT 0,
            reps INTEGER NOT NULL DEFAULT 0,
            lapses INTEGER NOT NULL DEFAULT 0,
            state INTEGER NOT NULL DEFAULT 0,
            last_review TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS coaching_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            move_number INTEGER NOT NULL,
            error_class TEXT,
            severity TEXT NOT NULL,
            score_loss REAL NOT NULL,
            llm_used INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS skill_history (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            player_id     INTEGER NOT NULL DEFAULT 1,
            recorded_at   TEXT NOT NULL DEFAULT (datetime('now')),
            source        TEXT NOT NULL DEFAULT 'game',
            overall_rank  REAL NOT NULL,
            reading_mu    REAL NOT NULL,
            shape_mu      REAL NOT NULL,
            direction_mu  REAL NOT NULL,
            endgame_mu    REAL NOT NULL,
            life_death_mu REAL NOT NULL,
            fighting_mu   REAL NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_skill_history_player_recorded
            ON skill_history (player_id, recorded_at);
        ",
    )?;

    Ok(())
}

fn reconcile_existing_problems(conn: &Connection) -> Result<(), AppError> {
    let tx = conn.unchecked_transaction()?;
    let mut stmt = tx.prepare(
        "SELECT id, setup_sgf, player_color, solutions_json, prompt, tags_json
         FROM problems ORDER BY id ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    let mut survivors = HashMap::new();
    for (position, (id, setup_sgf, player_color, solutions_json, prompt, tags_json)) in
        rows.into_iter().enumerate()
    {
        let clean_prompt = crate::import::clean_existing_prompt(&prompt, position + 1);
        let mut tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        if !tags.iter().any(|tag| tag == "category-inferred") {
            tags.push("category-inferred".to_string());
        }
        if !tags.iter().any(|tag| tag == "difficulty-inferred") {
            tags.push("difficulty-inferred".to_string());
        }
        let clean_tags = serde_json::to_string(&tags).map_err(|error| {
            AppError::Other(format!("serialize reconciled problem tags: {error}"))
        })?;
        tx.execute(
            "UPDATE problems SET prompt = ?1, tags_json = ?2 WHERE id = ?3",
            rusqlite::params![clean_prompt, clean_tags, id],
        )?;

        let key = crate::import::problem_identity(&setup_sgf, &player_color, &solutions_json);
        if let Some(&survivor_id) = survivors.get(&key) {
            tx.execute(
                "UPDATE problem_attempts SET problem_id = ?1 WHERE problem_id = ?2",
                rusqlite::params![survivor_id, id],
            )?;
            tx.execute("DELETE FROM srs_cards WHERE problem_id = ?1", [id])?;
            tx.execute("DELETE FROM problems WHERE id = ?1", [id])?;
        } else {
            survivors.insert(key, id);
        }
    }
    tx.commit()?;
    Ok(())
}

/// Run idempotent migrations for schema changes on existing databases.
fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    // Add player_color column to games table (added in coaching update).
    // skill_history needs no entry here: init_schema runs on every open and
    // its CREATE TABLE IF NOT EXISTS covers pre-beta databases.
    match conn
        .execute_batch("ALTER TABLE games ADD COLUMN player_color TEXT NOT NULL DEFAULT 'black'")
    {
        Ok(()) => {}
        Err(rusqlite::Error::SqliteFailure(_, Some(message)))
            if message.contains("duplicate column name") => {}
        Err(error) => return Err(error.into()),
    }

    // Migrate the 4-tier ai_strength values to rank tokens (rank slider update).
    // Idempotent: each UPDATE only matches the legacy value it rewrites.
    conn.execute_batch(
        "UPDATE settings SET value = '18k' WHERE key = 'ai_strength' AND value = 'beginner';
         UPDATE settings SET value = '9k'  WHERE key = 'ai_strength' AND value = 'intermediate';
         UPDATE settings SET value = '3k'  WHERE key = 'ai_strength' AND value = 'advanced';
         UPDATE settings SET value = 'max' WHERE key = 'ai_strength' AND value = 'dan';",
    )?;
    reconcile_existing_problems(conn)?;
    Ok(())
}

/// Initialize the SQLite database with the application schema.
pub fn init_db(path: &str) -> Result<Connection, AppError> {
    let conn = Connection::open(path)?;
    init_schema(&conn)?;
    run_migrations(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ai_strength(conn: &Connection) -> String {
        conn.query_row(
            "SELECT value FROM settings WHERE key = 'ai_strength'",
            [],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[test]
    fn migrates_legacy_ai_strength_to_rank_token_idempotently() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('ai_strength', 'advanced')",
            [],
        )
        .unwrap();

        run_migrations(&conn).unwrap();
        assert_eq!(ai_strength(&conn), "3k");

        // Second run must be a no-op
        run_migrations(&conn).unwrap();
        assert_eq!(ai_strength(&conn), "3k");
    }

    #[test]
    fn migrates_all_legacy_tiers() {
        let cases = [
            ("beginner", "18k"),
            ("intermediate", "9k"),
            ("advanced", "3k"),
            ("dan", "max"),
        ];
        for (legacy, expected) in cases {
            let conn = Connection::open_in_memory().unwrap();
            init_schema(&conn).unwrap();
            conn.execute(
                "INSERT INTO settings (key, value) VALUES ('ai_strength', ?1)",
                [legacy],
            )
            .unwrap();
            run_migrations(&conn).unwrap();
            assert_eq!(ai_strength(&conn), expected, "legacy value {legacy}");
        }
    }

    #[test]
    fn leaves_rank_token_ai_strength_untouched() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('ai_strength', '5d')",
            [],
        )
        .unwrap();
        run_migrations(&conn).unwrap();
        assert_eq!(ai_strength(&conn), "5d");
    }

    #[test]
    fn migration_propagates_unexpected_schema_errors() {
        let conn = Connection::open_in_memory().unwrap();
        let error = run_migrations(&conn).unwrap_err();
        assert!(error.to_string().contains("no such table: games"));
    }

    #[test]
    fn reconciles_dirty_duplicate_problems_and_preserves_attempts() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn.execute(
            "INSERT INTO problems
             (setup_sgf, board_size, player_color, solutions_json, category, difficulty, prompt, tags_json)
             VALUES ('(;SZ[9]AB[dd])', 9, 'black', '[]', 'LifeDeath', 1.0,
                     'GZP1', '[\"imported\"]')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO problems
             (setup_sgf, board_size, player_color, solutions_json, category, difficulty, prompt, tags_json)
             VALUES ('(;SZ[9]AB[dd])', 9, 'black', '[]', 'LifeDeath', 1.0,
                     'Black to play. https://gogameguru.com/', '[\"imported\"]')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO problem_attempts (problem_id, status) VALUES (2, 'solved')",
            [],
        )
        .unwrap();

        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let problems: Vec<(i64, String, String)> = conn
            .prepare("SELECT id, prompt, tags_json FROM problems")
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].0, 1);
        assert_eq!(problems[0].1, "Life & Death #1");
        assert!(problems[0].2.contains("category-inferred"));
        assert_eq!(
            conn.query_row("SELECT problem_id FROM problem_attempts", [], |row| row
                .get::<_, i64>(0),)
                .unwrap(),
            1
        );
    }
}
