use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const STATE_DIR: &str = "/run/kapsule";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerState {
    pub version: String,
    pub id: String,
    pub status: Status,
    pub pid: u32,
    pub bundle: PathBuf,
    #[serde(default)]
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Created,
    Running,
    Paused,
    Stopped,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Created => "created",
            Status::Running => "running",
            Status::Paused => "paused",
            Status::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub struct StateManager;

impl StateManager {
    pub fn container_dir(id: &str) -> PathBuf {
        PathBuf::from(STATE_DIR).join(id)
    }

    pub fn state_file(id: &str) -> PathBuf {
        Self::container_dir(id).join("state.json")
    }

    pub fn load(id: &str) -> anyhow::Result<ContainerState> {
        let path = Self::state_file(id);
        let json = fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("failed to read state for {}: {}", id, e))?;
        let state: ContainerState = serde_json::from_str(&json)
            .map_err(|e| anyhow::anyhow!("failed to parse state for {}: {}", id, e))?;
        Ok(state)
    }

    pub fn save(state: &ContainerState) -> anyhow::Result<()> {
        let dir = Self::container_dir(&state.id);
        fs::create_dir_all(&dir)
            .map_err(|e| anyhow::anyhow!("failed to create state dir: {}", e))?;
        let path = Self::state_file(&state.id);
        let json = serde_json::to_string_pretty(state)
            .map_err(|e| anyhow::anyhow!("failed to serialize state: {}", e))?;
        fs::write(&path, json).map_err(|e| anyhow::anyhow!("failed to write state: {}", e))?;
        Ok(())
    }

    pub fn exists(id: &str) -> bool {
        Self::state_file(id).exists()
    }

    pub fn remove(id: &str) -> anyhow::Result<()> {
        let dir = Self::container_dir(id);
        if dir.exists() {
            fs::remove_dir_all(&dir)
                .map_err(|e| anyhow::anyhow!("failed to remove state dir: {}", e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_display() {
        assert_eq!(Status::Created.as_str(), "created");
        assert_eq!(Status::Running.as_str(), "running");
        assert_eq!(Status::Paused.as_str(), "paused");
        assert_eq!(Status::Stopped.as_str(), "stopped");
    }

    #[test]
    fn test_state_json_serialization() {
        let state = ContainerState {
            version: "1.0.2".into(),
            id: "test".into(),
            status: Status::Running,
            pid: 1234,
            bundle: "/bundle".into(),
            annotations: HashMap::new(),
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"status\":\"running\""));
        assert!(json.contains("\"pid\":1234"));
        assert!(json.contains("\"id\":\"test\""));
        assert!(json.contains("\"version\":\"1.0.2\""));
    }

    #[test]
    fn test_container_dir_path() {
        assert_eq!(
            StateManager::container_dir("mycontainer"),
            PathBuf::from("/run/kapsule/mycontainer")
        );
        assert_eq!(
            StateManager::state_file("mycontainer"),
            PathBuf::from("/run/kapsule/mycontainer/state.json")
        );
    }
}
