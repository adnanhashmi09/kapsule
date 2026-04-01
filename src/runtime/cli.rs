use std::env;

#[derive(Debug)]
pub enum Command {
    Create { id: String, bundle: String },
    Start { id: String },
    State { id: String },
    Kill { id: String, signal: u32 },
    Delete { id: String },
    Pause { id: String },
    Resume { id: String },
}

pub fn parse_args() -> anyhow::Result<Command> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        anyhow::bail!("no command specified");
    }

    match args[1].as_str() {
        "create" => {
            if args.len() < 4 {
                anyhow::bail!("create requires <id> <bundle>");
            }
            Ok(Command::Create {
                id: args[2].clone(),
                bundle: args[3].clone(),
            })
        }
        "start" => {
            if args.len() < 3 {
                anyhow::bail!("start requires <id>");
            }
            Ok(Command::Start {
                id: args[2].clone(),
            })
        }
        "state" => {
            if args.len() < 3 {
                anyhow::bail!("state requires <id>");
            }
            Ok(Command::State {
                id: args[2].clone(),
            })
        }
        "kill" => {
            if args.len() < 4 {
                anyhow::bail!("kill requires <id> <signal>");
            }
            let signal: u32 = args[3].parse().unwrap_or(15);
            Ok(Command::Kill {
                id: args[2].clone(),
                signal,
            })
        }
        "delete" => {
            if args.len() < 3 {
                anyhow::bail!("delete requires <id>");
            }
            Ok(Command::Delete {
                id: args[2].clone(),
            })
        }
        "pause" => {
            if args.len() < 3 {
                anyhow::bail!("pause requires <id>");
            }
            Ok(Command::Pause {
                id: args[2].clone(),
            })
        }
        "resume" => {
            if args.len() < 3 {
                anyhow::bail!("resume requires <id>");
            }
            Ok(Command::Resume {
                id: args[2].clone(),
            })
        }
        cmd => {
            anyhow::bail!("unknown command: {}\n\n{}", cmd, HELP);
        }
    }
}

const HELP: &str = r#"kapsule OCI runtime

USAGE:
  kapsule <command> <container-id> [arguments]

COMMANDS:
  create <id> <bundle>   Create a container
  start <id>            Start a created container
  state <id>            Get the state of a container
  kill <id> <signal>    Send a signal to a container
  delete <id>           Delete a container
  pause <id>            Pause a container
  resume <id>           Resume a paused container

OPTIONS:
  --help, -h            Show this help
  --version, -v         Show version
"#;

pub fn print_help() {
    println!("{}", HELP);
}
