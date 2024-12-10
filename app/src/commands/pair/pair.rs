use anyhow::{anyhow, Result};
use clap::Parser;
use promkit::preset::confirm::Confirm;

use super::{create_invite_code, get_local_ip};
use crate::{
    cli::ClippyCli,
    commands::{pair::decrypt_code, ClippyCommand},
};

#[derive(Parser, Debug, PartialEq)]
/// Wipes all clips from clipboard
pub struct Pair {
    #[arg(trailing_var_arg(true))]
    code: Option<Vec<String>>,
}

impl ClippyCommand for Pair {
    fn execute(&self, _: &ClippyCli) -> Result<()> {
        if let Some(ref given_code) = self.code {
            let code = decrypt_code(&given_code.join(" "))?;

            if get_local_ip()? == code.local_ip {
                return Err(anyhow!("This code is from this device. Please use this code on another device to link them."));
            }

            println!("Other device ips: {code:?}");

            // Reach out to other device to setup dht
        } else {
            let code = create_invite_code()?;

            // TODO: implement the actual copying to the clipboard
            println!(
                "The code below has been copied to your clipboard, please use this code to link another device to this one. \\n
                \n\t {code} \n\nThis can be done on the other device using:\n\n\tclippy pair {code}"
            );
        }
        Ok(())
    }
}
