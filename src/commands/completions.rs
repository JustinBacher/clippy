use super::ClippyCommand;
use crate::{cli::ClippyCli, prelude::Result, utils::get_config_path};
use clap::{value_parser, Command, CommandFactory, Parser, ValueEnum, ValueHint::AnyPath};
use clap_complete::aot::{generate, Shell};
use serde::Serialize;
use std::fs::File;

#[derive(ValueEnum, Clone, PartialEq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum LinuxShells {
    Bash,
    Fish,
    Zsh,
}

#[derive(Parser, Debug, PartialEq)]
/// Lists all stored clips in clipboard
pub struct GenCompletions {
    #[arg(value_parser = value_parser!(LinuxShells))]
    shell: LinuxShells,
    #[arg(short, long, value_hint(AnyPath))]
    output: Option<String>,
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
        );

        println!(
            "Wrote completions to \n\n\t{}\n\n\
            Please move this file into your completions folder or it's contents to your completions file",
            path?
        );

        Ok(())
    }
}

fn write_to_config(shell: Shell, cmd: &mut Command) -> Result<String> {
    let config_path =
        get_config_path("clippy", &format!("{}_completions.{}", &shell, &shell)).unwrap();

    generate(
        shell,
        cmd,
        cmd.get_name().to_string(),
        &mut File::create(&config_path)?,
    );

    Ok(config_path)
}
