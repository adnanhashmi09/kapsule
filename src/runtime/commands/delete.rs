use crate::runtime::state::StateManager;
use anyhow::Result;

pub fn delete(id: &str) -> Result<()> {
    if !StateManager::exists(id) {
        anyhow::bail!("container {} does not exist", id);
    }

    StateManager::remove(id)?;
    println!("deleted");
    Ok(())
}
