use std::fs;

use clap::{Parser};

pub static BUILD_FOLDER: &str = "build";
pub static TEST_FOLDER: &str = "tests";

#[derive(Parser,Debug,Default)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    implementation: Option<String>,
    #[arg(short, long)]
    feature: Option<String>,
    #[arg(short, long)]
    reset: bool,
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[arg(short, long)]
    generate: bool,
    #[arg(short, long)]
    build: bool,
}

pub struct SanitizedArgs {
    pub(crate) implementation: String,
    pub(crate) feature: String
}

pub fn handle_args() -> SanitizedArgs {
    let args = Args::parse();

    let mut implementation = String::new();
    if let Some(i) = args.implementation.as_deref() {
        implementation = i.to_string();
    }

    let mut feature = String::new();
    if let Some(f) = args.feature.as_deref() {
        feature = f.to_string();
    }

    match fs::create_dir(BUILD_FOLDER) {
        Err(_) => {
            if args.reset {
                fs::remove_dir_all(BUILD_FOLDER).unwrap();
                fs::create_dir(BUILD_FOLDER).unwrap();
            }
        }
        _ => {}
    }

    return SanitizedArgs {
        implementation,
        feature
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    // match args.verbose {
    //     0 => println!("Debug mode is off"),
    //     1 => println!("Debug mode is kind of on"),
    //     2 => println!("Debug mode is on"),
    //     _ => println!("Don't be crazy"),
    // }
}