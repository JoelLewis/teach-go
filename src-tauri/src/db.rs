use rusqlite::Connection;

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

        -- Migration: add player_color to games (idempotent for existing DBs)
        -- New installs get it from CREATE TABLE; ALTER is for upgrades.

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

/// Run idempotent migrations for schema changes on existing databases.
fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    // Add player_color column to games table (added in coaching update)
    let _ = conn.execute_batch("ALTER TABLE games ADD COLUMN player_color TEXT NOT NULL DEFAULT 'black'");

    // Add skill_history table (added in beta)
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS skill_history (
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
            ON skill_history (player_id, recorded_at);",
    );
    Ok(())
}

/// Initialize the SQLite database with the application schema.
pub fn init_db(path: &str) -> Result<Connection, AppError> {
    let conn = Connection::open(path)?;
    init_schema(&conn)?;
    run_migrations(&conn)?;
    Ok(conn)
}
