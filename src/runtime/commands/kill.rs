use crate::runtime::state::StateManager;
use anyhow::Result;

pub fn kill(id: &str, _signal: u32) -> Result<()> {
    if !StateManager::exists(id) {
        anyhow::bail!("container {} does not exist", id);
    }

    let state = StateManager::load(id)?;

    if state.pid == 0 {
        anyhow::bail!("container {} has no active process", id);
    }

    // STUB: actual signal sending happens in Milestone 4
    println!("killed");
    Ok(())
}
