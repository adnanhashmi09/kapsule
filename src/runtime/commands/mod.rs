pub mod create;
pub mod delete;
pub mod kill;
pub mod pause;
pub mod resume;
pub mod state_cmd;

#[cfg(target_os = "linux")]
pub mod start;

pub use create::create;
pub use delete::delete;
pub use kill::kill;
pub use pause::pause;
pub use resume::resume;
pub use state_cmd::state;

#[cfg(target_os = "linux")]
pub use start::start;

use crate::runtime::cli::Command;
use anyhow::Result;

pub fn execute(cmd: Command) -> Result<()> {
    match cmd {
        Command::Create { id, bundle } => create(&id, &bundle),
        #[cfg(target_os = "linux")]
        Command::Start { id } => start(&id),
        #[cfg(not(target_os = "linux"))]
        Command::Start { .. } => {
            anyhow::bail!("container start is only supported on Linux")
        }
        Command::State { id } => state(&id),
        Command::Kill { id, signal } => kill(&id, signal),
        Command::Delete { id } => delete(&id),
        Command::Pause { id } => pause(&id),
        Command::Resume { id } => resume(&id),
    }
}
