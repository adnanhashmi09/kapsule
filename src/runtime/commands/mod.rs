pub mod create;
pub mod start;
pub mod state_cmd;
pub mod kill;
pub mod delete;
pub mod pause;
pub mod resume;

pub use create::create;
pub use start::start;
pub use state_cmd::state;
pub use kill::kill;
pub use delete::delete;
pub use pause::pause;
pub use resume::resume;

use crate::runtime::cli::Command;
use anyhow::Result;

pub fn execute(cmd: Command) -> Result<()> {
    match cmd {
        Command::Create { id, bundle } => create(&id, &bundle),
        Command::Start { id } => start(&id),
        Command::State { id } => state(&id),
        Command::Kill { id, signal } => kill(&id, signal),
        Command::Delete { id } => delete(&id),
        Command::Pause { id } => pause(&id),
        Command::Resume { id } => resume(&id),
    }
}
