use std::io;

use windows_win::{
    raw,
    Window,
    Messages
};

use winapi::shared::windef::HWND;

use winapi::um::winuser::{
    AddClipboardFormatListener,
    RemoveClipboardFormatListener,
    PostMessageW,
    WM_CLIPBOARDUPDATE,
};

use crate::{ClipboardHandler, CallbackResult};

const CLOSE_PARAM: isize = -1;

///Shutdown channel
///
///On drop requests shutdown to gracefully close clipboard listener as soon as possible.
pub struct Shutdown {
    window: HWND,
}

unsafe impl Send for Shutdown {}

impl Drop for Shutdown {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            PostMessageW(self.window, WM_CLIPBOARDUPDATE, 0, CLOSE_PARAM)
        };
    }
}

///Clipboard listener guard.
///
///On drop unsubscribes window from listening on clipboard changes
pub struct ClipboardListener(HWND);

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

///Clipboard master.
///
///Tracks changes of clipboard and invokes corresponding callbacks.
///
///# Platform notes:
///
///- On `windows` it creates dummy window that monitors each clipboard change message.
pub struct Master<H> {
    handler: H,
    window: Window,
}

impl<H: ClipboardHandler> Master<H> {
    #[inline(always)]
    ///Creates new instance.
    pub fn new(handler: H) -> io::Result<Self> {
        let window = Window::from_builder(raw::window::Builder::new().class_name("STATIC").parent_message())?;

        Ok(Self {
            handler,
            window
        })
    }

    #[inline(always)]
    ///Creates shutdown channel.
    pub fn shutdown_channel(&self) -> Shutdown {
        Shutdown {
            window: self.window.inner()
        }
    }

    ///Starts Master by creating dummy window and listening clipboard update messages.
    pub fn run(&mut self) -> io::Result<()> {
        let _guard = ClipboardListener::new(&self.window)?;

        let mut result = Ok(());

        for msg in Messages::new().window(Some(self.window.inner())).low(Some(WM_CLIPBOARDUPDATE)).high(Some(WM_CLIPBOARDUPDATE)) {
            match msg {
                Ok(msg) => match msg.id() {
                    WM_CLIPBOARDUPDATE => {
                        let msg = msg.inner();

                        //Shutdown requested
                        if msg.lParam == CLOSE_PARAM {
                            break;
                        }

                        match self.handler.on_clipboard_change() {
                            CallbackResult::Next => (),
                            CallbackResult::Stop => break,
                            CallbackResult::StopWithError(error) => {
                                result = Err(error);
                                break;
                            }
                        }
                    },
                    _ => panic!("Unexpected message"),
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
