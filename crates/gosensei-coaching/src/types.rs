use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
}

impl Severity {
    /// Classify severity based on score loss (in points) relative to rank band.
    pub fn from_score_loss(score_loss: f64, is_ddk: bool) -> Self {
        if is_ddk {
            // DDK thresholds (more forgiving)
            match score_loss {
                l if l < 2.0 => Severity::Good,
                l if l < 5.0 => Severity::Inaccuracy,
                l if l < 10.0 => Severity::Mistake,
                _ => Severity::Blunder,
            }
        } else {
            // SDK thresholds
            match score_loss {
                l if l < 1.0 => Severity::Good,
                l if l < 3.0 => Severity::Inaccuracy,
                l if l < 7.0 => Severity::Mistake,
                _ => Severity::Blunder,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorClass {
    Direction,
    Shape,
    Reading,
    LifeAndDeath,
    Endgame,
    Opening,
    Ko,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachingMessage {
    pub severity: Severity,
    pub error_class: Option<ErrorClass>,
    pub message: String,
    pub suggested_move: Option<String>,
    pub score_loss: f64,
    pub move_number: u16,
}
