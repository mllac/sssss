use std::path::PathBuf;

use clap::Parser;

#[derive(
    clap::Subcommand,
    Debug,
)]

pub enum Cmd {
    Track {
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    Sync,
}

pub fn args() -> Args {
    Args::parse()
}

#[derive(
    clap::Parser,
    Debug,
)]

#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Cmd,

    #[arg(
        default_value = "false",
        short,
        long,
    )]
    pub init: bool,
}
