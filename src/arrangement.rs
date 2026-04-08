use serde::de::DeserializeOwned;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Arrangement {
    pub name: Option<String>,
    pub description: Option<String>,

    pub workspaces: Vec<ArrangementWorkspace>,
    pub monitors: Vec<ArrangementMonitor> = Vec::new(),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArrangementMonitor {
    pub name: String,
    pub is_main: bool = false,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArrangementWorkspace {
    pub name: String,
    pub monitor: Option<String>,
    pub windows: Vec<ArrangementWindow>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArrangementWindow {
    pub app_name: String,
    pub title: Option<String>,
}