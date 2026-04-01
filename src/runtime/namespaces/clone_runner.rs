#![cfg(target_os = "linux")]

use nix::sched::CloneFlags;
use nix::unistd::Pid;

pub struct CloneConfig {
    pub flags: CloneFlags,
    pub child_fn: Option<Box<dyn FnOnce() -> i32 + Send + 'static>>,
}

pub fn clone_with_stack(config: CloneConfig) -> anyhow::Result<Pid> {
    let stack_size = 8 * 1024 * 1024;

    let mut stack = vec![0u8; stack_size];
    let stack_slice = &mut stack[..];

    let mut child_fn_opt = config.child_fn;

    let boxed_fn: Box<dyn FnMut() -> isize + Send + 'static> = Box::new(move || {
        if let Some(child_fn) = child_fn_opt.take() {
            let code = child_fn();
            code as isize
        } else {
            -1isize
        }
    });

    let result = unsafe {
        nix::sched::clone(
            boxed_fn,
            stack_slice,
            config.flags,
            Some(libc::SIGCHLD),
        )
    };

    match result {
        Ok(pid) => Ok(pid),
        Err(e) => {
            anyhow::bail!("clone failed: {}", e);
        }
    }
}
