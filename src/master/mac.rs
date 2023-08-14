use std::io;
use std::sync::mpsc::{self, SyncSender, Receiver, sync_channel};
use crate::{ClipboardHandler, CallbackResult};

use objc::runtime::{Object, Class};
use objc_id::Id;

#[link(name = "AppKit", kind = "framework")]
extern "C" {}

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

        let mut prev_count = unsafe { msg_send![pasteboard, changeCount] };
        let mut result = Ok(());

        loop {
            let count: isize = unsafe { msg_send![pasteboard, changeCount] };

            if count == prev_count {
                match self.recv.recv_timeout(self.handler.sleep_interval()) {
                    Ok(()) => break,
                    //timeout
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
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

            match self.recv.try_recv() {
                Ok(()) => break,
                Err(mpsc::TryRecvError::Empty) => continue,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }

        result
    }
}
