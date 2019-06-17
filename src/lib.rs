#![cfg(windows)]
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
//!     let _ = Master::new(Handler).run();
//! }
//! ```
use std::io;

use windows_win::{
    raw,
    Window,
    Messages
};

use winapi::um::winuser::{
    AddClipboardFormatListener,
    RemoveClipboardFormatListener
};

///Describes Clipboard handler
pub trait ClipboardHandler {
    ///Callback to call on clipboard change.
    fn on_clipboard_change(&mut self) -> CallbackResult;
    ///Callback to call on when error happens in master.
    fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
        CallbackResult::StopWithError(error)
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

///Clipboard master.
///
///Tracks changes of clipboard and invokes corresponding callbacks.
pub struct Master<H> {
    handler: H
}

///Clipboard listener guard.
///
///On drop unsubscribes window from listening on clipboard changes
pub struct ClipboardListener(winapi::shared::windef::HWND);

impl ClipboardListener {
    #[inline]
    ///Subscribes window to clipboard changes.
    pub fn new(window: &Window) -> io::Result<Self> {
        let window = window.inner();
        unsafe {
            if AddClipboardFormatListener(window) != 1 {
                Err(io::Error::last_os_error())
            } else {
                Ok(ClipboardListener(window))
            }
        }
    }
}

impl Drop for ClipboardListener {
    fn drop(&mut self) {
        unsafe {
            RemoveClipboardFormatListener(self.0);
        }
    }
}

impl<H: ClipboardHandler> Master<H> {
    ///Creates new instance.
    pub fn new(handler: H) -> Self {
        Master {
            handler
        }
    }

    ///Starts Master by creating dummy window and listening clipboard update messages.
    pub fn run(&mut self) -> io::Result<()> {
        let window = Window::from_builder(raw::window::Builder::new().class_name("STATIC").parent_message())?;

        let _guard = ClipboardListener::new(&window)?;

        let mut result = Ok(());

        for msg in Messages::new().window(Some(window.inner())).low(Some(797)).high(Some(797)) {
            match msg {
                Ok(_) => {
                    match self.handler.on_clipboard_change() {
                        CallbackResult::Next => (),
                        CallbackResult::Stop => break,
                        CallbackResult::StopWithError(error) => {
                            result = Err(error);
                            break;
                        }

                    }
                },
                Err(error) => {
                    match self.handler.on_clipboard_error(error) {
                        CallbackResult::Next => (),
                        CallbackResult::Stop => break,
                        CallbackResult::StopWithError(error) => {
                            result = Err(error);
                            break;
                        }
                    }
                }
            }
        }

        result
    }
}
