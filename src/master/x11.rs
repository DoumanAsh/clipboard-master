use std::io;
use crate::{ClipboardHandler, CallbackResult, Master};

use x11_clipboard::xcb;

impl<H: ClipboardHandler> Master<H> {
    ///Starts Master by waiting for any change
    pub fn run(&mut self) -> io::Result<()> {
        let mut result = Ok(());
        let clipboard = x11_clipboard::Clipboard::new().unwrap();

        let xfixes = match xcb::query_extension(&clipboard.getter.connection, "XFIXES").get_reply() {
            Ok(xfixes) => xfixes,
            Err(error) => return Err(io::Error::new(io::ErrorKind::Other, error)),
        };
        assert!(xfixes.present());
        xcb::xfixes::query_version(&clipboard.getter.connection, 5, 0);

        loop {
            let selection = clipboard.getter.atoms.utf8_string;

            let screen = match clipboard.getter.connection.get_setup().roots().nth(clipboard.getter.screen as usize) {
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
            xcb::xfixes::select_selection_input(&clipboard.getter.connection, screen.root(), clipboard.getter.atoms.primary, 0);
            xcb::xfixes::select_selection_input(&clipboard.getter.connection, screen.root(), clipboard.getter.atoms.clipboard, 0);
            xcb::xfixes::select_selection_input(&clipboard.getter.connection, screen.root(), selection,
                                                xcb::xfixes::SELECTION_EVENT_MASK_SET_SELECTION_OWNER |
                                                xcb::xfixes::SELECTION_EVENT_MASK_SELECTION_CLIENT_CLOSE |
                                                xcb::xfixes::SELECTION_EVENT_MASK_SELECTION_WINDOW_DESTROY);
            clipboard.getter.connection.flush();
            let first_event = xfixes.first_event();

            loop {
                let event = match clipboard.getter.connection.wait_for_event() {
                    Some(event) => event,
                    None => {
                        continue
                    }
                };

                let rsp_type = event.response_type();

                match rsp_type & !0x80 {
                    xcb::SELECTION_NOTIFY | xcb::PROPERTY_NOTIFY => break,
                    _ => (),
                }
            };

            match self.handler.on_clipboard_change() {
                CallbackResult::Next => (),
                CallbackResult::Stop => break,
                CallbackResult::StopWithError(error) => {
                    result = Err(error);
                    break;
                },
            }

            xcb::delete_property(&clipboard.getter.connection, clipboard.getter.window, clipboard.getter.atoms.property);
        }

        xcb::delete_property(&clipboard.getter.connection, clipboard.getter.window, clipboard.getter.atoms.property);

        result
    }
}
