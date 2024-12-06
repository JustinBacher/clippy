use super::ClippyCommand;
use crate::cli::ClippyCli;
use anyhow::Result;
use clap::Parser;
use clippy_daemon::database::get_db;
use mdns::{mDNSListener, Record};
use spinners::{Spinner, Spinners};
use std::{thread, time::Duration};
use tokio::runtime::Runtime;

#[derive(Parser, Debug, PartialEq)]
/// Wipes all clips from clipboard
pub struct Pair {}

impl ClippyCommand for Pair {
    fn execute(&self, args: &ClippyCli) -> Result<()> {
        let runtime = Runtime::new()?;

        // Block on the async mDNS discovery in a synchronous manner
        runtime.block_on(async {
            println!("Starting mDNS discovery...");

            // Create a spinner to show progress
            let mut spinner = Spinner::new(Spinners::Dots12, "Discovering devices...".into());

            // Start listening for discovered devices
            let mut stream = mDNSListener().await?;

            // Loop to print discovered devices and keep refreshing the spinner
            loop {
                // Check for new discovered services and print them
                if let Some(Ok(response)) = stream.next().await {
                    println!("\nDiscovered: {:?}", response);
                }

                // Refresh the spinner to keep it spinning
                spinner.spin();

                // Sleep for a bit to allow smooth spinner animation
                thread::sleep(Duration::from_millis(100));
            }

            // Return result from the async block
            Ok(())
        })?;

        Ok(())
    }
}
