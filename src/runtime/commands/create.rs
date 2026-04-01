use crate::runtime::state::{ContainerState, StateManager, Status};
use anyhow::{Context, Result};

pub fn create(id: &str, bundle: &str) -> Result<()> {
    let bundle_path = std::path::Path::new(bundle);

    if !bundle_path.exists() {
        anyhow::bail!("bundle does not exist: {}", bundle);
    }

    let config_path = bundle_path.join("config.json");
    if !config_path.exists() {
        anyhow::bail!("config.json not found in bundle: {}", bundle);
    }

    let spec = crate::runtime::spec::RuntimeSpec::load_config(bundle)
        .context("failed to load config.json")?;
    spec.validate().context("spec validation failed")?;

    let state = ContainerState {
        version: "1.0.2".into(),
        id: id.into(),
        status: Status::Created,
        pid: 0,
        bundle: bundle_path
            .canonicalize()
            .map_err(|e| anyhow::anyhow!("failed to canonicalize bundle path: {}", e))?,
        annotations: Default::default(),
    };

    StateManager::save(&state)?;

    println!("created");
    Ok(())
}
