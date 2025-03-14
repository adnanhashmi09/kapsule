use std::{
    env,
    os::unix::process::CommandExt,
    process::{exit, Command},
};

use nix::{
    libc::SIGCHLD,
    sched::{clone, CloneFlags},
    sys::wait::WaitPidFlag,
};

use crate::container;

pub fn run() {
    let args: Vec<String> = env::args().collect();
    // let mut cmd = Command::new("/proc/self/exe");
    // cmd.arg("child");
    // cmd.args(&args[2..]);
    // cmd.stdin(std::process::Stdio::inherit());
    // cmd.stdout(std::process::Stdio::inherit());
    // cmd.stderr(std::process::Stdio::inherit());

    unsafe {
        use nix::sys::wait::{waitpid, WaitStatus};
        // TODO: Figure out the correct stack size
        let mut stack = [0u8; 2 * 1024 * 1024];
        let flags = CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS;

        match clone(
            Box::new(move || {
                // cmd.exec();
                container::container(&args[2..]);
                std::process::exit(0);
            }),
            &mut stack,
            flags,
            Some(SIGCHLD),
        ) {
            Ok(pid) => {
                match waitpid(pid, Some(WaitPidFlag::__WALL)) {
                    Ok(_) => {
                        println!("Child exited.");
                        // Handle other statuses if needed
                    }
                    Err(err) => {
                        eprintln!("Error while waiting for child process: {}", err);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error during clone: {}", err);
            }
        }
    }
    println!("End Run");
}
