use super::ClippyCommand;
use crate::{
    cli::Cli,
    prelude::Result,
    utils::{
        database::{remove_duplicates, TABLE_DEF},
        formatting::trim,
    },
};
use camino::Utf8PathBuf;
use chrono::Local;
use clap::Parser;
use redb::Database;
use std::{
    env,
    io::{stdin, Read, Stdin},
    mem::size_of_val,
};

const FIVE_MEGABYTES: usize = 5e6 as usize;

//#[derive(ValueEnum, Parser, Clone, Default, PartialEq, Debug, Serialize)]
//#[serde(rename_all = "lowercase")]
//pub enum ClipboardState {
//    #[default]
//    Nil,
//    Data,
//    Clear,
//    Sensitive,
//}

#[derive(Parser, Debug, PartialEq)]
/// Reads a clip from stdin and remembers it for later recall
pub(crate) struct Store {
    //#[arg(env, action=ArgAction::Set, hide(true))]
    //clipboard_state: ClipboardState,
}

impl ClippyCommand for Store {
    fn execute(&self, args: &Cli) -> Result<()> {
        match env::var("CLIPBOARD_STATE").unwrap().as_str() {
            "sensitive" => todo!("Use non-persistent storage for secrets"),
            "clear" | "nil" => {
                println!("Should be warning");
                warn!(
                    "
                    Clippy does not implement \"nil\" or \"clear\" for `CLIPBOARD_STATE`
                    Environment Variable.
                    Please use clippy with
                    [wl-clipboard](https://github.com/bugaevc/wl-clipboard)"
                );
            }
            "data" => {
                let db_path = &args.db_path;
                store(db_path, &mut stdin())?;
                remove_duplicates(db_path, args.duplicates, args.keep)?;
            }
            _ => {}
        }
        Ok(())
    }
}

fn store(db_path: &Utf8PathBuf, input: &mut Stdin) -> Result<()> {
    let mut payload = Vec::new();
    input.read_to_end(&mut payload)?;

    if size_of_val(&payload) > FIVE_MEGABYTES || trim(&payload).len() == 0 {
        ()
    }

    println!("db path: {:?}", &db_path);
    let db = Database::create(&db_path)?;
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(TABLE_DEF)?;
        table.insert(Local::now().timestamp_millis(), payload.to_owned())?;
    }
    tx.commit()?;

    Ok(())
}
