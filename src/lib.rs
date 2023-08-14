//! Clipboard master
//!
//! Provides simple way to track updates of clipboard.
//!
//! ## Example:
//!
//! ```rust,no_run
//! extern crate clipboard_master;
//!
//! use clipboard_master::{Master, ClipboardHandler, CallbackResult};
//!
//! use std::io;
//!
//! struct Handler;
//!
//! impl ClipboardHandler for Handler {
//!     fn on_clipboard_change(&mut self) -> CallbackResult {
//!         println!("Clipboard change happened!");
//!         CallbackResult::Next
//!     }
//!
//!     fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
//!         eprintln!("Error: {}", error);
//!         CallbackResult::Next
//!     }
//! }
//!
//! fn main() {
//!     let _ = Master::new(Handler).expect("create new monitor").run();
//! }
//! ```

#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]
#![cfg_attr(rustfmt, rustfmt_skip)]

use std::io;

mod master;
pub use master::{Master, Shutdown};

///Describes Clipboard handler
pub trait ClipboardHandler {
    ///Callback to call on clipboard change.
    fn on_clipboard_change(&mut self) -> CallbackResult;
    ///Callback to call on when error happens in master.
    fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
        CallbackResult::StopWithError(error)
    }

    #[inline(always)]
    ///Returns sleep interval for polling implementations (e.g. Mac).
    ///
    ///Default value is 500ms
    fn sleep_interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(500)
    }
}

///Possible return values of callback.
pub enum CallbackResult {
    ///Wait for next clipboard change.
    Next,
    ///Stop handling messages.
    Stop,
    ///Special variant to propagate IO Error from callback.
    StopWithError(io::Error)
}

impl Shutdown {
    ///Signals shutdown
    pub fn signal(self) {
        drop(self);
    }
}
