use std::time::Duration;

use gosensei_coaching::types::Severity;
use gosensei_coaching::{classify, templates};
use gosensei_core::game::Game;
use gosensei_core::types::Move;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tracing::{info, warn};

use crate::convert;
use crate::error::AppError;
use crate::review::{MoveAnalysis, ReviewData, ReviewSession};
use crate::skill;
use crate::state::AppState;

const REVIEW_VISITS: u32 = 50;
const BATCH_SIZE: usize = 20;
const QUERY_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize)]
pub struct ReviewProgress {
    total_positions: u16,
    analyzed_positions: u16,
    is_complete: bool,
}

#[tauri::command]
pub async fn start_review(
    state: State<'_, AppState>,
    app: AppHandle,
    game_id: Option<i64>,
) -> Result<(), AppError> {
    // Load SGF — from saved game or active game
    let sgf = if let Some(id) = game_id {
        let db = state.db.lock().unwrap();
        db.query_row("SELECT sgf FROM games WHERE id = ?1", [id], |row| {
            row.get::<_, String>(0)
        })?
    } else {
        let game_lock = state.game.lock().unwrap();
        let game = game_lock
            .as_ref()
            .ok_or(AppError::Other("No active game".into()))?;
        game.to_sgf()
    };

    // Parse the game to get move history
    let game = Game::from_sgf(&sgf).map_err(AppError::Other)?;
    let history = game.move_history().to_vec();
    let board_size = game.board().dimension();
    let komi = game.komi();
    let total_moves = history.len() as u16;
    // Analyze positions 0..=total_moves (initial position + each move)
    let total_positions = total_moves + 1;

    // Initialize review session
    {
        let mut review = state.review.lock().await;
        *review = Some(ReviewSession {
            game_sgf: sgf,
            board_size,
            komi,
            total_positions,
            results: vec![None; total_positions as usize],
            ownership: vec![None; total_positions as usize],
            is_complete: false,
        });
    }

    // Read player rank for rank-aware severity thresholds
    let player_rank = skill::get_player_rank(&state);

    info!(
        "Starting review: {total_moves} moves, {total_positions} positions to analyze (player rank: {player_rank:.1})"
    );

    // Clone Arcs for the spawned task
    let katago = state.katago.clone();
    let review_state = state.review.clone();

    // Spawn async analysis task
    tokio::spawn(async move {
        let mut analyzed = 0u16;

        // Process in batches
        for batch_start in (0..total_positions as usize).step_by(BATCH_SIZE) {
            let batch_end = (batch_start + BATCH_SIZE).min(total_positions as usize);

            // Fire all queries in this batch
            let mut receivers = Vec::new();

            {
                let katago_lock = katago.lock().await;
                let client = match katago_lock.as_ref() {
                    Some(c) => c,
                    None => {
                        warn!("KataGo not available for review");
                        break;
                    }
                };

                for i in batch_start..batch_end {
                    let query_id = format!("review-{i}");
                    // Build query with moves up to position i
                    let moves_slice = &history[..i];
                    let query = convert::build_query(
                        query_id,
                        moves_slice,
                        board_size,
                        komi,
                        REVIEW_VISITS,
                        None, // Full strength for review analysis
                        Some(true), // Request ownership data for territory overlay
                    );

                    match client.query_fire(query).await {
                        Ok(rx) => receivers.push((i, rx)),
                        Err(e) => {
                            warn!("Failed to send review query {i}: {e}");
                            continue;
                        }
                    }
                }
            } // Drop katago lock

            // Collect responses for this batch
            for (i, rx) in receivers {
                let response = match tokio::time::timeout(QUERY_TIMEOUT, rx).await {
                    Ok(Ok(resp)) => resp,
                    Ok(Err(_)) => {
                        warn!("Review query {i} channel closed");
                        continue;
                    }
                    Err(_) => {
                        warn!("Review query {i} timed out");
                        continue;
                    }
                };

                // KataGo reports winrate from the perspective of the current player.
                // Position i: after i moves. Black plays odd-indexed positions (moves 1,3,5...)
                // Position 0 = Black to move, position 1 = White to move, etc.
                let is_black_to_move = i .is_multiple_of(2);
                let winrate_black = if is_black_to_move {
                    response.root_info.winrate
                } else {
                    1.0 - response.root_info.winrate
                };
                let score_lead = if is_black_to_move {
                    response.root_info.score_lead
                } else {
                    -response.root_info.score_lead
                };

                // Determine the played move at this position (the move that created position i)
                let (color, player_move_gtp) = if i > 0 {
                    let record = &history[i - 1];
                    let color_str = record.color.as_str();
                    let gtp = match record.mv {
                        Move::Play(p) => Some(convert::point_to_gtp(p, board_size)),
                        Move::Pass => Some("pass".to_string()),
                        Move::Resign => None,
                    };
                    (Some(color_str.to_string()), gtp)
                } else {
                    (None, None)
                };

                let best_move = response.move_infos.first().map(|m| m.mv.clone());
                let best_variation = response
                    .move_infos
                    .first()
                    .map(|m| m.pv.clone())
                    .unwrap_or_default();

                // Score loss will be computed after all positions are analyzed
                // (comparing consecutive positions). For now, store raw data.
                let analysis = MoveAnalysis {
                    move_number: i as u16,
                    color,
                    player_move: player_move_gtp,
                    winrate_black,
                    score_lead,
                    best_move,
                    score_loss: 0.0, // Computed later
                    severity: Severity::Good, // Computed later
                    coaching_message: None, // Computed later
                    best_variation,
                };

                // Normalize ownership to Black's perspective (same convention as winrate)
                let normalized_ownership: Vec<f32> = if !response.ownership.is_empty() {
                    if is_black_to_move {
                        response.ownership.clone()
                    } else {
                        response.ownership.iter().map(|v| -v).collect()
                    }
                } else {
                    vec![]
                };

                // Store result and ownership
                {
                    let mut review = review_state.lock().await;
                    if let Some(session) = review.as_mut() {
                        session.results[i] = Some(analysis);
                        if !normalized_ownership.is_empty() {
                            session.ownership[i] = Some(normalized_ownership);
                        }
                    }
                }

                analyzed += 1;
            }

            // Emit progress
            let progress = ReviewProgress {
                total_positions,
                analyzed_positions: analyzed,
                is_complete: false,
            };
            let _ = app.emit("review-progress", &progress);
        }

        // Post-processing: compute score loss from consecutive positions
        {
            let mut review = review_state.lock().await;
            if let Some(session) = review.as_mut() {
                compute_score_loss_and_severity(session, board_size, player_rank);
                session.is_complete = true;
            }
        }

        // Emit completion
        let _ = app.emit(
            "review-progress",
            &ReviewProgress {
                total_positions,
                analyzed_positions: total_positions,
                is_complete: true,
            },
        );

        info!("Review analysis complete: {total_positions} positions analyzed");
    });

    Ok(())
}

