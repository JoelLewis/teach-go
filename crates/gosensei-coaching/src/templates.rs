use gosensei_katago::protocol::MoveInfo;

use crate::types::{CoachingMessage, ErrorClass, Severity};

/// Generate a coaching message from classification results.
pub fn generate_message(
    severity: Severity,
    error_class: Option<ErrorClass>,
    score_loss: f64,
    suggested_move: Option<String>,
    simplest_move: Option<String>,
    move_number: u16,
) -> CoachingMessage {
    let message = match (&severity, &error_class) {
        (Severity::Excellent, _) => "Excellent move! This matches the engine's top recommendation.".into(),
        (Severity::Good, _) => "Good move! This is close to what the engine recommends.".into(),
        // Direction
        (Severity::Inaccuracy, Some(ErrorClass::Direction)) => {
            format!(
                "This move is a small inaccuracy — the direction of play could be better. \
                 Consider the flow of the game and where the biggest areas are. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        (Severity::Mistake | Severity::Blunder, Some(ErrorClass::Direction)) => {
            format!(
                "The biggest action is elsewhere on the board. Look for the largest open area \
                 or the most urgent fight before playing locally. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        // Shape
        (Severity::Inaccuracy, Some(ErrorClass::Shape)) => {
            format!(
                "The shape here isn't quite right. A more efficient stone placement \
                 would keep your groups connected and strong. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        (Severity::Mistake | Severity::Blunder, Some(ErrorClass::Shape)) => {
            format!(
                "This creates an inefficient shape. Watch for empty triangles and \
                 overconcentrated formations — try to keep your stones working together. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        // Reading
        (_, Some(ErrorClass::Reading)) => {
            format!(
                "There's a deeper tactical sequence here that changes the outcome. \
                 Try reading a few moves ahead — the position is more complex than it appears. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        // Life & Death
        (Severity::Inaccuracy, Some(ErrorClass::LifeAndDeath)) => {
            format!(
                "This area involves life and death — be careful with your group's safety. \
                 Check whether your stones have enough room to make two eyes. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        (Severity::Mistake | Severity::Blunder, Some(ErrorClass::LifeAndDeath)) => {
            format!(
                "This is a life-and-death mistake — your group's safety is at risk! \
                 Look for moves that secure two eyes or threaten your opponent's eyespace. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        // Ko
        (_, Some(ErrorClass::Ko)) => {
            format!(
                "This position involves a ko fight. Consider the value of ko threats \
                 available to each side before deciding whether to fight or concede. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        // Sente/Gote
        (_, Some(ErrorClass::SenteGote)) => {
            format!(
                "This move loses the initiative. The engine suggests a move that \
                 forces a response — keeping sente (the initiative) is often worth \
                 more than the local gain. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        // Opening / Endgame generic
        (_, Some(ErrorClass::Opening)) => {
            format!(
                "In the opening, focus on balance: claim corners, then sides, then center. \
                 Consider whether this move develops your position efficiently. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        (_, Some(ErrorClass::Endgame)) => {
            format!(
                "In the endgame, counting is key. Look for the largest remaining boundary \
                 plays — a few points here can decide the game. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        // Severity-only fallback
        (Severity::Mistake, _) => {
            format!(
                "This move loses some ground. The engine suggests a different approach here. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        (Severity::Blunder, _) => {
            format!(
                "This is a significant mistake that changes the evaluation of the position. \
                 Take a moment to reconsider — what is the biggest area on the board right now? \
                 (Lost about {score_loss:.1} points)"
            )
        }
        (_, _) => {
            format!("Consider a different approach here. (Lost about {score_loss:.1} points)")
        }
    };

    CoachingMessage {
        severity,
        error_class,
        message,
        suggested_move,
        simplest_move,
        score_loss,
        move_number,
    }
}

const PRAISE_MESSAGES: &[&str] = &[
    "Great read! You found the strongest continuation.",
    "Well played — that's a move the pros would approve of.",
    "Excellent judgment. You're reading the board well here.",
    "Nice! That move shows real understanding of the position.",
    "Spot on — you identified the key point in this area.",
    "Strong move! Your positional sense is developing nicely.",
];

/// Generate praise for an excellent move, returning `Some(CoachingMessage)` roughly 30% of the time
/// when the player's move is among the engine's top 3 candidates.
pub fn maybe_praise(
    move_number: u16,
    player_move: &str,
    move_infos: &[MoveInfo],
    score_loss: f64,
) -> Option<CoachingMessage> {
    // Only praise if the player's move is among the top 3 engine candidates
    let in_top3 = move_infos
        .iter()
        .take(3)
        .any(|m| m.mv == player_move);
    if !in_top3 {
        return None;
    }

    // Rate-limit: praise roughly 30% of qualifying moves
    // Use move_number for deterministic but varied selection
    if move_number % 10 >= 3 {
        return None;
    }

    let idx = (move_number as usize) % PRAISE_MESSAGES.len();
    let message = PRAISE_MESSAGES[idx].to_string();

    Some(CoachingMessage {
        severity: Severity::Excellent,
        error_class: None,
        message,
        suggested_move: None,
        simplest_move: None,
        score_loss,
        move_number,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gosensei_katago::protocol::MoveInfo;

    fn make_move_info(mv: &str, score_lead: f64) -> MoveInfo {
        MoveInfo {
            mv: mv.to_string(),
            visits: 100,
            winrate: 0.5,
            score_lead,
            prior: 0.1,
            order: 0,
            pv: vec![mv.to_string()],
        }
    }

    #[test]
    fn good_move_message() {
        let msg = generate_message(Severity::Good, None, 0.3, None, None, 10);
        assert!(msg.message.contains("Good move"));
    }

    #[test]
    fn excellent_move_message() {
        let msg = generate_message(Severity::Excellent, None, 0.1, None, None, 10);
        assert!(msg.message.contains("Excellent"));
    }

    #[test]
    fn blunder_message_includes_score_loss() {
        let msg = generate_message(Severity::Blunder, Some(ErrorClass::Direction), 15.2, None, None, 25);
        assert!(msg.message.contains("15.2"));
    }

    #[test]
    fn praise_for_top3_move_at_qualifying_number() {
        let infos = vec![
            make_move_info("D4", 5.0),
            make_move_info("Q16", 4.8),
            make_move_info("C3", 4.5),
        ];
        // move_number 0: 0 % 10 = 0 < 3 → should praise
        let result = maybe_praise(0, "Q16", &infos, 0.2);
        assert!(result.is_some());
        assert_eq!(result.unwrap().severity, Severity::Excellent);
    }

    #[test]
    fn no_praise_for_non_top3_move() {
        let infos = vec![
            make_move_info("D4", 5.0),
            make_move_info("Q16", 4.8),
            make_move_info("C3", 4.5),
        ];
        // R1 is not in top 3
        let result = maybe_praise(0, "R1", &infos, 0.3);
        assert!(result.is_none());
    }

    #[test]
    fn praise_rate_limited() {
        let infos = vec![make_move_info("D4", 5.0)];
        // move_number 5: 5 % 10 = 5 >= 3 → should NOT praise
        let result = maybe_praise(5, "D4", &infos, 0.1);
        assert!(result.is_none());
        // move_number 2: 2 % 10 = 2 < 3 → should praise
        let result = maybe_praise(2, "D4", &infos, 0.1);
        assert!(result.is_some());
    }
}
