#![cfg(windows)]
//! Clipboard master
//!
//! Provides simple way to track updates of clipboard.
//!
//! ## Example:
//!
//! ```rust
//! fn callback() -> CallbackResult {
//!     println!("Clipboard change happened!");
//!     CallbackResult::Next
//! }
//!
//! fn error_callback(error: io::Error) -> CallbackResult {
//!     println!("Error: {}", error);
//!     CallbackResult::Next
//! }
//!
//! fn main() {
//!     let _ = Master::new(callback, error_callback).run()
//! }
//! ```
extern crate windows_win;
extern crate clipboard_win;
extern crate winapi;

use std::io;
use std::ops::FnMut;

use windows_win::{
    raw,
    Window,
    Messages
};

use winapi::um::winuser::{
    AddClipboardFormatListener,
    RemoveClipboardFormatListener
};

///Possible return values of callback.
pub enum CallbackResult {
    ///Wait for next clipboard change.
    Next,
    ///Stop handling messages.
    Stop,
    ///Special variant to propagate IO Error from callback.
    StopWithError(io::Error)
}

///Default error callback that stops Master and propagates error in return value of `run()`
pub fn default_error(error: io::Error) -> CallbackResult {
    CallbackResult::StopWithError(error)
}

///Clipboard master.
///
///Tracks changes of clipboard and invokes corresponding callbacks.
pub struct Master<OK, ERR>
    where OK: FnMut() -> CallbackResult,
          ERR: FnMut(io::Error) -> CallbackResult
{
    cb_ok: OK,
    cb_err: ERR
}

impl<OK, ERR> Master<OK, ERR>
    where OK: FnMut() -> CallbackResult,
          ERR: FnMut(io::Error) -> CallbackResult
{
    ///Creates new instance.
    pub fn new(cb: OK, cb_err: ERR) -> Self {
        Master {
            cb_ok: cb,
            cb_err: cb_err
        }
    }

    ///Starts Master by creating dummy window and listening clipboard update messages.
    pub fn run(&mut self) -> io::Result<()> {
        let window = Window::from_builder(raw::window::Builder::new().class_name("STATIC").parent_message())?;

        unsafe {
            if AddClipboardFormatListener(window.inner()) != 1 {
                return Err(io::Error::last_os_error());
            };
        }

        for msg in Messages::new().window(Some(window.inner())).low(Some(797)).high(Some(797)) {
            match msg {
                Ok(_) => {
                    match (self.cb_ok)() {
                        CallbackResult::Next => (),
                        CallbackResult::Stop => break,
                        CallbackResult::StopWithError(error) => return Err(error),

                    }
                },
                Err(error) => {
                    match (self.cb_err)(error) {
                        CallbackResult::Next => (),
                        CallbackResult::Stop => break,
                        CallbackResult::StopWithError(error) => return Err(error),
                    }
                }
            }
        }

        unsafe {
            RemoveClipboardFormatListener(window.inner());
        }

        Ok(())
    }
}
