use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: i64,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub global_shortcut: Option<String>,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipItem {
    pub id: i64,
    pub content: String,
    pub content_type: String,
    pub preview: String,
    pub folder_id: i64,
    pub is_pinned: bool,
    pub is_deleted: bool,
    pub source_app: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub id: i64,
    pub master_shortcut: String,
    pub auto_clean_enabled: bool,
    pub auto_clean_days: i64,
    pub max_history_items: i64,
    pub paste_on_click: bool,
    pub theme: String,
    pub launch_at_login: bool,
    pub ignored_apps: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStats {
    pub total_clips: i64,
    pub folders_count: i64,
    pub pinned_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    pub master_shortcut: Option<String>,
    pub auto_clean_enabled: Option<bool>,
    pub auto_clean_days: Option<i64>,
    pub max_history_items: Option<i64>,
    pub paste_on_click: Option<bool>,
    pub theme: Option<String>,
    pub launch_at_login: Option<bool>,
    pub ignored_apps: Option<String>,
}
