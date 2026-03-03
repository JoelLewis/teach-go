use gosensei_coaching::types::ErrorClass;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;

const DEFAULT_RANK: f64 = 25.0;

/// Read player rank from skill profile, defaulting to 25k for new players.
pub fn get_player_rank(state: &AppState) -> f64 {
    let db = state.db.lock().unwrap();
    get_skill_profile(&db)
        .map(|p| p.overall_rank)
        .unwrap_or(DEFAULT_RANK)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDimension {
    pub mu: f64,
    pub sigma: f64,
}

impl Default for SkillDimension {
    fn default() -> Self {
        Self {
            mu: 25.0,
            sigma: 8.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProfile {
    pub overall_rank: f64,
    pub reading: SkillDimension,
    pub shape: SkillDimension,
    pub direction: SkillDimension,
    pub endgame: SkillDimension,
    pub life_death: SkillDimension,
    pub fighting: SkillDimension,
    pub games_played: u32,
    pub last_updated: String,
}

impl Default for SkillProfile {
    fn default() -> Self {
        Self {
            overall_rank: 25.0,
            reading: SkillDimension::default(),
            shape: SkillDimension::default(),
            direction: SkillDimension::default(),
            endgame: SkillDimension::default(),
            life_death: SkillDimension::default(),
            fighting: SkillDimension::default(),
            games_played: 0,
            last_updated: String::new(),
        }
    }
}

pub struct GameError {
    pub error_class: ErrorClass,
    pub score_loss: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionKind {
    Reading,
    Shape,
    Direction,
    Endgame,
    LifeDeath,
    Fighting,
}

pub fn error_class_to_dimension(ec: ErrorClass) -> DimensionKind {
    match ec {
        ErrorClass::Reading => DimensionKind::Reading,
        ErrorClass::Shape => DimensionKind::Shape,
        ErrorClass::Direction | ErrorClass::Opening => DimensionKind::Direction,
        ErrorClass::Endgame => DimensionKind::Endgame,
        ErrorClass::LifeAndDeath => DimensionKind::LifeDeath,
        ErrorClass::Ko => DimensionKind::Fighting,
    }
}

fn update_dimension(dim: &mut SkillDimension, total_score_loss: f64, had_errors: bool) {
    let learning_rate = 0.1 * (dim.sigma / 8.0);
    if had_errors {
        let delta = learning_rate * (total_score_loss / 10.0).min(3.0);
        dim.mu = (dim.mu + delta).min(30.0);
    } else {
        let delta = learning_rate * 0.5;
        dim.mu = (dim.mu - delta).max(1.0);
    }
    dim.sigma = (dim.sigma * 0.95).max(1.0);
}

fn compute_overall_rank(profile: &SkillProfile) -> f64 {
    let weighted = profile.reading.mu * 0.25
        + profile.direction.mu * 0.20
        + profile.shape.mu * 0.15
        + profile.endgame.mu * 0.15
        + profile.life_death.mu * 0.15
        + profile.fighting.mu * 0.10;
    weighted.clamp(1.0, 30.0)
}

fn get_dimension_mut(profile: &mut SkillProfile, kind: DimensionKind) -> &mut SkillDimension {
    match kind {
        DimensionKind::Reading => &mut profile.reading,
        DimensionKind::Shape => &mut profile.shape,
        DimensionKind::Direction => &mut profile.direction,
        DimensionKind::Endgame => &mut profile.endgame,
        DimensionKind::LifeDeath => &mut profile.life_death,
        DimensionKind::Fighting => &mut profile.fighting,
    }
}

const ALL_DIMENSIONS: [DimensionKind; 6] = [
    DimensionKind::Reading,
    DimensionKind::Shape,
    DimensionKind::Direction,
    DimensionKind::Endgame,
    DimensionKind::LifeDeath,
    DimensionKind::Fighting,
];

// --- DB operations ---

pub fn get_skill_profile(conn: &Connection) -> Result<SkillProfile, AppError> {
    let mut stmt = conn.prepare(
        "SELECT overall_rank, reading_mu, reading_sigma, shape_mu, shape_sigma,
                direction_mu, direction_sigma, endgame_mu, endgame_sigma,
                life_death_mu, life_death_sigma, fighting_mu, fighting_sigma,
                games_played, last_updated
         FROM skill_profiles WHERE player_id = 1",
    )?;

    let profile = stmt.query_row([], |row| {
        Ok(SkillProfile {
            overall_rank: row.get(0)?,
            reading: SkillDimension { mu: row.get(1)?, sigma: row.get(2)? },
            shape: SkillDimension { mu: row.get(3)?, sigma: row.get(4)? },
            direction: SkillDimension { mu: row.get(5)?, sigma: row.get(6)? },
            endgame: SkillDimension { mu: row.get(7)?, sigma: row.get(8)? },
            life_death: SkillDimension { mu: row.get(9)?, sigma: row.get(10)? },
            fighting: SkillDimension { mu: row.get(11)?, sigma: row.get(12)? },
            games_played: row.get(13)?,
            last_updated: row.get(14)?,
        })
    });

    match profile {
        Ok(p) => Ok(p),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // First call — insert default row
            conn.execute(
                "INSERT INTO skill_profiles (player_id) VALUES (1)",
                [],
            )?;
            // Re-read to get defaults from schema
            get_skill_profile(conn)
        }
        Err(e) => Err(e.into()),
    }
}

pub fn save_skill_profile(conn: &Connection, profile: &SkillProfile) -> Result<(), AppError> {
    conn.execute(
        "UPDATE skill_profiles SET
            overall_rank = ?1, reading_mu = ?2, reading_sigma = ?3,
            shape_mu = ?4, shape_sigma = ?5, direction_mu = ?6, direction_sigma = ?7,
            endgame_mu = ?8, endgame_sigma = ?9, life_death_mu = ?10, life_death_sigma = ?11,
            fighting_mu = ?12, fighting_sigma = ?13, games_played = ?14,
            last_updated = datetime('now')
         WHERE player_id = 1",
        rusqlite::params![
            profile.overall_rank,
            profile.reading.mu, profile.reading.sigma,
            profile.shape.mu, profile.shape.sigma,
            profile.direction.mu, profile.direction.sigma,
            profile.endgame.mu, profile.endgame.sigma,
            profile.life_death.mu, profile.life_death.sigma,
            profile.fighting.mu, profile.fighting.sigma,
            profile.games_played,
        ],
    )?;
    Ok(())
}

pub fn update_skill_after_game(
    conn: &Connection,
    errors: &[GameError],
) -> Result<SkillProfile, AppError> {
    let mut profile = get_skill_profile(conn)?;

    // Group errors by dimension: accumulate total score loss per dimension
    let mut dim_losses: std::collections::HashMap<DimensionKind, f64> =
        std::collections::HashMap::new();
    for err in errors {
        let kind = error_class_to_dimension(err.error_class);
        *dim_losses.entry(kind).or_default() += err.score_loss;
    }

    // Update each dimension
    for kind in ALL_DIMENSIONS {
        let dim = get_dimension_mut(&mut profile, kind);
        match dim_losses.get(&kind) {
            Some(&loss) => update_dimension(dim, loss, true),
            None => update_dimension(dim, 0.0, false),
        }
    }

    profile.games_played += 1;
    profile.overall_rank = compute_overall_rank(&profile);

    save_skill_profile(conn, &profile)?;
    Ok(profile)
}

/// Lighter skill update from problem solving (half the learning rate of games).
pub fn update_skill_after_problem(
    conn: &Connection,
    dimension: DimensionKind,
    solved: bool,
    _difficulty: f64,
) -> Result<SkillProfile, AppError> {
    let mut profile = get_skill_profile(conn)?;

    let dim = get_dimension_mut(&mut profile, dimension);
    // Half the game learning rate: problems are quicker, noisier signals
    let learning_rate = 0.05 * (dim.sigma / 8.0);
    if solved {
        dim.mu = (dim.mu - learning_rate * 0.5).max(1.0);
    } else {
        dim.mu = (dim.mu + learning_rate * 1.0).min(30.0);
    }
    // Sigma decays by 2% (vs 5% for games)
    dim.sigma = (dim.sigma * 0.98).max(1.0);

    profile.overall_rank = compute_overall_rank(&profile);
    save_skill_profile(conn, &profile)?;
    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_schema;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn default_profile_at_25k() {
        let conn = test_db();
        let profile = get_skill_profile(&conn).unwrap();
        assert!((profile.overall_rank - 25.0).abs() < f64::EPSILON);
        assert!((profile.reading.mu - 25.0).abs() < f64::EPSILON);
        assert!((profile.reading.sigma - 8.0).abs() < f64::EPSILON);
        assert_eq!(profile.games_played, 0);
    }

    #[test]
    fn errors_increase_mu() {
        let mut dim = SkillDimension { mu: 25.0, sigma: 8.0 };
        update_dimension(&mut dim, 10.0, true);
        assert!(dim.mu > 25.0, "mu should increase (rank worsens) after errors");
    }

    #[test]
    fn no_errors_decrease_mu() {
        let mut dim = SkillDimension { mu: 25.0, sigma: 8.0 };
        update_dimension(&mut dim, 0.0, false);
        assert!(dim.mu < 25.0, "mu should decrease (rank improves) with no errors");
    }

    #[test]
    fn sigma_decreases_over_updates() {
        let mut dim = SkillDimension { mu: 25.0, sigma: 8.0 };
        update_dimension(&mut dim, 0.0, false);
        assert!(dim.sigma < 8.0, "sigma should decrease after an update");
    }

    #[test]
    fn sigma_respects_floor() {
        let mut dim = SkillDimension { mu: 25.0, sigma: 1.0 };
        update_dimension(&mut dim, 0.0, false);
        assert!(
            (dim.sigma - 1.0).abs() < f64::EPSILON,
            "sigma should not go below 1.0"
        );
    }

    #[test]
    fn overall_rank_clamped() {
        let mut profile = SkillProfile::default();
        // Set all dimensions to extreme low (strong)
        profile.reading.mu = 0.5;
        profile.shape.mu = 0.5;
        profile.direction.mu = 0.5;
        profile.endgame.mu = 0.5;
        profile.life_death.mu = 0.5;
        profile.fighting.mu = 0.5;
        assert!((compute_overall_rank(&profile) - 1.0).abs() < f64::EPSILON);

        // Set all dimensions to extreme high (weak)
        profile.reading.mu = 35.0;
        profile.shape.mu = 35.0;
        profile.direction.mu = 35.0;
        profile.endgame.mu = 35.0;
        profile.life_death.mu = 35.0;
        profile.fighting.mu = 35.0;
        assert!((compute_overall_rank(&profile) - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn error_class_mapping_all_variants() {
        assert_eq!(error_class_to_dimension(ErrorClass::Reading), DimensionKind::Reading);
        assert_eq!(error_class_to_dimension(ErrorClass::Shape), DimensionKind::Shape);
        assert_eq!(error_class_to_dimension(ErrorClass::Direction), DimensionKind::Direction);
        assert_eq!(error_class_to_dimension(ErrorClass::Opening), DimensionKind::Direction);
        assert_eq!(error_class_to_dimension(ErrorClass::Endgame), DimensionKind::Endgame);
        assert_eq!(error_class_to_dimension(ErrorClass::LifeAndDeath), DimensionKind::LifeDeath);
        assert_eq!(error_class_to_dimension(ErrorClass::Ko), DimensionKind::Fighting);
    }

    #[test]
    fn full_update_skill_after_game() {
        let conn = test_db();
        let errors = vec![
            GameError { error_class: ErrorClass::Reading, score_loss: 5.0 },
            GameError { error_class: ErrorClass::Reading, score_loss: 3.0 },
            GameError { error_class: ErrorClass::Shape, score_loss: 7.0 },
        ];
        let profile = update_skill_after_game(&conn, &errors).unwrap();
        assert_eq!(profile.games_played, 1);
        // Reading had errors — mu should increase
        assert!(profile.reading.mu > 25.0);
        // Shape had errors — mu should increase
        assert!(profile.shape.mu > 25.0);
        // Direction had no errors — mu should decrease
        assert!(profile.direction.mu < 25.0);
    }

    #[test]
    fn games_played_increments() {
        let conn = test_db();
        update_skill_after_game(&conn, &[]).unwrap();
        update_skill_after_game(&conn, &[]).unwrap();
        let profile = get_skill_profile(&conn).unwrap();
        assert_eq!(profile.games_played, 2);
    }
}
