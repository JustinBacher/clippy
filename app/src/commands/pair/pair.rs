use std::{io::BufWriter, io::prelude::*, net::TcpStream};

use anyhow::{Result, anyhow};
use clap::Parser;
use rmp_serde::Serializer;
use serde::Serialize;

use super::{create_invite_code, get_local_ip};
use crate::{
    cli::ClippyCli,
    commands::{ClippyCommand, pair::decrypt_code},
};
use clippy_daemon::{
    database::node::{Node, NodeMessage},
    prelude::DEFAULT_PORTS,
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
                return Err(anyhow!(
                    "This code is from this device. Please use this code on another device to link them."
                ));
            }
            for port in DEFAULT_PORTS.iter() {
                if let Ok(mut stream) = TcpStream::connect((code.local_ip, *port)) {
                    let mut node = Node::new();
                    node.local_ip = code.local_ip;

                    let mut writer = BufWriter::new(Vec::new());
                    NodeMessage::JoinNetwork(node)
                        .serialize(&mut Serializer::new(&mut writer))
                        .unwrap();

                    stream.write_all(&(writer.buffer().len() as u32).to_be_bytes())?;
                    stream.write_all(writer.buffer())?;
                }
            }
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