/// Compute score loss by comparing each position to the previous one.
/// A good move maintains or improves the position; a bad move loses score.
fn compute_score_loss_and_severity(session: &mut ReviewSession, board_size: u8, player_rank: f64) {
    let results = &mut session.results;

    for i in 1..results.len() {
        // Get score_lead of the previous position (what the engine expected)
        // and the current position (what actually happened after the move)
        let prev_score = results[i - 1]
            .as_ref()
            .map(|a| a.score_lead)
            .unwrap_or(0.0);
        let curr_score = results[i]
            .as_ref()
            .map(|a| a.score_lead)
            .unwrap_or(0.0);

        // Determine who played this move
        let is_black_move = results[i]
            .as_ref()
            .and_then(|a| a.color.as_deref())
            .map(|c| c == "black")
            .unwrap_or(false);

        // Score loss: how much did this player's move hurt their position?
        // For Black: prev_score was positive (Black leads), if curr_score is less, Black lost.
        // For White: prev_score was negative (Black's perspective), if curr_score is more
        //   positive, White lost ground.
        let score_loss = if is_black_move {
            (prev_score - curr_score).max(0.0)
        } else {
            (curr_score - prev_score).max(0.0)
        };

        let severity = Severity::from_score_loss(score_loss, player_rank);

        // Generate coaching message for non-good moves
        let coaching_message = if !matches!(severity, Severity::Excellent | Severity::Good) {
            // Try to classify the error type using move coordinates
            let error_class = results[i]
                .as_ref()
                .and_then(|a| a.player_move.as_deref())
                .and_then(|gtp| convert::gtp_to_point(gtp, board_size))
                .map(|point| classify::classify_error(i as u16, board_size, point.row, point.col, score_loss))
                .unwrap_or(None);

            let suggested = results[i].as_ref().and_then(|a| a.best_move.clone());
            let msg = templates::generate_message(severity, error_class, score_loss, suggested, None, i as u16);
            Some(msg.message)
        } else {
            None
        };

        if let Some(analysis) = results[i].as_mut() {
            analysis.score_loss = score_loss;
            analysis.severity = severity;
            analysis.coaching_message = coaching_message;
        }
    }
}

