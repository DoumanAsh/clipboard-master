//! Magnet links downloader module
use std::process::Command;

#[inline(always)]
pub fn is_applicable<T: AsRef<str>>(text: T) -> bool {
    text.as_ref().starts_with("magnet:")
}

#[inline(always)]
pub fn run<T: AsRef<str>>(magnet_uri: T) {
    Command::new("powershell").arg("-NoProfile")
                              .arg("-c")
                              .arg(format!("start {}", magnet_uri.as_ref()))
                              .status()
                              .expect("Unable to start powershell");
}
