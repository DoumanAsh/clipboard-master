use std::io;
use std::sync::mpsc::{self, SyncSender, Receiver, sync_channel};
use crate::{ClipboardHandler, CallbackResult};

use objc2::{msg_send_id, rc::Id, ClassType};
use objc2_app_kit::NSPasteboard;

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
        let pasteboard: Option<Id<NSPasteboard>> = unsafe { msg_send_id![NSPasteboard::class(), generalPasteboard] };

        let pasteboard = match pasteboard {
            Some(pasteboard) => pasteboard,
            None => return Err(io::Error::new(io::ErrorKind::Other, "Unable to create mac pasteboard")),
        };

        let mut prev_count = unsafe { pasteboard.changeCount() };
        let mut result = Ok(());

        loop {
            let count: isize = unsafe { pasteboard.changeCount() };

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
