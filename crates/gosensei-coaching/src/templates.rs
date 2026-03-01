use crate::types::{CoachingMessage, ErrorClass, Severity};

/// Generate a coaching message from classification results.
pub fn generate_message(
    severity: Severity,
    error_class: Option<ErrorClass>,
    score_loss: f64,
    suggested_move: Option<String>,
    move_number: u16,
) -> CoachingMessage {
    let message = match (&severity, &error_class) {
        (Severity::Good, _) => "Good move! This is close to what the engine recommends.".into(),
        (Severity::Inaccuracy, Some(ErrorClass::Direction)) => {
            format!(
                "This move is a small inaccuracy — the direction of play could be better. \
                 Consider the flow of the game and where the biggest areas are. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        (Severity::Inaccuracy, Some(ErrorClass::Shape)) => {
            format!(
                "The shape here isn't quite right. A more efficient stone placement \
                 would keep your groups connected and strong. \
                 (Lost about {score_loss:.1} points)"
            )
        }
        (Severity::Mistake, Some(ErrorClass::LifeAndDeath)) => {
            format!(
                "This is a life-and-death mistake — your group's safety is at risk! \
                 Look for moves that secure two eyes or threaten your opponent's eyespace. \
                 (Lost about {score_loss:.1} points)"
            )
        }
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
        score_loss,
        move_number,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_move_message() {
        let msg = generate_message(Severity::Good, None, 0.3, None, 10);
        assert!(msg.message.contains("Good move"));
    }

    #[test]
    fn blunder_message_includes_score_loss() {
        let msg = generate_message(Severity::Blunder, Some(ErrorClass::Direction), 15.2, None, 25);
        assert!(msg.message.contains("15.2"));
    }
}
