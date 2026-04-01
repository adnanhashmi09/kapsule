#![cfg(target_os = "linux")]

use crate::runtime::namespaces::{
    clone_with_stack, set_hostname, setup_mounts, wait_for_child, CloneConfig, CloneFlags,
};
use crate::runtime::state::{StateManager, Status};
use std::os::unix::process::CommandExt;
use std::path::Path;

pub fn start(id: &str) -> anyhow::Result<()> {
    if !StateManager::exists(id) {
        anyhow::bail!("container {} does not exist", id);
    }

    let mut state = StateManager::load(id)?;

    if state.status != Status::Created {
        anyhow::bail!("container {} is not in created state", id);
    }

    let bundle_path = state.bundle.to_string_lossy().to_string();
    let spec = crate::runtime::spec::RuntimeSpec::load_config(&bundle_path)?;

    let flags = CloneFlags::CLONE_NEWUTS
        | CloneFlags::CLONE_NEWPID
        | CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWIPC;

    let hostname = spec.hostname.clone();
    let rootfs_path = Path::new(&bundle_path).join(&spec.root.path);
    let program = spec.process.args[0].clone();
    let args = if spec.process.args.len() > 1 {
        spec.process.args[1..].to_vec()
    } else {
        vec![]
    };
    let cwd = if spec.process.cwd.is_empty() {
        "/".to_string()
    } else {
        spec.process.cwd.clone()
    };

    let child_fn: Box<dyn FnOnce() -> i32 + Send + 'static> = Box::new(move || {
        if !hostname.is_empty() {
            let _ = set_hostname(&hostname);
        }

        let rootfs = Path::new(&rootfs_path);
        if let Err(e) = setup_mounts(rootfs) {
            eprintln!("setup_mounts failed: {}", e);
            return 1;
        }

        let _ = std::env::set_current_dir(&cwd);
        for env in &spec.process.env {
            if let Some((k, v)) = env.split_once('=') {
                std::env::set_var(k, v);
            }
        }

        std::process::Command::new(&program)
            .args(&args)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .stdin(std::process::Stdio::inherit())
            .exec();

        eprintln!("exec failed for {}", program);
        1
    });

    let child_pid = clone_with_stack(CloneConfig {
        flags,
        child_fn: Some(child_fn),
    })?;

    state.pid = child_pid.as_raw() as u32;
    state.status = Status::Running;
    StateManager::save(&state)?;
    StateManager::write_pidfile(id, state.pid)?;

    let _status = wait_for_child(child_pid);

    state.status = Status::Stopped;
    StateManager::save(&state)?;

    println!("started");
    Ok(())
}
