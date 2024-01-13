use crate::{CallbackResult, ClipboardHandler};
use std::io;

///Shutdown channel
///
///On drop requests shutdown to gracefully close clipboard listener as soon as possible.
pub struct Shutdown {
}

impl Drop for Shutdown {
    #[inline(always)]
    fn drop(&mut self) {
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
}

impl<H: ClipboardHandler> Master<H> {
    #[inline(always)]
    ///Creates new instance.
    pub fn new(handler: H) -> io::Result<Self> {
        Ok(Self {
            handler,
        })
    }

    #[inline(always)]
    ///Creates shutdown channel.
    pub fn shutdown_channel(&self) -> Shutdown {
        Shutdown {
        }
    }


    ///Starts Master by waiting for any change
    pub fn run(&mut self) -> io::Result<()> {
        let clipboard = x11_clipboard::Clipboard::new();
        let clipboard = match clipboard {
            Ok(clipboard) => clipboard,
            Err(error) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to initialize clipboard: {:?}", error),
                ))
            }
        };

        loop {
            let res = clipboard.load_wait(
                clipboard.getter.atoms.clipboard,
                clipboard.getter.atoms.incr,
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
