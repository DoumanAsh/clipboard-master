use std::io;
use crate::{ClipboardHandler, CallbackResult, Master};

impl<H: ClipboardHandler> Master<H> {
    ///Starts Master by waiting for any change
    pub fn run(&mut self) -> io::Result<()> {
        let mut result = Ok(());
        let clipboard = x11_clipboard::Clipboard::new().unwrap();

        loop {
            match clipboard.load_wait(clipboard.getter.atoms.primary, x11_clipboard::xcb::xproto::ATOM_ANY, clipboard.getter.atoms.property) {
                Ok(_) => match self.handler.on_clipboard_change() {
                    CallbackResult::Next => (),
                    CallbackResult::Stop => break,
                    CallbackResult::StopWithError(error) => {
                        result = Err(error);
                        break;
                    }

                },
                Err(error) => match self.handler.on_clipboard_error(io::Error::new(io::ErrorKind::Other, error)) {
                    CallbackResult::Next => (),
                    CallbackResult::Stop => break,
                    CallbackResult::StopWithError(error) => {
                        result = Err(error);
                        break;
                    }
                }
            }
        }

        result
    }
}
