use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

mod c64;
use c64::C64;

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut c64 = C64::new();

    c64.enable_trace(64);
    c64.reset();

    let mut now = Instant::now();

    while running.load(Ordering::SeqCst){
        let r = c64.run_single();

        match r{
            Ok(()) => {},
            Err(e) => {
                eprintln!("C64 Cpu error: {}", e);
                break;
            }
        }

        match now.elapsed(){
            //v if v >= Duration::from_micros(16666) => {
            v if v >= Duration::from_secs(1) => {
                println!("Intterrupt!");
                 c64.interrupt();
                 now = Instant::now();
            }
            _ => {}
         }
    }

    println!("Exiting...");
    c64.show_cpu_debug();
}
