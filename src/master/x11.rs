use crate::{CallbackResult, ClipboardHandler};

use std::io;
use std::sync::OnceLock;
use std::sync::mpsc::{self, SyncSender, Receiver, sync_channel};

use x11rb::protocol::xfixes;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;

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


        if let Err(error) = xfixes::query_version(&clipboard.getter.connection, 5, 0) {
            return Err(io::Error::new(io::ErrorKind::Other, error));
        }

        let mut result = Ok(());
        'main: loop {
            let selection = clipboard.getter.atoms.clipboard;

            let screen = match clipboard.getter.connection.setup().roots.get(clipboard.getter.screen) {
                Some(screen) => screen,
                None => match self.handler.on_clipboard_error(io::Error::new(io::ErrorKind::Other, "Screen is not available")) {
                    CallbackResult::Next => continue,
                    CallbackResult::Stop => break,
                    CallbackResult::StopWithError(error) => {
                        result = Err(error);
                        break;
                    }
                }
            };

            // Clear selection sources...
            let cookie = xfixes::select_selection_input(
                &clipboard.getter.connection,
                screen.root,
                clipboard.getter.atoms.primary,
                xfixes::SelectionEventMask::default()
            ).and_then(|_| xfixes::select_selection_input(
                &clipboard.getter.connection,
                screen.root,
                clipboard.getter.atoms.clipboard,
                xfixes::SelectionEventMask::default()
            // ...and set the one requested now
            )).and_then(|_| xfixes::select_selection_input(
                &clipboard.getter.connection,
                screen.root,
                selection,
                xfixes::SelectionEventMask::SET_SELECTION_OWNER | xfixes::SelectionEventMask::SELECTION_CLIENT_CLOSE | xfixes::SelectionEventMask::SELECTION_WINDOW_DESTROY
            ));

            if let Err(error) = clipboard.getter.connection.flush() {
                match self.handler.on_clipboard_error(io::Error::new(io::ErrorKind::Other, error)) {
                    CallbackResult::Next => continue,
                    CallbackResult::Stop => break,
                    CallbackResult::StopWithError(error) => {
                        result = Err(error);
                        break;
                    }
                }
            }

            let sequence_number = match cookie {
                Ok(cookie) => {
                    let sequence_number = cookie.sequence_number();
                    if let Err(error) = cookie.check() {
                        match self.handler.on_clipboard_error(io::Error::new(io::ErrorKind::Other, error)) {
                            CallbackResult::Next => continue,
                            CallbackResult::Stop => break,
                            CallbackResult::StopWithError(error) => {
                                result = Err(error);
                                break;
                            }
                        }
                    }
                    sequence_number
                },
                Err(error) => match self.handler.on_clipboard_error(io::Error::new(io::ErrorKind::Other, error)) {
                    CallbackResult::Next => continue,
                    CallbackResult::Stop => break,
                    CallbackResult::StopWithError(error) => {
                        result = Err(error);
                        break;
                    }
                }
            };

            'poll: loop {
                match clipboard.getter.connection.poll_for_event_with_sequence() {
                    Ok(Some((_, seq))) if seq >= sequence_number => {
                        match self.handler.on_clipboard_change() {
                            CallbackResult::Next => break 'poll,
                            CallbackResult::Stop => break 'main,
                            CallbackResult::StopWithError(error) => {
                                result =  Err(error);
                                break 'main;
                            }
                        }
                    },
                    Ok(_) => {
                        match self.recv.recv_timeout(self.handler.sleep_interval()) {
                            Ok(()) => break 'main,
                            //timeout
                            Err(mpsc::RecvTimeoutError::Timeout) => continue 'poll,
                            Err(mpsc::RecvTimeoutError::Disconnected) => break 'main,
                        }
                    }
                    Err(error) => {
                        let error = io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to load clipboard: {:?}", error),
                        );

                        match self.handler.on_clipboard_error(error) {
                            CallbackResult::Next => break 'poll,
                            CallbackResult::Stop => break 'main,
                            CallbackResult::StopWithError(error) => {
                                result = Err(error);
                                break 'main;
                            }
                        }
                    }
                }
            }

            let delete = clipboard.getter.connection.delete_property(clipboard.getter.window, clipboard.getter.atoms.property)
                                                    .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
                                                    .and_then(|cookie| cookie.check().map_err(|error| io::Error::new(io::ErrorKind::Other, error)));
            if let Err(error) = delete {
                match self.handler.on_clipboard_error(error) {
                    CallbackResult::Next => (),
                    CallbackResult::Stop => break,
                    CallbackResult::StopWithError(error) => {
                        result = Err(error);
                        break;
                    }
                }
            }

            match self.recv.recv_timeout(self.handler.sleep_interval()) {
                Ok(()) => break,
                //timeout
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        match clipboard.getter.connection.delete_property(clipboard.getter.window, clipboard.getter.atoms.property) {
            Ok(cookie) => match cookie.check() {
                Ok(_) => result,
                Err(error) => Err(io::Error::new(io::ErrorKind::Other, error)),
            },
            Err(error) => Err(io::Error::new(io::ErrorKind::Other, error)),
        }
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
