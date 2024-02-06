use crate::{CallbackResult, ClipboardHandler};

use std::io;
use std::sync::OnceLock;
use std::sync::mpsc::{self, SyncSender, Receiver, sync_channel};

///Shutdown channel
///
///On drop requests shutdown to gracefully close clipboard listener as soon as possible.
pub struct Shutdown {
    sender: SyncSender<()>,
}

impl Drop for Shutdown {
    #[inline(always)]
    fn drop(&mut self) {
        let _ = self.sender.send(());
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
    sender: SyncSender<()>,
    recv: Receiver<()>
}

impl<H: ClipboardHandler> Master<H> {
    #[inline(always)]
    ///Creates new instance.
    pub fn new(handler: H) -> io::Result<Self> {
        let (sender, recv) = sync_channel(0);

        Ok(Self {
            handler,
            sender,
            recv,
        })
    }

    #[inline(always)]
    ///Creates shutdown channel.
    pub fn shutdown_channel(&self) -> Shutdown {
        Shutdown {
            sender: self.sender.clone()
        }
    }


    ///Starts Master by waiting for any change
    pub fn run(&mut self) -> io::Result<()> {
        let clipboard = match Self::x11_clipboard() {
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
            match res {
                Ok(_) => {
                    match self.handler.on_clipboard_change() {
                        CallbackResult::Next => (),
                        CallbackResult::Stop => break,
                        CallbackResult::StopWithError(error) => {
                            return Err(error);
                        }
                    }
                },
                Err(error) => {
                    let error = io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to load clipboard: {:?}", error),
                    );

                    match self.handler.on_clipboard_error(error) {
                        CallbackResult::Next => (),
                        CallbackResult::Stop => break,
                        CallbackResult::StopWithError(error) => {
                            return Err(error);
                        }
                    }
                }
            }

            match self.recv.try_recv() {
                Ok(()) => break,
                Err(mpsc::TryRecvError::Empty) => continue,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }

        Ok(())
    }

    ///Gets one time initialized x11 clipboard.
    ///
    ///This is only available on linux
    ///
    ///Prefer to use it on Linux as underlying `x11-clipboard` crate has buggy dtor
    ///and doesn't clean up all resources associated with `Clipboard`
    pub fn x11_clipboard() -> &'static Result<x11_clipboard::Clipboard, x11_clipboard::error::Error> {
        static CLIP: OnceLock<Result<x11_clipboard::Clipboard, x11_clipboard::error::Error>> = OnceLock::new();
        CLIP.get_or_init(x11_clipboard::Clipboard::new)
    }
}
