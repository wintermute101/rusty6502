mod cpu6502;
use cpu6502::{Memory,CPU6502};

fn main() {

        let mem = Memory::from_file("./tests/6502_functional_test.bin").unwrap();
        let mut cpu = CPU6502::new(mem);
        cpu.reset_at(0x0400);

        let mut cnt = 0;

        loop{
            cpu.run_single();
            //println!("CPU: {:?}", cpu);

            cnt += 1;

            println!("CNT {}", cnt);
            if cnt > 100{
                break;
            }
        }
}
