use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ImageConfigManifest {
    pub architecture: String,
    pub config: Config,
    pub created: String,
    pub history: Vec<HistoryEntry>,
    pub os: String,
    pub rootfs: RootFs,
    pub variant: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "Cmd")]
    pub cmd: Option<Vec<String>>,
    #[serde(rename = "Entrypoint")]
    pub entrypoint: Option<Vec<String>>,
    #[serde(rename = "Env")]
    pub env: Option<Vec<String>>,
    #[serde(rename = "ExposedPorts")]
    pub exposed_ports: Option<HashMap<String, serde_json::Value>>,
    #[serde(rename = "Volumes")]
    pub volumes: Option<HashMap<String, serde_json::Value>>,
    #[serde(rename = "WorkingDir")]
    pub working_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryEntry {
    pub comment: Option<String>,
    pub created: String,
    #[serde(rename = "created_by")]
    pub created_by: Option<String>,
    #[serde(rename = "empty_layer")]
    pub empty_layer: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RootFs {
    #[serde(rename = "diff_ids")]
    pub diff_ids: Vec<String>,
    #[serde(rename = "type")]
    pub fs_type: String,
}
