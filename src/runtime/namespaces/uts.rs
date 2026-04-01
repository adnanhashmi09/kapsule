#![cfg(target_os = "linux")]

use nix::unistd::sethostname;

pub fn set_hostname(hostname: &str) -> anyhow::Result<()> {
    sethostname(hostname).map_err(|e| anyhow::anyhow!("failed to set hostname: {}", e))
}
