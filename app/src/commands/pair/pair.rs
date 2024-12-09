use anyhow::Result;
use clap::Parser;

use super::create_invite_code;
use crate::{
    cli::ClippyCli,
    commands::{pair::decrypt_code, ClippyCommand},
};

#[derive(Parser, Debug, PartialEq)]
/// Wipes all clips from clipboard
pub struct Pair {}

impl ClippyCommand for Pair {
    fn execute(&self, _: &ClippyCli) -> Result<()> {
        let code = create_invite_code()?;
        println!("{}", code);
        println!("{:?}", decrypt_code(&code)?);
        Ok(())
    }
}
