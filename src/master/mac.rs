use std::io;
use crate::{ClipboardHandler, CallbackResult, Master};

use objc::runtime::{Object, Class};
use objc_id::{Id};

impl<H: ClipboardHandler> Master<H> {
    ///Starts Master by polling clipboard for change
    pub fn run(&mut self) -> io::Result<()> {
        use objc::{msg_send, sel, sel_impl};

        let cls = match Class::get("NSPasteboard") {
            Some(cls) => cls,
            None => return Err(io::Error::new(io::ErrorKind::Other, "Unable to create mac pasteboard")),
        };
        let pasteboard: *mut Object = unsafe { msg_send![cls, generalPasteboard] };

        if pasteboard.is_null() {
            return Err(io::Error::new(io::ErrorKind::Other, "Unable to create mac pasteboard"));
        }

        let pasteboard: Id<Object> = unsafe { Id::from_ptr(pasteboard) };

        let mut prev_count = 0;
        let mut result = Ok(());

        loop {
            let count: isize = unsafe { msg_send![pasteboard, changeCount] };

            if count == prev_count {
                std::thread::sleep(self.handler.sleep_interval());
                continue;
            }

            prev_count = count;

            match self.handler.on_clipboard_change() {
                CallbackResult::Next => (),
                CallbackResult::Stop => break,
                CallbackResult::StopWithError(error) => {
                    result = Err(error);
                    break;
                }
            }
        }

        result
    }
}
