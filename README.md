# clipboard-master

[![Build Status](https://dev.azure.com/DoumanAsh/clipboard-master/_apis/build/status/DoumanAsh.clipboard-master?branchName=master)](https://dev.azure.com/DoumanAsh/clipboard-master/_build/latest?definitionId=5&branchName=master)
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
    let _ = Master::new(Handler).run();
}
```
