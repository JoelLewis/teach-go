use gosensei_katago::protocol::MoveInfo;

/// For DDK players, find the engine candidate within `max_score_gap` points of best
/// that has the shortest principal variation (simplest to understand).
///
/// Returns `None` if there are no candidates or the best move is already the simplest.
pub fn find_simplest_good_move(move_infos: &[MoveInfo], max_score_gap: f64) -> Option<&MoveInfo> {
    let best = move_infos.first()?;
    let best_score = best.score_lead;

    // Filter to candidates within the score gap of the best move
    let candidates: Vec<&MoveInfo> = move_infos
        .iter()
        .filter(|m| (best_score - m.score_lead).abs() <= max_score_gap)
        .collect();

    // Pick the one with the shortest PV (ties broken by higher score)
    let simplest = candidates.iter().min_by(|a, b| {
        a.pv.len().cmp(&b.pv.len()).then_with(|| {
            b.score_lead
                .partial_cmp(&a.score_lead)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    })?;

    // Only return if it's different from the best move
    if simplest.mv == best.mv {
        None
    } else {
        Some(simplest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_move_info(mv: &str, score_lead: f64, pv_len: usize) -> MoveInfo {
        MoveInfo {
            mv: mv.to_string(),
            visits: 100,
            winrate: 0.5,
            score_lead,
            prior: 0.1,
            order: 0,
            pv: (0..pv_len).map(|i| format!("M{i}")).collect(),
        }
    }

    #[test]
    fn returns_shorter_pv_within_gap() {
        let moves = vec![
            make_move_info("Q16", 5.0, 12), // best but deep
            make_move_info("D4", 4.5, 3),   // within 1pt, much shorter PV
            make_move_info("C3", 4.0, 5),   // within 1pt, medium PV
        ];
        let result = find_simplest_good_move(&moves, 1.0);
        assert_eq!(result.unwrap().mv, "D4");
    }

    #[test]
    fn returns_none_when_best_is_already_simplest() {
        let moves = vec![
            make_move_info("Q16", 5.0, 3), // best AND shortest
            make_move_info("D4", 4.5, 8),  // within gap but longer
            make_move_info("C3", 4.0, 12), // within gap but longest
        ];
        let result = find_simplest_good_move(&moves, 1.0);
        assert!(result.is_none());
    }

    #[test]
    fn excludes_moves_outside_gap() {
        let moves = vec![
            make_move_info("Q16", 5.0, 12), // best
            make_move_info("D4", 3.5, 2),   // short PV but 1.5pt worse — outside 1pt gap
            make_move_info("C3", 4.2, 6),   // within gap
        ];
        let result = find_simplest_good_move(&moves, 1.0);
        assert_eq!(result.unwrap().mv, "C3");
    }

    #[test]
    fn returns_none_for_empty() {
        let result = find_simplest_good_move(&[], 1.0);
        assert!(result.is_none());
    }
}
