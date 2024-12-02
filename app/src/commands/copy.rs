use std::io::{Read, stdin};

use anyhow::Result;
use arboard::Clipboard;
use clap::Parser;

use super::ClippyCommand;
use crate::cli::ClippyCli;

#[derive(Parser, Debug, PartialEq)]
/// Reads a clip from stdin and remembers it for later recall
pub struct Copy {
    #[arg(short, long)]
    data: String,
}

impl ClippyCommand for Copy {
    fn execute(&self, _: &ClippyCli) -> Result<()> {
        let mut payload = Vec::new();
        stdin().read_to_end(&mut payload)?;

        Ok(Clipboard::new()?.set().text(std::str::from_utf8(&payload)?)?)
    }
}
