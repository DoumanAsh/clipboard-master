use crate::{CallbackResult, ClipboardHandler};

use std::io;

use wl_clipboard_rs::watch;

///Shutdown channel
///
///On drop requests shutdown to gracefully close clipboard listener as soon as possible.
pub struct Shutdown {
    sender: watch::CancelHandle
}

impl Drop for Shutdown {
    #[inline(always)]
    fn drop(&mut self) {
        self.sender.cancel();
    }
}

///Clipboard master using wayland **without window**
///
///Tracks changes of clipboard and invokes corresponding callbacks.
pub struct Master<H> {
    handler: H,
    watcher: watch::Watcher,
}

impl<H: ClipboardHandler> Master<H> {
    #[inline(always)]
    ///Creates new instance.
    pub fn new(handler: H) -> io::Result<Self> {
        let watcher = watch::Watcher::new(watch::ClipboardType::Both, wl_clipboard_rs::paste::Seat::Unspecified).map_err(|error| io::Error::other(error))?;
        Ok(Self {
            handler,
            watcher,
        })
    }

    #[inline(always)]
    ///Creates shutdown channel.
    pub fn shutdown_channel(&self) -> Shutdown {
        Shutdown {
            sender: self.watcher.cancel_handle()
        }
    }

    ///Starts Master by waiting for any change
    pub fn run(&mut self) -> io::Result<()> {
        loop {
            match self.watcher.next_event() {
                Ok(Some(event)) => match event {
                    watch::ClipboardEvent::Changed { .. } => match self.handler.on_clipboard_change() {
                        CallbackResult::Next => continue,
                        CallbackResult::Stop => break Ok(()),
                        CallbackResult::StopWithError(error) => break Err(error),
                    },
                    watch::ClipboardEvent::Cleared { .. } => continue,
                },
                Ok(None) => break Ok(()),
                Err(error) => match self.handler.on_clipboard_error(io::Error::other(error)) {
                    CallbackResult::Next => continue,
                    CallbackResult::Stop => break Ok(()),
                    CallbackResult::StopWithError(error) => break Err(error),
                }
            }
        }
    }
}
