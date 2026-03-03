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

/// Classify severity of a move based on score loss and player rank.
pub fn classify_severity(score_loss: f64, player_rank: f64) -> Severity {
    Severity::from_score_loss(score_loss, player_rank)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_thresholds_beginner_band() {
        // 25-15 kyu: Excellent <0.5, Good <2.0, Inaccuracy <5.0, Mistake <12.0, Blunder 12.0+
        let rank = 20.0;
        assert_eq!(classify_severity(0.3, rank), Severity::Excellent);
        assert_eq!(classify_severity(1.0, rank), Severity::Good);
        assert_eq!(classify_severity(3.0, rank), Severity::Inaccuracy);
        assert_eq!(classify_severity(7.0, rank), Severity::Mistake);
        assert_eq!(classify_severity(15.0, rank), Severity::Blunder);
    }

    #[test]
    fn severity_thresholds_intermediate_band() {
        // 15-10 kyu: Excellent <0.5, Good <1.5, Inaccuracy <4.0, Mistake <8.0, Blunder 8.0+
        let rank = 12.0;
        assert_eq!(classify_severity(0.3, rank), Severity::Excellent);
        assert_eq!(classify_severity(1.0, rank), Severity::Good);
        assert_eq!(classify_severity(2.0, rank), Severity::Inaccuracy);
        assert_eq!(classify_severity(5.0, rank), Severity::Mistake);
        assert_eq!(classify_severity(10.0, rank), Severity::Blunder);
    }

    #[test]
    fn severity_thresholds_advanced_band() {
        // 10-5 kyu: Excellent <0.3, Good <1.0, Inaccuracy <3.0, Mistake <6.0, Blunder 6.0+
        let rank = 7.0;
        assert_eq!(classify_severity(0.2, rank), Severity::Excellent);
        assert_eq!(classify_severity(0.5, rank), Severity::Good);
        assert_eq!(classify_severity(2.0, rank), Severity::Inaccuracy);
        assert_eq!(classify_severity(4.0, rank), Severity::Mistake);
        assert_eq!(classify_severity(8.0, rank), Severity::Blunder);
    }

    #[test]
    fn severity_thresholds_dan_band() {
        // 5k-1d: Excellent <0.2, Good <0.8, Inaccuracy <2.0, Mistake <4.0, Blunder 4.0+
        let rank = 3.0;
        assert_eq!(classify_severity(0.1, rank), Severity::Excellent);
        assert_eq!(classify_severity(0.5, rank), Severity::Good);
        assert_eq!(classify_severity(1.5, rank), Severity::Inaccuracy);
        assert_eq!(classify_severity(3.0, rank), Severity::Mistake);
        assert_eq!(classify_severity(5.0, rank), Severity::Blunder);
    }

    #[test]
    fn severity_boundary_values() {
        // Exact boundary tests for beginner band
        assert_eq!(classify_severity(0.5, 20.0), Severity::Good); // exactly at threshold = next band
        assert_eq!(classify_severity(2.0, 20.0), Severity::Inaccuracy);
        assert_eq!(classify_severity(5.0, 20.0), Severity::Mistake);
        assert_eq!(classify_severity(12.0, 20.0), Severity::Blunder);
    }
}
