use crate::runtime::state::{StateManager, Status};
use anyhow::Result;

pub fn pause(id: &str) -> Result<()> {
    if !StateManager::exists(id) {
        anyhow::bail!("container {} does not exist", id);
    }

    let mut state = StateManager::load(id)?;

    if state.status != Status::Running {
        anyhow::bail!("container {} is not running", id);
    }

    state.status = Status::Paused;
    StateManager::save(&state)?;

    println!("paused");
    Ok(())
}
