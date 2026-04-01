use crate::runtime::state::{StateManager, Status};
use anyhow::Result;

pub fn resume(id: &str) -> Result<()> {
    if !StateManager::exists(id) {
        anyhow::bail!("container {} does not exist", id);
    }

    let mut state = StateManager::load(id)?;

    if state.status != Status::Paused {
        anyhow::bail!("container {} is not paused", id);
    }

    state.status = Status::Running;
    StateManager::save(&state)?;

    println!("resumed");
    Ok(())
}
