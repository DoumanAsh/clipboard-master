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
//! ./cp-master
//! ```
extern crate clipboard_master;
extern crate clipboard_win;

use std::io;
use std::process::exit;

use clipboard_master::{
    Master,
    CallbackResult,
};

use clipboard_win::{
    Clipboard,
    formats
};

mod process;

fn callback() -> CallbackResult {
    match Clipboard::new() {
        Ok(clip) => {
            if Clipboard::is_format_avail(formats::CF_UNICODETEXT) {
                match clip.get_string() {
                    Ok(content) => {
                        if process::magnet::is_applicable(&content) {
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
        }
        Err(error) => {
            println!("Failed to open clipboard. Error: {}", error);
        }
    }

    CallbackResult::Next
}

fn error_callback(error: io::Error) -> CallbackResult {
    println!("Error: {}", error);
    CallbackResult::Next
}

fn main() {
    let result = Master::new(callback, error_callback).run();

    match result {
        Ok(_) => (),
        Err(error) => {
            println!("Aborted. Error: {}", error);
            exit(1)
        }
    }
}
