// #![cfg(target_os = "linux")]
use std::{env, process::exit};

mod container;
mod errors;
mod layers;
mod run;
mod runtime;
mod sys;

use errors::{EXIT_INSUFFICIENT_ARGS, EXIT_INVALID_ARG};
use reqwest::{Error, Request};
use run::run;

use crate::runtime::spec::RuntimeSpec;
use anyhow::Context;
use clap::{Parser, Subcommand};
use sys::sysinfo;

use crate::layers::ContainerImageFetcher;

#[derive(Parser)]
#[command(name = "krt")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a container
    Create {
        /// Container ID
        container_id: String,

        /// Path to OCI bundle
        path_to_bundle: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            container_id,
            path_to_bundle,
        } => {
            let spec = RuntimeSpec::load_config(&path_to_bundle)
                .context("Cannot parse bundle config.json");

            println!("{:?}", spec);
        }
    }
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     println!("Please provide a command.");
    //     exit(EXIT_INSUFFICIENT_ARGS);
    // }
    // let image_fetcher = ContainerImageFetcher::new("nginx").await.unwrap();
    // image_fetcher.fetch_image().await.unwrap();
    //
    // // let a = sysinfo::get_platform_information().unwrap();
    // // println!("{:#?}", a);
    //
    // return;
    //
    // match args[1].as_str() {
    //     "run" => run(),
    //     _ => {
    //         println!("'{}' not a valid command", args[1]);
    //         exit(EXIT_INVALID_ARG);
    //     }
    // }
}

// #[cfg(not(target_os = "linux"))]
// mod errors;
//
// #[cfg(not(target_os = "linux"))]
// fn main() {
//     use errors::EXIT_UNSUPPORTED_PLATFORM;
//
//     println!("Kapsule is only supported on Linux");
//     std::process::exit(EXIT_UNSUPPORTED_PLATFORM);
// }