#[tauri::command]
pub async fn get_review_progress(
    state: State<'_, AppState>,
) -> Result<ReviewProgress, AppError> {
    let review = state.review.lock().await;
    let session = review
        .as_ref()
        .ok_or(AppError::Other("No review in progress".into()))?;

    let analyzed = session.results.iter().filter(|r| r.is_some()).count() as u16;

    Ok(ReviewProgress {
        total_positions: session.total_positions,
        analyzed_positions: analyzed,
        is_complete: session.is_complete,
    })
}

#[tauri::command]
pub async fn get_review_data(
    state: State<'_, AppState>,
) -> Result<ReviewData, AppError> {
    let review = state.review.lock().await;
    let session = review
        .as_ref()
        .ok_or(AppError::Other("No review in progress".into()))?;

    if !session.is_complete {
        return Err(AppError::Other("Review not yet complete".into()));
    }

    let move_analyses: Vec<MoveAnalysis> = session
        .results
        .iter()
        .filter_map(|r| r.clone())
        .collect();

    // Find top 5 mistakes sorted by score_loss descending (skip position 0)
    let mut scored: Vec<(u16, f64)> = move_analyses
        .iter()
        .filter(|a| a.move_number > 0 && a.score_loss > 0.0)
        .map(|a| (a.move_number, a.score_loss))
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let top_mistakes: Vec<u16> = scored.into_iter().take(5).map(|(n, _)| n).collect();

    Ok(ReviewData {
        board_size: session.board_size,
        total_moves: session.total_positions - 1,
        komi: session.komi,
        move_analyses,
        top_mistakes,
    })
}

#[tauri::command]
pub async fn get_review_position(
    state: State<'_, AppState>,
    move_number: u16,
) -> Result<gosensei_core::game::GameState, AppError> {
    let review = state.review.lock().await;
    let session = review
        .as_ref()
        .ok_or(AppError::Other("No review in progress".into()))?;

    let game = Game::from_sgf_partial(&session.game_sgf, Some(move_number))
        .map_err(AppError::Other)?;

    Ok(game.to_state())
}

