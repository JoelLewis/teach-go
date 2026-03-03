use crate::types::CoachingPayload;

pub const SYSTEM_PROMPT: &str = "\
You are GoSensei, a patient and encouraging Go (Weiqi/Baduk) coach. \
You speak directly to the student in 1-3 clear sentences. \
Adapt your language to their rank. \
Never use jargon above their level.

Output format:
<classification>{\"error_class\": \"Direction|Shape|Reading|LifeAndDeath|Endgame|Opening|Ko\"}</classification>
<coaching>Your 1-3 sentence coaching message here.</coaching>";

/// Build the user portion of the chat prompt from a coaching payload.
pub fn build_user_prompt(payload: &CoachingPayload) -> String {
    let mut parts = Vec::new();

    parts.push(format!(
        "Player rank: {}. Move {}: played {} (score loss: {:.1}pt, severity: {}).",
        payload.player_rank,
        payload.move_number,
        payload.played,
        payload.score_loss,
        payload.severity,
    ));

    parts.push(format!(
        "Best/simplest move was {}.",
        payload.best_or_simplest,
    ));

    if !payload.pv_best.is_empty() {
        parts.push(format!(
            "Best continuation: {}.",
            payload.pv_best.join(" → "),
        ));
    }

    if let Some(ref hint) = payload.error_class_hint {
        parts.push(format!("Heuristic error class: {hint}."));
    }

    // Human SL policy signals
    if let Some(p) = payload.human_policy_played_at_rank {
        parts.push(format!(
            "A typical {rank} player would play this move {pct:.0}% of the time.",
            rank = payload.player_rank,
            pct = p * 100.0,
        ));
    }
    if let Some(p) = payload.human_policy_played_one_up {
        parts.push(format!(
            "A player ~5 stones stronger would play this move {pct:.0}% of the time.",
            pct = p * 100.0,
        ));
    }
    if let Some(p) = payload.human_policy_best_at_rank {
        parts.push(format!(
            "A typical {rank} player would find the best move {pct:.0}% of the time.",
            rank = payload.player_rank,
            pct = p * 100.0,
        ));
    }

    // Session context
    let ctx = &payload.session_context;
    if ctx.total_mistakes_this_game > 0 {
        parts.push(format!(
            "This game: {} mistakes so far ({} similar).",
            ctx.total_mistakes_this_game, ctx.similar_errors_this_game,
        ));
    }

    parts.join(" ")
}

/// Convert a numeric rank (mu value) to a display string.
/// Values 1-30 represent kyu ranks (30k-1k), 0 and below represent dan.
pub fn rank_to_display(rank: f64) -> String {
    if rank >= 1.0 {
        format!("{}k", rank.round() as u32)
    } else if rank >= 0.0 {
        "1d".to_string()
    } else {
        format!("{}d", (1.0 - rank).round() as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SessionContext;

    fn sample_payload() -> CoachingPayload {
        CoachingPayload {
            player_rank: "12k".to_string(),
            move_number: 42,
            played: "D4".to_string(),
            best_or_simplest: "Q16".to_string(),
            score_loss: 3.5,
            severity: "Mistake".to_string(),
            error_class_hint: Some("Direction".to_string()),
            pv_best: vec!["Q16".to_string(), "R14".to_string(), "Q15".to_string()],
            session_context: SessionContext {
                similar_errors_this_game: 1,
                total_mistakes_this_game: 3,
            },
            human_policy_played_at_rank: Some(0.12),
            human_policy_played_one_up: Some(0.05),
            human_policy_best_at_rank: Some(0.35),
        }
    }

    #[test]
    fn build_user_prompt_includes_all_fields() {
        let prompt = build_user_prompt(&sample_payload());
        assert!(prompt.contains("12k"));
        assert!(prompt.contains("Move 42"));
        assert!(prompt.contains("D4"));
        assert!(prompt.contains("Q16"));
        assert!(prompt.contains("3.5pt"));
        assert!(prompt.contains("Mistake"));
        assert!(prompt.contains("Direction"));
        assert!(prompt.contains("Q16 → R14 → Q15"));
        assert!(prompt.contains("12%"));
        assert!(prompt.contains("5%"));
        assert!(prompt.contains("35%"));
        assert!(prompt.contains("3 mistakes"));
    }

    #[test]
    fn build_user_prompt_without_optional_fields() {
        let mut payload = sample_payload();
        payload.error_class_hint = None;
        payload.human_policy_played_at_rank = None;
        payload.human_policy_played_one_up = None;
        payload.human_policy_best_at_rank = None;
        payload.session_context.total_mistakes_this_game = 0;
        payload.pv_best.clear();

        let prompt = build_user_prompt(&payload);
        assert!(prompt.contains("D4"));
        assert!(!prompt.contains("Heuristic"));
        assert!(!prompt.contains("typical"));
        assert!(!prompt.contains("mistakes"));
        assert!(!prompt.contains("continuation"));
    }

    #[test]
    fn rank_to_display_beginner() {
        assert_eq!(rank_to_display(25.0), "25k");
        assert_eq!(rank_to_display(12.0), "12k");
    }

    #[test]
    fn rank_to_display_intermediate() {
        assert_eq!(rank_to_display(5.0), "5k");
        assert_eq!(rank_to_display(1.0), "1k");
    }

    #[test]
    fn rank_to_display_dan() {
        assert_eq!(rank_to_display(0.5), "1d");
        assert_eq!(rank_to_display(0.0), "1d");
        assert_eq!(rank_to_display(-1.0), "2d");
    }
}
