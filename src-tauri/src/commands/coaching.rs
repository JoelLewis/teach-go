use gosensei_coaching::types::CoachingMessage;
use gosensei_coaching::{classify, delta, templates};
use gosensei_core::types::Move;
use tauri::State;
use tracing::info;

use crate::convert;
use crate::error::AppError;
use crate::state::AppState;

const COACHING_VISITS: u32 = 100;

#[tauri::command]
pub async fn get_coaching_feedback(
    state: State<'_, AppState>,
) -> Result<Option<CoachingMessage>, AppError> {
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
    let query = convert::build_query(query_id, pre_move_history, board_size, komi, COACHING_VISITS, None);

    let response = client.query(query).await?;
    drop(katago);

    // Compute delta between best move and played move
    let player_move_gtp = convert::point_to_gtp(point, board_size);
    let score_loss = delta::score_loss(&response, &player_move_gtp);
    let severity = delta::classify_severity(score_loss, true); // DDK default

    // Skip feedback for good moves to avoid noise
    if severity == gosensei_coaching::types::Severity::Good {
        return Ok(None);
    }

    let error_class = classify::classify_error(move_number, board_size, point.row, point.col, score_loss);
    let suggested = response.move_infos.first().map(|m| m.mv.clone());

    let message = templates::generate_message(severity, error_class, score_loss, suggested, move_number);

    info!(
        "Coaching move {move_number}: {severity:?} (loss: {score_loss:.1}pt) — {}",
        message.message
    );

    Ok(Some(message))
}
