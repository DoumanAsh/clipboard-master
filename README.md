# clipboard-master

[![Build status](https://ci.appveyor.com/api/projects/status/b6qd83x9p5ej3n2j/branch/master?svg=true)](https://ci.appveyor.com/project/DoumanAsh/clipboard-master/branch/master)
[![Crates.io](https://img.shields.io/crates/v/clipboard-master.svg)](https://crates.io/crates/clipboard-master)
[![Docs.rs](https://docs.rs/clipboard-master/badge.svg)](https://docs.rs/clipboard-master/*/x86_64-pc-windows-msvc/clipboard_master/)

Clipboard monitoring library.

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
