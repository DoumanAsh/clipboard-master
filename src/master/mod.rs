#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::{Master, Shutdown};

#[cfg(all(unix, not(any(target_os="macos", target_os="ios", target_os="android", target_os="emscripten"))))]
mod x11;
#[cfg(all(unix, not(any(target_os="macos", target_os="ios", target_os="android", target_os="emscripten"))))]
pub use x11::{Master, Shutdown};

#[cfg(target_os="macos")]
mod mac;
#[cfg(target_os="macos")]
pub use mac::{Master, Shutdown};
