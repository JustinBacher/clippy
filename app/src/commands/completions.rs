use std::fs::File;

use clap::{value_parser, Command, CommandFactory, Parser, ValueEnum, ValueHint::AnyPath};
use clap_complete::aot::{generate, Generator, Shell};
use itertools::Either;
use serde::{Deserialize, Serialize};

use super::ClippyCommand;
use crate::{
    cli::{ClippyCli, APP_NAME},
    prelude::Result,
    utils::get_config_path,
};

#[derive(Serialize, Deserialize, ValueEnum, Clone, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LinuxShells {
    Bash,
    Fish,
    Zsh,
}

/// Generate shell completions for clippy
#[derive(Parser, Debug, PartialEq)]
pub struct GenCompletions {
    /// Name of the shell to generate completions for.
    #[arg(value_parser = value_parser!(LinuxShells))]
    shell: LinuxShells,
    /// The location to output the completions file. Do not include the file_name or extention.
    #[arg(short, long, value_hint(AnyPath), default_value = get_config_path("clippy", "").unwrap())]
    output: String,
}

impl ClippyCommand for GenCompletions {
    fn execute(&self, _: &ClippyCli) -> Result<()> {
        let path = write_to_config(
            match self.shell {
                LinuxShells::Bash => Shell::Bash,
                LinuxShells::Fish => Shell::Fish,
                LinuxShells::Zsh => Shell::Zsh,
            },
            &mut ClippyCli::command(),
            Either::Left(self),
        )?;

        Ok(println!(
            "Wrote completions to \n\n\t{path:?}\n\n\
            \
            If you did not specify an output location please move this file into your \
            completions folder or it's contents to your completions file",
        ))
    }
}

pub fn write_to_config(
    shell: Shell,
    cmd: &mut Command,
    args: Either<&GenCompletions, String>,
) -> Result<String> {
    let config_path = match args {
        Either::Left(a) => a.output.clone(),
        Either::Right(a) => a,
    };

    shell.file_name("clippy_completions");
    generate(shell, cmd, APP_NAME, &mut File::create(&config_path)?);

    Ok(config_path)
}
