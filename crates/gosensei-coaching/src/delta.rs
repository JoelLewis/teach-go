use gosensei_katago::protocol::AnalysisResponse;

use crate::types::Severity;

/// Extract the score delta between the player's move and the best move.
pub fn score_loss(pre_analysis: &AnalysisResponse, player_move: &str) -> f64 {
    let best_score = pre_analysis
        .move_infos
        .first()
        .map(|m| m.score_lead)
        .unwrap_or(0.0);

    let player_score = pre_analysis
        .move_infos
        .iter()
        .find(|m| m.mv == player_move)
        .map(|m| m.score_lead)
        .unwrap_or(best_score);

    (best_score - player_score).abs()
}

/// Extract winrate loss between best and played move.
pub fn winrate_loss(pre_analysis: &AnalysisResponse, player_move: &str) -> f64 {
    let best_wr = pre_analysis
        .move_infos
        .first()
        .map(|m| m.winrate)
        .unwrap_or(0.5);

    let player_wr = pre_analysis
        .move_infos
        .iter()
        .find(|m| m.mv == player_move)
        .map(|m| m.winrate)
        .unwrap_or(best_wr);

    (best_wr - player_wr).abs()
}

/// Classify severity of a move based on score loss.
pub fn classify_severity(score_loss: f64, is_ddk: bool) -> Severity {
    Severity::from_score_loss(score_loss, is_ddk)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_thresholds_ddk() {
        assert_eq!(classify_severity(0.5, true), Severity::Good);
        assert_eq!(classify_severity(3.0, true), Severity::Inaccuracy);
        assert_eq!(classify_severity(7.0, true), Severity::Mistake);
        assert_eq!(classify_severity(15.0, true), Severity::Blunder);
    }

    #[test]
    fn severity_thresholds_sdk() {
        assert_eq!(classify_severity(0.5, false), Severity::Good);
        assert_eq!(classify_severity(2.0, false), Severity::Inaccuracy);
        assert_eq!(classify_severity(5.0, false), Severity::Mistake);
        assert_eq!(classify_severity(10.0, false), Severity::Blunder);
    }
}
