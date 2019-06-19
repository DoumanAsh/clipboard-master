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

use crate::{ClipboardHandler, CallbackResult, Master};

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
