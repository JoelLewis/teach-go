use serde::{Deserialize, Serialize};

/// Input payload for the LLM coaching prompt builder.
/// Constructed from KataGo analysis data and game context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachingPayload {
    pub player_rank: String,
    pub move_number: u16,
    pub played: String,
    pub best_or_simplest: String,
    pub score_loss: f64,
    pub severity: String,
    pub error_class_hint: Option<String>,
    pub pv_best: Vec<String>,
    pub session_context: SessionContext,
    /// Human SL policy probability that a player at this rank would play the move.
    pub human_policy_played_at_rank: Option<f64>,
    /// Human SL policy probability that a player one band stronger would play the move.
    pub human_policy_played_one_up: Option<f64>,
    /// Human SL policy probability that a player at this rank would play the best move.
    pub human_policy_best_at_rank: Option<f64>,
}

/// Tracks how many similar mistakes the player has made this game session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub similar_errors_this_game: u32,
    pub total_mistakes_this_game: u32,
}

/// Parsed output from the LLM response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmCoachingOutput {
    pub error_class: Option<String>,
    pub coaching_text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coaching_payload_json_roundtrip() {
        let payload = CoachingPayload {
            player_rank: "12k".to_string(),
            move_number: 42,
            played: "D4".to_string(),
            best_or_simplest: "Q16".to_string(),
            score_loss: 3.5,
            severity: "Mistake".to_string(),
            error_class_hint: Some("Direction".to_string()),
            pv_best: vec!["Q16".to_string(), "R14".to_string()],
            session_context: SessionContext {
                similar_errors_this_game: 1,
                total_mistakes_this_game: 3,
            },
            human_policy_played_at_rank: Some(0.12),
            human_policy_played_one_up: Some(0.05),
            human_policy_best_at_rank: Some(0.35),
        };

        let json = serde_json::to_string(&payload).unwrap();
        let parsed: CoachingPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.player_rank, "12k");
        assert_eq!(parsed.move_number, 42);
        assert_eq!(parsed.played, "D4");
        assert_eq!(parsed.score_loss, 3.5);
        assert_eq!(parsed.pv_best.len(), 2);
        assert_eq!(parsed.session_context.similar_errors_this_game, 1);
    }

    #[test]
    fn llm_output_parse_roundtrip() {
        let output = LlmCoachingOutput {
            error_class: Some("Direction".to_string()),
            coaching_text: "This move loses the initiative.".to_string(),
        };

        let json = serde_json::to_string(&output).unwrap();
        let parsed: LlmCoachingOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, output);
    }

    #[test]
    fn llm_output_none_error_class() {
        let output = LlmCoachingOutput {
            error_class: None,
            coaching_text: "Consider the whole board.".to_string(),
        };

        let json = serde_json::to_string(&output).unwrap();
        let parsed: LlmCoachingOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.error_class, None);
    }
}
