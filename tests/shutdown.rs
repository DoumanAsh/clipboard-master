use clipboard_master::{Master, ClipboardHandler, CallbackResult};

pub struct Handler;

impl ClipboardHandler for Handler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        CallbackResult::Next
    }
}

//TODO: Make shutdown work on Linux
//This is currently difficult due to buggy x11-clipboard lib
#[cfg(not(target_arch = "linux"))]
#[test]
fn should_shutdown_successfully() {
    let mut master = Master::new(Handler).expect("To create master");
    let shutdown = master.shutdown_channel();
    std::thread::spawn(move || {
        std::thread::sleep(core::time::Duration::from_secs(5));
        println!("signal");
        shutdown.signal();
    });

    println!("RUN");
    master.run().expect("to finish");
}
