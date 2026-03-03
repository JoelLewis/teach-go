use rusqlite::Connection;

use crate::error::AppError;
use crate::import::import_from_sgf;
use crate::problem::insert_problem;

struct CollectionSpec {
    name: &'static str,
    sgf: &'static str,
    /// Difficulty for the first problem (easiest in this collection).
    difficulty_base: f64,
    /// Difficulty for the last problem (hardest in this collection).
    difficulty_top: f64,
    /// Source tag for tracking.
    source_tag: &'static str,
}

const COLLECTIONS: &[CollectionSpec] = &[
    // Note: Gokyo Shumyo excluded — its FF[3] SGFs use non-root setup stones
    // and linear solutions that our tree parser doesn't handle.
    CollectionSpec {
        name: "Xuanxuan Qijing",
        sgf: include_str!("problems/xuanxuan-qijing.sgf"),
        difficulty_base: 5.0,
        difficulty_top: 1.0,
        source_tag: "xuanxuan-qijing",
    },
    CollectionSpec {
        name: "Guanzipu Part 1",
        sgf: include_str!("problems/guanzipu-1.sgf"),
        difficulty_base: 6.0,
        difficulty_top: 1.0,
        source_tag: "guanzipu",
    },
    CollectionSpec {
        name: "Guanzipu Part 2",
        sgf: include_str!("problems/guanzipu-2.sgf"),
        difficulty_base: 6.0,
        difficulty_top: 1.0,
        source_tag: "guanzipu",
    },
    CollectionSpec {
        name: "Guanzipu Part 3",
        sgf: include_str!("problems/guanzipu-3.sgf"),
        difficulty_base: 6.0,
        difficulty_top: 1.0,
        source_tag: "guanzipu",
    },
    CollectionSpec {
        name: "Go Game Guru Easy",
        sgf: include_str!("problems/gogameguru-easy.sgf"),
        difficulty_base: 15.0,
        difficulty_top: 10.0,
        source_tag: "gogameguru",
    },
    CollectionSpec {
        name: "Go Game Guru Intermediate",
        sgf: include_str!("problems/gogameguru-intermediate.sgf"),
        difficulty_base: 8.0,
        difficulty_top: 3.0,
        source_tag: "gogameguru",
    },
    CollectionSpec {
        name: "Go Game Guru Hard",
        sgf: include_str!("problems/gogameguru-hard.sgf"),
        difficulty_base: 3.0,
        difficulty_top: 1.0,
        source_tag: "gogameguru",
    },
];

/// Import all bundled classical problem collections into the database.
/// Uses a single transaction for performance.
pub fn seed_bundled_collections(conn: &Connection) -> Result<u32, AppError> {
    let tx = conn.unchecked_transaction()?;
    let mut total = 0u32;

    for spec in COLLECTIONS {
        let result = import_from_sgf(spec.sgf, Some(spec.difficulty_base));
        let count = result.problems.len();

        for (i, mut problem) in result.problems.into_iter().enumerate() {
            // Interpolate difficulty from base→top across the collection
            if count > 1 {
                let t = i as f64 / (count - 1) as f64;
                problem.difficulty =
                    spec.difficulty_base + t * (spec.difficulty_top - spec.difficulty_base);
            }

            // Add the collection source tag
            problem.tags.push(spec.source_tag.to_string());

            let _ = insert_problem(&tx, &problem);
            total += 1;
        }

        if !result.errors.is_empty() {
            eprintln!(
                "[seed_content] {}: {} imported, {} errors (first: {})",
                spec.name,
                count,
                result.errors.len(),
                result.errors[0]
            );
        }
    }

    tx.commit()?;
    Ok(total)
}
