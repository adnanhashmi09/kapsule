#![cfg(target_os = "linux")]

use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::Pid;

pub fn wait_for_child(pid: Pid) -> anyhow::Result<WaitStatus> {
    waitpid(pid, None).map_err(|e| anyhow::anyhow!("waitpid failed: {}", e))
}
