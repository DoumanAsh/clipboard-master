#![cfg(windows)]
//! Clipboard master
//!
//! Monitors clipboard and process its content on each change.
//!
//! ## Processors
//!
//! ### Trim text
//!
//! Content of clipboard trimmed of right whitespaces on each line
//!
//! ### Start torrent with magnet uri
//!
//! If magnet uri is detected, then master attempts to start torrent with it.
//!
//! ## Usage
//!
//! ```
//! USAGE: cp-master [flags]
//!
//! Starts monitoring Clipboard changes
//!
//! Flags:
//!   -h, --help    - Prints this message.
//!   -m, --magnet  - Starts torrent client when detecting magnet URI.
//! ```

extern crate clipboard_master;
extern crate clipboard_win;

use std::io;
use std::process::exit;

use clipboard_master::{
    Master,
    ClipboardHandler,
    CallbackResult,
};

use clipboard_win::{
    Clipboard,
    formats
};

mod process;
mod cli;

pub struct Handler {
    args: cli::Parser
}

impl ClipboardHandler for Handler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        const RES: CallbackResult = CallbackResult::Next;

        if !Clipboard::is_format_avail(formats::CF_UNICODETEXT) {
            return RES;
        }

        match Clipboard::new() {
            Ok(clip) => {
                match clip.get_string() {
                    Ok(content) => {
                        if self.args.flags.magnet && process::magnet::is_applicable(&content) {
                            println!(">>>Run torrent client on uri: {}", &content);
                            process::magnet::run(&content);
                        }
                        else if let Some(new_content) = process::trim::lines(&content) {
                            if let Err(error) = clip.set_string(&new_content) {
                                println!("Failed to set clipboard content. Error: {}", error);
                            }
                            else {
                                println!(">>>Trimmed clipboard");
                            }
                        }
                    }
                    Err(error) => {
                        println!("Failed to get clipboard content. Error: {}", error);
                    }
                }
            }
            Err(error) => {
                println!("Failed to open clipboard. Error: {}", error);
            }
        }

        RES
    }

    fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
        eprintln!("Error: {}", error);
        CallbackResult::Next
    }
}

fn main() {
    let args = match cli::Parser::new() {
        Ok(args) => args,
        Err(error) => {
            println!("{}", error);
            exit(1);
        }
    };

    if args.flags.help {
        println!("{}", args.usage());
        return;
    }

    let handler = Handler {
        args,
    };

    match Master::new(handler).run() {
        Ok(_) => (),
        Err(error) => {
            println!("Aborted. Error: {}", error);
            exit(1)
        }
    }
}
