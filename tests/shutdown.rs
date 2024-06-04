use std::time;
use clipboard_master::{Master, ClipboardHandler, CallbackResult};

pub struct Handler;

impl ClipboardHandler for Handler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        CallbackResult::Next
    }
}

#[test]
fn should_shutdown_successfully() {
    const TIMEOUT: time::Duration = time::Duration::from_secs(5);
    let mut master = Master::new(Handler).expect("To create master");
    let shutdown = master.shutdown_channel();
    std::thread::spawn(move || {
        std::thread::sleep(TIMEOUT);
        println!("signal");
        shutdown.signal();
    });

    println!("RUN");
    let now = time::Instant::now();
    master.run().expect("to finish");
    assert!(now.elapsed() >= (TIMEOUT - time::Duration::from_millis(500)));
    assert!(now.elapsed() <= (TIMEOUT + time::Duration::from_millis(500)));
}
