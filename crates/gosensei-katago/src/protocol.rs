use serde::{Deserialize, Serialize};

/// KataGo Analysis Engine query format.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisQuery {
    pub id: String,
    pub moves: Vec<(String, String)>,
    pub rules: String,
    pub komi: f32,
    #[serde(rename = "boardXSize")]
    pub board_x_size: u8,
    #[serde(rename = "boardYSize")]
    pub board_y_size: u8,
    pub max_visits: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_ownership: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_policy: Option<bool>,
}

/// KataGo Analysis Engine response format.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResponse {
    pub id: String,
    pub move_infos: Vec<MoveInfo>,
    pub root_info: RootInfo,
    #[serde(default)]
    pub ownership: Vec<f32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveInfo {
    #[serde(rename = "move")]
    pub mv: String,
    pub visits: u32,
    pub winrate: f64,
    pub score_lead: f64,
    pub prior: f64,
    pub order: u32,
    pub pv: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootInfo {
    pub winrate: f64,
    pub score_lead: f64,
    pub visits: u32,
}
