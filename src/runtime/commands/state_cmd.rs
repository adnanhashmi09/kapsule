use crate::runtime::state::StateManager;
use anyhow::Result;

pub fn state(id: &str) -> Result<()> {
    if !StateManager::exists(id) {
        anyhow::bail!("container {} does not exist", id);
    }

    let state = StateManager::load(id)?;
    let json = serde_json::to_string(&state)?;
    println!("{}", json);
    Ok(())
}
