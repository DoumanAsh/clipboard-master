# clipboard-master

![](https://github.com/DoumanAsh/clipboard-master/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/clipboard-master.svg)](https://crates.io/crates/clipboard-master)
[![Docs.rs](https://docs.rs/clipboard-master/badge.svg)](https://docs.rs/clipboard-master/*/x86_64-pc-windows-msvc/clipboard_master/)

Clipboard monitoring library.

## Supported platforms

- Windows - uses dummy window to receive messages when clipboard changes;
- Linux - uses [x11_clipboard](https://github.com/quininer/x11-clipboard)
- MacOS - uses polling via `NSPasteboard::changeCount` as there is no event notification.

## Clipboard Master Library

This project exports `Master` struct that provides simple way to handle clipboard updates.

Example:

```rust
extern crate clipboard_master;

use clipboard_master::{Master, ClipboardHandler, CallbackResult};

use std::io;

struct Handler;

impl ClipboardHandler for Handler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        println!("Clipboard change happened!");
        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
        eprintln!("Error: {}", error);
        CallbackResult::Next
    }
}

fn main() {
    let mut master = Master::new(Handler).expect("create new monitor");

    let shutdown = master.shutdown_channel();
    std::thread::spawn(move || {
        std::thread::sleep(core::time::Duration::from_secs(1));
        println!("I did some work so time to finish...");
        shutdown.signal();
    });
    //Working until shutdown
    master.run().expect("Success");
}
```
