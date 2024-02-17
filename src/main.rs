mod cpu6502;
use cpu6502::{Memory,CPU6502};

fn main() {

        let mem = Memory::from_file("./tests/6502_functional_test.bin").unwrap();
        let mut cpu = CPU6502::new(mem);
        cpu.reset_at(0x0400);
        cpu.enable_trace(128);

        let mut cnt = 0;

        loop{
            match cpu.run_single() {
                Ok(_) => {},
                Err(e) => {
                    cpu.show_cpu_debug();
                    println!("Error: {}", e);
                    break;
                }
            };
        }
        cpu.show_trace();
}
