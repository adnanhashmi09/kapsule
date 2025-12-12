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

use sys::sysinfo;

use crate::layers::ContainerImageFetcher;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide a command.");
        exit(EXIT_INSUFFICIENT_ARGS);
    }
    let image_fetcher = ContainerImageFetcher::new("redis").await.unwrap();
    image_fetcher.fetch_image().await.unwrap();

    // let a = sysinfo::get_platform_information().unwrap();
    // println!("{:#?}", a);

    return;

    match args[1].as_str() {
        "run" => run(),
        _ => {
            println!("'{}' not a valid command", args[1]);
            exit(EXIT_INVALID_ARG);
        }
    }
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
