use crate::runtime::state::{ContainerState, StateManager, Status};
use anyhow::Result;

pub fn start(id: &str) -> Result<()> {
    if !StateManager::exists(id) {
        anyhow::bail!("container {} does not exist", id);
    }

    let mut state = StateManager::load(id)?;

    if state.status != Status::Created {
        anyhow::bail!("container {} is not in created state", id);
    }

    // STUB: actual process spawning happens in Milestone 4
    state.status = Status::Running;
    state.pid = std::process::id();

    StateManager::save(&state)?;

    println!("started");
    Ok(())
}
