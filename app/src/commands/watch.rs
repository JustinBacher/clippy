use std::process::{Command, Stdio};

use anyhow::Result;
use clap::Parser;

use super::ClippyCommand;
use crate::cli::ClippyCli;

/// Starts daemon to watch for clipboard events
#[derive(Parser, Debug, PartialEq)]
pub struct Watch {
    board: Option<String>,
}

impl ClippyCommand for Watch {
    fn execute(&self, _: &ClippyCli) -> Result<()> {
        run_in_background("clippy_daemon", &[])?;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn run_in_background(command: &str, args: &[&str]) -> Result<()> {
    Command::new(command)
        .args(args)
        .creation_flags(
            winapi::um::winbase::DETACHED_PROCESS | winapi::um::winbase::CREATE_NEW_PROCESS_GROUP,
        )
        .spawn()?;
    Ok(())
}

#[cfg(target_family = "unix")]
fn run_in_background(command: &str, args: &[&str]) -> Result<()> {
    Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}
