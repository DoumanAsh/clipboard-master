use crate::{CallbackResult, ClipboardHandler, Master};
use std::io;

impl<H: ClipboardHandler> Master<H> {
    ///Starts Master by waiting for any change
    pub fn run(&mut self) -> io::Result<()> {
        let clipboard = x11_clipboard::Clipboard::new();
        if let Err(error) = clipboard {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to initialize clipboard: {:?}", error),
            ));
        }
        let clipboard = clipboard.unwrap();

        loop {
            let res = clipboard.load_wait(
                clipboard.getter.atoms.clipboard,
                clipboard.getter.atoms.utf8_string,
                clipboard.getter.atoms.property,
            );
            if let Err(error) = res {
                let error = io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to load clipboard: {:?}", error),
                );

                match self.handler.on_clipboard_error(error) {
                    CallbackResult::Next => continue,
                    CallbackResult::Stop => break,
                    CallbackResult::StopWithError(error) => {
                        return Err(error);
                    }
                }
            }

            match self.handler.on_clipboard_change() {
                CallbackResult::Next => (),
                CallbackResult::Stop => break,
                CallbackResult::StopWithError(error) => {
                    return Err(error);
                }
            }
        }

        Ok(())
    }
}
