extern crate clap;
extern crate clap_complete;
extern crate clap_mangen;
extern crate clippy;
extern crate itertools;

use std::{env, fs::write};

use clap::CommandFactory;
use clap_complete::Shell;
use clap_mangen::Man;
use itertools::Either;

use clippy::{cli::ClippyCli, commands::completions::write_to_config, prelude::Result};

fn main() -> Result<()> {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }

    Ok(())
}

fn try_main() -> Result<()> {
    let task = env::args().nth(1);

    match task.as_deref() {
        Some("mangen") => man_gen()?,
        Some("completions") => {
            for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
                let out_dir = env!("CARGO_MANIFEST_DIR");

                write_to_config(
                    shell,
                    &mut ClippyCli::command(),
                    Either::Right(out_dir.to_string()),
                )
                .unwrap();
            }
        },
        _ => return Ok(()),
    }
    Ok(())
}

fn man_gen() -> Result<()> {
    let out_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/clippy.man");
    let man = Man::new(ClippyCli::command());
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    dbg!(out_dir);
    write(out_dir, buffer)?;

    Ok(())
}
