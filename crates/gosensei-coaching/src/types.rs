use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Excellent,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
}

impl Severity {
    /// Classify severity based on score loss (in points) relative to player rank.
    ///
    /// Uses 4 rank bands with progressively tighter thresholds:
    /// - 25-15 kyu (rank >= 15.0): most forgiving
    /// - 15-10 kyu (rank >= 10.0)
    /// - 10-5 kyu (rank >= 5.0)
    /// - 5 kyu-1 dan (rank < 5.0): strictest
    pub fn from_score_loss(score_loss: f64, player_rank: f64) -> Self {
        let (excellent, good, inaccuracy, mistake) = if player_rank >= 15.0 {
            (0.5, 2.0, 5.0, 12.0)
        } else if player_rank >= 10.0 {
            (0.5, 1.5, 4.0, 8.0)
        } else if player_rank >= 5.0 {
            (0.3, 1.0, 3.0, 6.0)
        } else {
            (0.2, 0.8, 2.0, 4.0)
        };

        match score_loss {
            l if l < excellent => Severity::Excellent,
            l if l < good => Severity::Good,
            l if l < inaccuracy => Severity::Inaccuracy,
            l if l < mistake => Severity::Mistake,
            _ => Severity::Blunder,
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
    SenteGote,
}

impl std::str::FromStr for ErrorClass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Direction" => Ok(Self::Direction),
            "Shape" => Ok(Self::Shape),
            "Reading" => Ok(Self::Reading),
            "LifeAndDeath" => Ok(Self::LifeAndDeath),
            "Endgame" => Ok(Self::Endgame),
            "Opening" => Ok(Self::Opening),
            "Ko" => Ok(Self::Ko),
            "SenteGote" => Ok(Self::SenteGote),
            _ => Err(format!("unknown error class: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachingMessage {
    pub severity: Severity,
    pub error_class: Option<ErrorClass>,
    pub message: String,
    pub suggested_move: Option<String>,
    pub simplest_move: Option<String>,
    pub score_loss: f64,
    pub move_number: u16,
}
