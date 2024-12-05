extern crate anyhow;
extern crate clap;
extern crate clap_complete;
extern crate clap_mangen;
extern crate clippy;
extern crate clippy_daemon;
extern crate itertools;

use std::{collections::HashMap, env, fs::write, process::exit};

use anyhow::Result;
use clap::CommandFactory;
use clap_complete::Shell::*;
use clap_mangen::Man;
use clippy::{
    cli::ClippyCli,
    commands::completions::{LinuxShells, write_to_config},
};
use clippy_daemon::utils::config::*;
use itertools::Either::Right;
use serde::Serialize;
use toml;

fn main() -> Result<()> {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        exit(-1);
    }
    Ok(())
}

fn try_main() -> Result<()> {
    match env::args().nth(1).as_deref() {
        Some("man") => man_gen()?,
        Some("completions") => {
            let out_dir = env!("CARGO_MANIFEST_DIR");
            for shell in [Bash, Fish, Zsh] {
                write_to_config(shell, &mut ClippyCli::command(), Right(&out_dir))?;
            }
        },
        Some("config") => {
            let config = Config {
                general: Some(General::default()),
                clipboard: HashMap::from([("general".to_string(), Clipboard::default())]),
            };
            println!("{}", toml::to_string(&config)?);
        },
        _ => panic!("Invalid argument passed"),
    }
    Ok(())
}

fn man_gen() -> Result<()> {
    let out_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/clippy.man");
    let man = Man::new(ClippyCli::command());
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    Ok(write(out_dir, buffer)?)
}
