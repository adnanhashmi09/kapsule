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
    let pid = std::process::id();
    state.status = Status::Running;
    state.pid = pid;

    StateManager::save(&state)?;
    StateManager::write_pidfile(id, pid)?;

    println!("started");
    Ok(())
}
