use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DiskSceneConfig {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub welcome_message: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub monologues: Vec<String>,
    #[serde(default)]
    pub time_windows: Vec<DiskSceneTimeWindow>,
    /// 角色处于本场景、用户从另一场景发消息时的短素材（可与 `away_life.txt` 并用）
    #[serde(default)]
    pub away_life_notes: Vec<String>,
    /// 按「用户对话上下文场景 id」覆盖的轨迹素材（键为场景 id）
    #[serde(default)]
    pub away_life_by_user_scene: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DiskSceneTimeWindow {
    pub start: String,
    pub end: String,
}