#[tauri::command]
pub async fn get_ownership_at(
    state: State<'_, AppState>,
    move_number: u16,
) -> Result<Option<Vec<f32>>, AppError> {
    let review = state.review.lock().await;
    let session = review
        .as_ref()
        .ok_or(AppError::Other("No review in progress".into()))?;
    let idx = move_number as usize;
    if idx >= session.ownership.len() {
        return Err(AppError::Other("Move number out of range".into()));
    }
    Ok(session.ownership[idx].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winrate_normalization_black_to_move() {
        // Position 0 (Black to move): KataGo reports 0.6 from Black's perspective
        // After normalization: should stay 0.6
        let position: usize = 0;
        let is_black_to_move = position .is_multiple_of(2);
        let katago_winrate: f64 = 0.6;
        let winrate_black: f64 = if is_black_to_move {
            katago_winrate
        } else {
            1.0 - katago_winrate
        };
        assert!((winrate_black - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn winrate_normalization_white_to_move() {
        // Position 1 (White to move): KataGo reports 0.55 from White's perspective
        // After normalization to Black: 1.0 - 0.55 = 0.45
        let position: usize = 1;
        let is_black_to_move = position .is_multiple_of(2);
        let katago_winrate: f64 = 0.55;
        let winrate_black: f64 = if is_black_to_move {
            katago_winrate
        } else {
            1.0 - katago_winrate
        };
        assert!((winrate_black - 0.45).abs() < f64::EPSILON);
    }

    #[test]
    fn score_loss_computation() {
        let mut session = ReviewSession {
            game_sgf: String::new(),
            board_size: 9,
            komi: 6.5,
            total_positions: 3,
            ownership: vec![None; 3],
            results: vec![
                Some(MoveAnalysis {
                    move_number: 0,
                    color: None,
                    player_move: None,
                    winrate_black: 0.5,
                    score_lead: 0.0,
                    best_move: Some("E5".into()),
                    score_loss: 0.0,
                    severity: Severity::Good,
                    coaching_message: None,
                    best_variation: vec![],
                }),
                Some(MoveAnalysis {
                    move_number: 1,
                    color: Some("black".into()),
                    player_move: Some("A1".into()),
                    winrate_black: 0.45,
                    score_lead: -3.0, // Black played badly, now behind
                    best_move: Some("E5".into()),
                    score_loss: 0.0,
                    severity: Severity::Good,
                    coaching_message: None,
                    best_variation: vec![],
                }),
                Some(MoveAnalysis {
                    move_number: 2,
                    color: Some("white".into()),
                    player_move: Some("E5".into()),
                    winrate_black: 0.44,
                    score_lead: -3.2, // White played well, maintained advantage
                    best_move: Some("E5".into()),
                    score_loss: 0.0,
                    severity: Severity::Good,
                    coaching_message: None,
                    best_variation: vec![],
                }),
            ],
            is_complete: false,
        };

        // Use beginner rank (25k) — same band as the old DDK thresholds
        compute_score_loss_and_severity(&mut session, 9, 25.0);

        // Position 0 stays at 0
        assert!((session.results[0].as_ref().unwrap().score_loss - 0.0).abs() < f64::EPSILON);

        // Position 1: Black moved. score_lead went from 0.0 to -3.0.
        // For Black: prev(0.0) - curr(-3.0) = 3.0 points lost
        // At rank 25.0 (beginner band): 3.0 < 5.0 threshold → Inaccuracy
        let loss1 = session.results[1].as_ref().unwrap().score_loss;
        assert!((loss1 - 3.0).abs() < f64::EPSILON);
        assert_eq!(session.results[1].as_ref().unwrap().severity, Severity::Inaccuracy);

        // Position 2: White moved. score_lead went from -3.0 to -3.2.
        // For White: curr(-3.2) - prev(-3.0) = -0.2, max(0) = 0.0 — excellent move
        let loss2 = session.results[2].as_ref().unwrap().score_loss;
        assert!(loss2 < 0.01);
    }

    #[test]
    fn ownership_normalization_black_to_move() {
        // Position 0 (even index) = Black to move: values stay as-is
        let raw_ownership = vec![0.8, -0.5, 0.0, 0.3];
        let is_black_to_move = true;
        let normalized: Vec<f32> = if is_black_to_move {
            raw_ownership.clone()
        } else {
            raw_ownership.iter().map(|v| -v).collect()
        };
        assert_eq!(normalized, vec![0.8, -0.5, 0.0, 0.3]);
    }

    #[test]
    fn ownership_normalization_white_to_move() {
        // Position 1 (odd index) = White to move: values negated
        let raw_ownership = vec![0.8, -0.5, 0.0, 0.3];
        let is_black_to_move = false;
        let normalized: Vec<f32> = if is_black_to_move {
            raw_ownership.clone()
        } else {
            raw_ownership.iter().map(|v| -v).collect()
        };
        assert_eq!(normalized, vec![-0.8, 0.5, -0.0, -0.3]);
    }

    #[test]
    fn top_mistakes_selection() {
        let analyses = vec![
            (1u16, 0.5),  // small
            (2, 8.0),     // big
            (3, 0.1),     // tiny
            (4, 15.0),    // huge
            (5, 3.0),     // medium
            (6, 12.0),    // large
            (7, 0.0),     // perfect
        ];

        let mut scored = analyses.clone();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top: Vec<u16> = scored.into_iter().take(5).map(|(n, _)| n).collect();

        assert_eq!(top, vec![4, 6, 2, 5, 1]);
    }
}
