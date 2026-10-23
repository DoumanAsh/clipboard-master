mod x11;
mod wayland;

use std::{io, env};

use crate::ClipboardHandler;

///Shutdown channel
///
///On drop requests shutdown to gracefully close clipboard listener as soon as possible.
pub struct Shutdown {
    x11: Option<x11::Shutdown>,
    wayland: Option<wayland::Shutdown>,
}

#[allow(clippy::large_enum_variant)]
enum Backend<H> {
    X11(x11::Master<H>),
    CliWayland(wayland::Master<H>),
}

///Linux clipboard master
///
///Uses `WAYLAND_DISPALY` and/or `XDG_SESSION_TYPE` to determine session type
pub struct Master<H> {
    backend: Backend<H>
}

impl<H: ClipboardHandler> Master<H> {
    #[inline(always)]
    ///Creates new instance.
    pub fn new(handler: H) -> io::Result<Self> {
        let backend = if env::var("WAYLAND_DISPALY").is_ok() {
            Backend::CliWayland(wayland::Master::new(handler)?)
        } else {
            match env::var("XDG_SESSION_TYPE").ok().as_deref() {
                Some("wayland") => Backend::CliWayland(wayland::Master::new(handler)?),
                Some("x11") => Backend::X11(x11::Master::new(handler)?),
                Some(unknown) => return Err(io::Error::other(format!("Unknown XDG_SESSION_TYPE={unknown}"))),
                None => return Err(io::Error::other("Unable determine session type")),
            }
        };

        Ok(Self {
            backend
        })
    }

    #[inline(always)]
    ///Creates shutdown channel.
    pub fn shutdown_channel(&self) -> Shutdown {
        let mut shutdown = Shutdown {
            x11: None,
            wayland: None,
        };

        match &self.backend {
            Backend::CliWayland(master) => {
                shutdown.wayland = Some(master.shutdown_channel());
            },
            Backend::X11(master) => {
                shutdown.x11 = Some(master.shutdown_channel());
            },
        }

        shutdown
    }

    #[inline(always)]
    ///Starts Master by waiting for any change
    pub fn run(&mut self) -> io::Result<()> {
        match &mut self.backend {
            Backend::CliWayland(master) => master.run(),
            Backend::X11(master) => master.run()
        }
    }
}
