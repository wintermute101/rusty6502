mod c64;
use c64::C64;

fn main() {
    let mut c64 = C64::new();

    c64.enable_trace(64);
    c64.reset();

    let r = c64.run();

    match r{
        Ok(()) => {},
        Err(e) => {
            eprintln!("C64 Cpu error: {}", e);
            c64.show_cpu_debug();
        }
    }
}
