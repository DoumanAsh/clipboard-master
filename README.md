# clipboard-master

[![Build status](https://ci.appveyor.com/api/projects/status/b6qd83x9p5ej3n2j/branch/master?svg=true)](https://ci.appveyor.com/project/DoumanAsh/clipboard-master/branch/master)

Clipboard monitoring utilities.

## Clipboard Master Library

This project exports `Master` struct that provides simple way to handle clipboard updates.

Example:
```rust
fn callback() -> CallbackResult {
    println!("Clipboard change happened!");
    CallbackResult::Next
}

fn error_callback(error: io::Error) -> CallbackResult {
    println!("Error: {}", error);
    CallbackResult::Next
}

fn main() {
    let _ = Master::new(callback, error_callback).run()
}
```

## Clipboard Master CLI

Simple monitor of clipboard content.
Following actions are performed:
- Add magnet link to default torrent client.
- Trim clipboard content

### Usage

```
USAGE: cp-master [flags]

Starts monitoring Clipboard changes

Flags:
  -h, --help    - Prints this message.
  -m, --magnet  - Starts torrent client when detecting magnet URI.
```
