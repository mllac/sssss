#![feature(if_let_guard)]

#![allow(dead_code)]

use std::{
    io::{Read, Write},
    fs::OpenOptions,
    env,
};

use color_eyre::eyre::Result;
use prompt::Prompt;
use args::Cmd;

use ratatui::{
    prelude::Backend,
    TerminalOptions,
    Terminal,
    Viewport,
};
use sync::in_tmp;

mod prompt;
mod cred;
mod sync;
mod args;
mod git;

const REP_F: &str = "sys-remote";
const TMP_F: &str = "sys-sync";

fn term() -> Terminal<impl Backend> {
    let viewport = Viewport::Inline(8);

    ratatui::init_with_options(
        TerminalOptions {
            viewport,
        }
    )
}

fn remote(
    term: &mut Terminal<impl Backend>,
    init: bool,
) -> Result<String> {
    let path = env::temp_dir().join(REP_F);

    let existed = if !init {
        in_tmp(REP_F)
    } else {
        false
    };

    let mut sys_remote =
        OpenOptions::new()
            .create(true)
            .truncate(init)
            .write(true)
            .read(true)
            .open(path)?;

    if !existed {
        let r = Prompt::prompt(
            term,
            "remote"
        )?;

        sys_remote.write_all(
            r.as_bytes()
        )?;

        Ok(r)
    } else {
        let mut buf = String::new();

        sys_remote.read_to_string(
            &mut buf
        )?;

        Ok(buf)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = args::args();

    let mut term = term();

    let remote = remote(
        &mut term,
        args.init,
    )?;

    ratatui::restore();

    let mut store = sync::Store::new(
        TMP_F
    )?;

    match args.cmd {
        Cmd::Track { path } => {
            let path = path.unwrap_or_else(|| {
                env::current_dir()
                    .expect("failed to get current dir")
            });

            store.insert(path)?;
        },
        Cmd::Sync => {
            store.sync(remote)?;
        },
    }

    Ok(())
}
