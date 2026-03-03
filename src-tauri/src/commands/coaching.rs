use gosensei_coaching::types::{CoachingMessage, Severity};
use gosensei_coaching::{classify, delta, simplest, templates};
use gosensei_core::types::Move;
use tauri::State;
use tracing::info;

#[cfg(feature = "llm")]
use tracing::warn;

use crate::convert;
use crate::error::AppError;
use crate::skill;
use crate::state::AppState;

#[cfg(feature = "llm")]
use gosensei_katago::protocol::AnalysisResponse;

use tauri::AppHandle;

#[cfg(feature = "llm")]
use tauri::Emitter;

const COACHING_VISITS: u32 = 100;
const SIMPLEST_MOVE_SCORE_GAP: f64 = 1.0;
const SIMPLEST_MOVE_RANK_THRESHOLD: f64 = 10.0;

#[tauri::command]
pub async fn get_coaching_feedback(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<Option<CoachingMessage>, AppError> {
    let _ = &app; // used conditionally by llm feature

    // Extract game data under a brief std mutex lock
    let (history, board_size, komi) = {
        let game_lock = state.game.lock().unwrap();
        let game = game_lock
            .as_ref()
            .ok_or(AppError::Other("No active game".into()))?;
        (
            game.move_history().to_vec(),
            game.board().dimension(),
            game.komi(),
        )
    };

    // Read player rank from skill profile for rank-aware thresholds
    let player_rank = skill::get_player_rank(&state);

    // Need at least one move to analyze
    let last_record = match history.last() {
        Some(r) => r,
        None => return Ok(None),
    };

    // Only analyze Play moves — skip Pass/Resign
    let point = match last_record.mv {
        Move::Play(p) => p,
        Move::Pass | Move::Resign => return Ok(None),
    };
    let move_number = last_record.move_number;

    // Check if KataGo is available — graceful degradation
    let katago = state.katago.lock().await;
    let client = match katago.as_ref() {
        Some(c) => c,
        None => return Ok(None),
    };

    // Build pre-move query (history excluding the last move)
    let pre_move_history = &history[..history.len() - 1];
    let query_id = format!("coaching-{move_number}");
    let standard_query = convert::build_query(
        query_id, pre_move_history, board_size, komi, COACHING_VISITS, None, None,
    );

    // Fire standard analysis query
    let standard_rx = client.query_fire(standard_query).await?;

    // Pipeline Human SL queries only when LLM is compiled in and loaded
    #[cfg(feature = "llm")]
    let (human_at_rank_rx, human_one_up_rx) = {
        let llm_loaded = state.llm.try_lock().map(|g| g.is_some()).unwrap_or(false);
        if llm_loaded {
            let at_rank_profile = convert::rank_to_human_profile(player_rank);
            let one_up_profile = convert::rank_one_up_profile(player_rank);
            let q1 = convert::build_query(
                format!("coaching-{move_number}-hsl"), pre_move_history, board_size, komi,
                1, Some(at_rank_profile), None,
            );
            let q2 = convert::build_query(
                format!("coaching-{move_number}-hsl-up"), pre_move_history, board_size, komi,
                1, Some(one_up_profile), None,
            );
            (client.query_fire(q1).await.ok(), client.query_fire(q2).await.ok())
        } else {
            (None, None)
        }
    };

    let response = standard_rx.await.map_err(|_| AppError::KataGo("coaching query dropped".into()))?;

    // Await Human SL responses concurrently (non-fatal)
    #[cfg(feature = "llm")]
    let (human_at_rank, human_one_up): (Option<AnalysisResponse>, Option<AnalysisResponse>) = {
        let (at_rank, one_up) = tokio::join!(
            async { match human_at_rank_rx { Some(rx) => rx.await.ok(), None => None } },
            async { match human_one_up_rx { Some(rx) => rx.await.ok(), None => None } },
        );
        (at_rank, one_up)
    };

    drop(katago);

    // Compute delta between best move and played move
    let player_move_gtp = convert::point_to_gtp(point, board_size);
    let score_loss = delta::score_loss(&response, &player_move_gtp);
    let severity = delta::classify_severity(score_loss, player_rank);

    // Extract human policy signals (only needed for LLM coaching)
    #[cfg(feature = "llm")]
    let (human_policy_played_at_rank, human_policy_played_one_up, human_policy_best_at_rank) = {
        let best_move_gtp = response.move_infos.first().map(|m| m.mv.as_str());
        let at_rank = human_at_rank.as_ref()
            .and_then(|r| r.move_infos.iter().find(|m| m.mv == player_move_gtp))
            .map(|m| m.prior);
        let one_up = human_one_up.as_ref()
            .and_then(|r| r.move_infos.iter().find(|m| m.mv == player_move_gtp))
            .map(|m| m.prior);
        let best_at_rank = best_move_gtp.and_then(|best| {
            human_at_rank.as_ref()
                .and_then(|r| r.move_infos.iter().find(|m| m.mv == best))
                .map(|m| m.prior)
        });
        (at_rank, one_up, best_at_rank)
    };

    // For excellent moves, occasionally return praise
    if severity == Severity::Excellent {
        let praise = templates::maybe_praise(move_number, &player_move_gtp, &response.move_infos, score_loss);
        if let Some(ref msg) = praise {
            info!("Coaching move {move_number}: Excellent — {}", msg.message);
        }
        return Ok(praise);
    }

    // Skip feedback for good moves to avoid noise
    if severity == Severity::Good {
        return Ok(None);
    }

    // Extract best move coordinates and PV for richer classification
    let best_move_info = response.move_infos.first();
    let (best_row, best_col) = best_move_info
        .and_then(|m| convert::gtp_to_point(&m.mv, board_size))
        .map(|p| (p.row, p.col))
        .unwrap_or((point.row, point.col));
    let pv_length = best_move_info.map(|m| m.pv.len()).unwrap_or(0);

    // Convert ownership from f32 to f64 for the classifier
    let ownership_f64: Vec<f64> = response.ownership.iter().map(|&v| v as f64).collect();
    let ownership_ref = if ownership_f64.is_empty() {
        None
    } else {
        Some(ownership_f64.as_slice())
    };

    let error_class = classify::classify_error(&classify::ClassifyInput {
        move_number,
        board_size,
        player_row: point.row,
        player_col: point.col,
        best_row,
        best_col,
        pv_length,
        ownership: ownership_ref,
        score_loss,
    });
    let suggested = best_move_info.map(|m| m.mv.clone());

    // For DDK players (rank >= 10 kyu), suggest the simplest good move instead of the absolute best
    let simplest_move = if player_rank >= SIMPLEST_MOVE_RANK_THRESHOLD {
        simplest::find_simplest_good_move(&response.move_infos, SIMPLEST_MOVE_SCORE_GAP)
            .map(|m| m.mv.clone())
    } else {
        None
    };

    // Accumulate error for skill model update at game end
    if let Some(ec) = error_class {
        state.game_errors.lock().unwrap().push(
            crate::skill::GameError { error_class: ec, score_loss },
        );
    }

    // Always compute the template message first (zero-latency fallback)
    let template_msg = templates::generate_message(severity, error_class, score_loss, suggested.clone(), simplest_move.clone(), move_number);

    // Try LLM coaching if available, fall back to template on any failure
    #[allow(unused_mut)]
    let mut llm_used = false;

    #[cfg(feature = "llm")]
    let final_message = {
        match try_llm_coaching(
            &state, &app, player_rank, move_number, &player_move_gtp,
            suggested.as_deref(), simplest_move.as_deref(),
            score_loss, severity, error_class, &response,
            human_policy_played_at_rank, human_policy_played_one_up,
            human_policy_best_at_rank,
        ).await {
            Ok(msg) => {
                llm_used = true;
                msg
            }
            Err(e) => {
                warn!("LLM coaching fallback: {e}");
                template_msg
            }
        }
    };

    #[cfg(not(feature = "llm"))]
    let final_message = template_msg;

    // Record coaching event in DB for session context
    {
        let db = state.db.lock().unwrap();
        let _ = crate::coaching_db::insert_event(&db, &crate::coaching_db::CoachingEvent {
            move_number,
            error_class: error_class.map(|ec| format!("{ec:?}")),
            severity: format!("{severity:?}"),
            score_loss,
            llm_used,
        });
    }

    info!(
        "Coaching move {move_number}: {severity:?} (loss: {score_loss:.1}pt) — {}",
        final_message.message
    );

    Ok(Some(final_message))
}

#[cfg(feature = "llm")]
#[derive(Clone, serde::Serialize)]
struct CoachingStreamChunk {
    move_number: u16,
    text_delta: String,
    is_complete: bool,
}

#[cfg(feature = "llm")]
#[allow(clippy::too_many_arguments)]
async fn try_llm_coaching(
    state: &State<'_, AppState>,
    app: &AppHandle,
    player_rank: f64,
    move_number: u16,
    player_move_gtp: &str,
    suggested: Option<&str>,
    simplest_move: Option<&str>,
    score_loss: f64,
    severity: Severity,
    error_class: Option<gosensei_coaching::types::ErrorClass>,
    response: &gosensei_katago::protocol::AnalysisResponse,
    human_policy_played_at_rank: Option<f64>,
    human_policy_played_one_up: Option<f64>,
    human_policy_best_at_rank: Option<f64>,
) -> Result<CoachingMessage, AppError> {
    // Check if LLM is loaded — clone the Arc-wrapped manager and drop the guard
    let manager = {
        let llm_guard = state.llm.lock().await;
        llm_guard
            .as_ref()
            .ok_or(AppError::Llm("model not loaded".into()))?
            .clone()
    };

    // Build session context from DB
    let (similar_errors, total_mistakes) = {
        let db = state.db.lock().unwrap();
        let similar = error_class
            .map(|ec| crate::coaching_db::count_class_this_session(&db, &format!("{ec:?}")).unwrap_or(0))
            .unwrap_or(0);
        let total = crate::coaching_db::count_mistakes_this_session(&db).unwrap_or(0);
        (similar, total)
    };

    // Determine best/simplest move for the prompt
    let best_or_simplest = simplest_move
        .or(suggested)
        .unwrap_or("unknown")
        .to_string();

    // Build PV from best move
    let pv_best: Vec<String> = response
        .move_infos
        .first()
        .map(|m| m.pv.clone())
        .unwrap_or_default();

    let payload = gosensei_llm::CoachingPayload {
        player_rank: gosensei_llm::prompt::rank_to_display(player_rank),
        move_number,
        played: player_move_gtp.to_string(),
        best_or_simplest,
        score_loss,
        severity: format!("{severity:?}"),
        error_class_hint: error_class.map(|ec| format!("{ec:?}")),
        pv_best,
        session_context: gosensei_llm::SessionContext {
            similar_errors_this_game: similar_errors,
            total_mistakes_this_game: total_mistakes,
        },
        human_policy_played_at_rank,
        human_policy_played_one_up,
        human_policy_best_at_rank,
    };

    let user_prompt = gosensei_llm::prompt::build_user_prompt(&payload);

    // Apply chat template and generate
    let prompt = manager
        .apply_chat_template(gosensei_llm::prompt::SYSTEM_PROMPT, &user_prompt)
        .map_err(|e| AppError::Llm(e.to_string()))?;

    let app_for_stream = app.clone();
    let mn = move_number;

    // Run generation in blocking task (LlamaContext is !Send)
    let raw_output = tokio::task::spawn_blocking(move || {
        manager.generate_streaming(&prompt, 150, |piece| {
            let _ = app_for_stream.emit("coaching-stream", CoachingStreamChunk {
                move_number: mn,
                text_delta: piece.to_string(),
                is_complete: false,
            });
        })
    })
    .await
    .map_err(|e| AppError::Llm(format!("task join: {e}")))?
    .map_err(|e| AppError::Llm(e.to_string()))?;

    // Signal stream completion
    let _ = app.emit("coaching-stream", CoachingStreamChunk {
        move_number,
        text_delta: String::new(),
        is_complete: true,
    });

    // Parse LLM output
    let parsed = gosensei_llm::parse::parse_llm_output(&raw_output);

    Ok(CoachingMessage {
        severity,
        error_class: error_class.or_else(|| {
            parsed.error_class.as_deref()
                .and_then(gosensei_llm::parse::validate_error_class)
                .and_then(|name| name.parse().ok())
        }),
        message: parsed.coaching_text,
        suggested_move: suggested.map(String::from),
        simplest_move: simplest_move.map(String::from),
        score_loss,
        move_number,
    })
}
