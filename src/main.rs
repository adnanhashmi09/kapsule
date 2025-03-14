#![cfg(target_os = "linux")]
use std::{
    env,
    os::unix::{fs::chroot, process::CommandExt},
    path::Path,
    process::{exit, Command},
};

use nix::{
    libc::SIGCHLD,
    mount::{mount, umount, MntFlags, MsFlags},
    sched::{clone, unshare, CloneFlags},
    sys::wait::WaitPidFlag,
    unistd::sethostname,
};

mod container;
mod run;
use container::container;
use run::run;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{}", args[0]);
    if args.len() < 2 {
        println!("Please provide a command");
        exit(1);
    }

    match args[1].as_str() {
        "run" => run(),
        // "child" => container(),
        _ => {
            println!("'{}' not a valid command", args[1]);
            exit(1);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("Kapsule is only supported on Linux");
    std::process::exit(1);
}
