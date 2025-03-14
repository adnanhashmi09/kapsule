use std::{env, os::unix::fs::chroot, path::Path, process::Command};

use nix::{
    mount::{mount, umount, MsFlags},
    unistd::sethostname,
};

pub fn container(args: &[String]) {
    sethostname("container_masti");

    chroot("/ubuntu-fs").expect("Failed to chroot");
    let root = Path::new("/");
    std::env::set_current_dir(&root).expect("failed to chdir");

    // Now mount proc
    mount::<str, str, str, str>(Some("proc"), "proc", Some("proc"), MsFlags::MS_NOSUID, None)
        .expect("Failed to mount proc");

    let mut cmd = Command::new(&args[0]);
    cmd.args(&args[1..]);
    cmd.stdin(std::process::Stdio::inherit());
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());

    match cmd.spawn() {
        Ok(mut child) => match child.wait() {
            Ok(status) => {
                if status.success() {
                    println!("Command Succeeded");
                } else {
                    println!("Command exited with an error")
                }
            }
            Err(err) => eprintln!("Error waiting for process {}", err),
        },
        Err(err) => eprintln!("Command failed to spawn"),
    }

    umount("proc");
}
