use gosensei_core::game::GameState;
use gosensei_core::types::Point;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::import;
use crate::problem::{self, ProblemSummary};
use crate::solver::{HintData, HintLevel, MoveResult, SolveStatus, SolverSession};
use crate::srs;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemState {
    pub problem_id: i64,
    pub board_state: GameState,
    pub prompt: String,
    pub category: String,
    pub status: SolveStatus,
    pub hints_used: u8,
    pub attempts: u16,
    pub elapsed_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveMoveResult {
    pub move_result: MoveResult,
    pub board_state: GameState,
    pub status: SolveStatus,
}

fn solver_to_problem_state(solver: &SolverSession) -> ProblemState {
    ProblemState {
        problem_id: solver.problem_id(),
        board_state: solver.game().to_state(),
        prompt: solver.prompt().to_string(),
        category: solver.category().to_string(),
        status: solver.status(),
        hints_used: solver.hints_used(),
        attempts: solver.attempts(),
        elapsed_seconds: solver.elapsed_seconds(),
    }
}

#[tauri::command]
pub fn list_problems(
    state: State<'_, AppState>,
    category: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<ProblemSummary>, AppError> {
    let conn = state.db.lock().unwrap();
    problem::list_problems(&conn, category.as_deref(), limit)
}

#[tauri::command]
pub fn start_problem(
    state: State<'_, AppState>,
    problem_id: i64,
) -> Result<ProblemState, AppError> {
    let conn = state.db.lock().unwrap();
    let prob = problem::get_problem(&conn, problem_id)?;
    drop(conn);

    let session = SolverSession::new(&prob).map_err(AppError::Other)?;
    let ps = solver_to_problem_state(&session);

    let mut solver = state.solver.lock().unwrap();
    *solver = Some(session);

    Ok(ps)
}

#[tauri::command]
pub fn solve_move(
    state: State<'_, AppState>,
    row: u8,
    col: u8,
) -> Result<SolveMoveResult, AppError> {
    let mut solver_lock = state.solver.lock().unwrap();
    let solver = solver_lock
        .as_mut()
        .ok_or_else(|| AppError::Other("no active problem".into()))?;

    let point = Point::new(row, col);
    let move_result = solver.try_move(point);
    let board_state = solver.game().to_state();
    let status = solver.status();

    // If solved or failed, record the attempt
    if status == SolveStatus::Solved || status == SolveStatus::Failed {
        let problem_id = solver.problem_id();
        let hints_used = solver.hints_used();
        let attempts = solver.attempts();
        let elapsed = solver.elapsed_seconds();
        let status_str = match status {
            SolveStatus::Solved => "solved",
            SolveStatus::Failed => "failed",
            SolveStatus::InProgress => "in_progress",
        };
        drop(solver_lock);
        record_attempt(&state, problem_id, status_str, hints_used, attempts, elapsed);
    }

    Ok(SolveMoveResult {
        move_result,
        board_state,
        status,
    })
}

#[tauri::command]
pub fn get_hint(state: State<'_, AppState>, level: String) -> Result<HintData, AppError> {
    let mut solver_lock = state.solver.lock().unwrap();
    let solver = solver_lock
        .as_mut()
        .ok_or_else(|| AppError::Other("no active problem".into()))?;

    let hint_level = match level.as_str() {
        "Area" => HintLevel::Area,
        "Candidates" => HintLevel::Candidates,
        "Answer" => HintLevel::Answer,
        _ => return Err(AppError::Other("invalid hint level".into())),
    };

    Ok(solver.get_hint(hint_level))
}

#[tauri::command]
pub fn skip_problem(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut solver_lock = state.solver.lock().unwrap();
    let solver = solver_lock
        .as_mut()
        .ok_or_else(|| AppError::Other("no active problem".into()))?;

    solver.mark_failed();
    let problem_id = solver.problem_id();
    let hints_used = solver.hints_used();
    let attempts = solver.attempts();
    let elapsed = solver.elapsed_seconds();
    drop(solver_lock);

    record_attempt(&state, problem_id, "failed", hints_used, attempts, elapsed);

    // Clear the solver
    let mut solver_lock = state.solver.lock().unwrap();
    *solver_lock = None;

    Ok(())
}

#[tauri::command]
pub fn get_problem_state(state: State<'_, AppState>) -> Result<Option<ProblemState>, AppError> {
    let solver_lock = state.solver.lock().unwrap();
    Ok(solver_lock.as_ref().map(solver_to_problem_state))
}

#[tauri::command]
pub async fn generate_problems_from_game(
    state: State<'_, AppState>,
    threshold: Option<f64>,
) -> Result<u32, AppError> {
    let threshold = threshold.unwrap_or(3.0);

    // Get the review session data
    let review_lock = state.review.lock().await;
    let review = review_lock
        .as_ref()
        .ok_or_else(|| AppError::Other("no active review session".into()))?;

    if !review.is_complete {
        return Err(AppError::Other("review is not yet complete".into()));
    }

    let game_sgf = review.game_sgf.clone();
    let board_size = review.board_size;

    // Collect completed analyses
    let analyses: Vec<_> = review.results.iter().filter_map(|r| r.clone()).collect();
    drop(review_lock);

    // Look up game_id from saved games (most recent with matching SGF)
    let game_id = {
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT id FROM games ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
    };

    // Generate problems
    let problems = crate::generate::generate_from_review(
        &game_sgf,
        board_size,
        &analyses,
        game_id,
        threshold,
    )
    .map_err(AppError::Other)?;

    let count = problems.len() as u32;

    // Insert into DB
    let conn = state.db.lock().unwrap();
    for prob in &problems {
        problem::insert_problem(&conn, prob)?;
    }

    Ok(count)
}

fn record_attempt(
    state: &AppState,
    problem_id: i64,
    status: &str,
    hints_used: u8,
    attempts: u16,
    time_seconds: u64,
) {
    if let Ok(conn) = state.db.lock() {
        // Record the attempt
        let _ = conn.execute(
            "INSERT INTO problem_attempts (problem_id, status, hints_used, attempts, time_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                problem_id,
                status,
                hints_used as i32,
                attempts as i32,
                time_seconds as i64,
            ],
        );

        // Update SRS card
        let solved = status == "solved";
        let rating = srs::solve_to_rating(solved, hints_used);
        let _ = srs::update_card(&conn, problem_id, rating);

        // Update skill profile
        if let Ok(prob) = problem::get_problem(&conn, problem_id) {
            let dimension = prob.category.to_dimension();
            let _ = crate::skill::update_skill_after_problem(
                &conn, dimension, solved, prob.difficulty,
            );
        }
    }
}

#[tauri::command]
pub fn get_recommended_problem(
    state: State<'_, AppState>,
) -> Result<ProblemState, AppError> {
    let conn = state.db.lock().unwrap();
    let profile = crate::skill::get_skill_profile(&conn)?;
    let problem_id = problem::select_next_problem(&conn, &profile)?;
    let prob = problem::get_problem(&conn, problem_id)?;
    drop(conn);

    let session = SolverSession::new(&prob).map_err(AppError::Other)?;
    let ps = solver_to_problem_state(&session);

    let mut solver = state.solver.lock().unwrap();
    *solver = Some(session);

    Ok(ps)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemStats {
    pub total_solved: u32,
    pub total_attempted: u32,
    pub accuracy_percent: f64,
    pub per_category: Vec<CategoryStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStat {
    pub category: String,
    pub solved: u32,
    pub attempted: u32,
}

#[tauri::command]
pub fn get_problem_stats(
    state: State<'_, AppState>,
) -> Result<ProblemStats, AppError> {
    let conn = state.db.lock().unwrap();

    let total_solved: u32 = conn.query_row(
        "SELECT COUNT(*) FROM problem_attempts WHERE status = 'solved'",
        [],
        |row| row.get(0),
    )?;

    let total_attempted: u32 = conn.query_row(
        "SELECT COUNT(*) FROM problem_attempts",
        [],
        |row| row.get(0),
    )?;

    let accuracy_percent = if total_attempted > 0 {
        (total_solved as f64 / total_attempted as f64) * 100.0
    } else {
        0.0
    };

    // Per-category breakdown
    let mut stmt = conn.prepare(
        "SELECT p.category,
                SUM(CASE WHEN pa.status = 'solved' THEN 1 ELSE 0 END) as solved,
                COUNT(*) as attempted
         FROM problem_attempts pa
         JOIN problems p ON pa.problem_id = p.id
         GROUP BY p.category",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(CategoryStat {
            category: row.get(0)?,
            solved: row.get(1)?,
            attempted: row.get(2)?,
        })
    })?;

    let mut per_category = Vec::new();
    for row in rows {
        per_category.push(row?);
    }

    Ok(ProblemStats {
        total_solved,
        total_attempted,
        accuracy_percent,
        per_category,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProblemResult {
    pub imported: u32,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn import_problems_from_sgf(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Option<ImportProblemResult>, AppError> {
    use tauri_plugin_dialog::DialogExt;

    let paths = app
        .dialog()
        .file()
        .add_filter("SGF Files", &["sgf"])
        .blocking_pick_files();

    let Some(paths) = paths else {
        return Ok(None); // User cancelled
    };

    let mut all_problems = Vec::new();
    let mut all_errors = Vec::new();

    // Phase 1: Read and parse all files without holding the DB lock
    for file_path in paths {
        let path_buf = match file_path.as_path() {
            Some(p) => p.to_path_buf(),
            None => {
                all_errors.push("Invalid file path".to_string());
                continue;
            }
        };

        let sgf_text = match std::fs::read_to_string(&path_buf) {
            Ok(text) => text,
            Err(e) => {
                all_errors.push(format!("{}: {e}", path_buf.display()));
                continue;
            }
        };

        let result = import::import_from_sgf(&sgf_text);
        all_errors.extend(
            result
                .errors
                .into_iter()
                .map(|e| format!("{}: {e}", path_buf.display())),
        );
        all_problems.extend(result.problems);
    }

    // Phase 2: Batch insert within a single transaction
    let mut total_imported = 0u32;
    let conn = state.db.lock().unwrap();
    let _ = conn.execute("BEGIN", []);
    for prob in &all_problems {
        match problem::insert_problem(&conn, prob) {
            Ok(_id) => total_imported += 1,
            Err(e) => all_errors.push(format!("DB insert: {e}")),
        }
    }
    let _ = conn.execute("COMMIT", []);

    Ok(Some(ImportProblemResult {
        imported: total_imported,
        errors: all_errors,
    }))
}
