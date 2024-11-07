use super::ClippyCommand;
use crate::{cli::Cli, prelude::Result};
use clap::{Parser, ValueEnum};
use serde::Serialize;

#[derive(ValueEnum, Parser, Clone, PartialEq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Shells {
    Bash,
    Fish,
    Zsh,
}

#[derive(Parser, Debug, PartialEq)]
/// Lists all stored clips in clipboard
pub(crate) struct GenCompletions {
    #[arg(long("generate-completions"), visible_aliases = ["gen-completions"], action)]
    shell: Shells,
}

impl ClippyCommand for GenCompletions {
    fn execute(&self, _: &Cli) -> Result<()> {
        unimplemented!()
    }
}
