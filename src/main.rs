// #![cfg(target_os = "linux")]

// mod container;
mod errors;
mod layers;
// mod run;
mod runtime;
mod sys;

use runtime::cli::{parse_args, print_help};
use runtime::commands::execute;
use anyhow::Result;

fn main() {
    if let Err(e) = execute_cli() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn execute_cli() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("kapsule version 0.1.0");
        return Ok(());
    }

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    if args.len() < 2 {
        print_help();
        anyhow::bail!("no command specified");
    }

    let cmd = parse_args()?;
    execute(cmd)
}
