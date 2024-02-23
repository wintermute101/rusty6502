
mod c64;
//use cpu6502::{Memory,CPU6502};
use std::time::Instant;

fn main() {

        /*let mem = Memory::from_file("./tests/6502_functional_test.bin").unwrap();
        let mut cpu = CPU6502::new(mem);
        cpu.reset_at(0x0400);

        let mut cnt = 0;
        let now = Instant::now();

        loop{
            cnt += 1;
            match cpu.run_single() {

                Ok(_) => {},
                Err(e) => {
                    if e.pc == 0x3469{
                        println!("Run {} instructions", cnt); //This test program loops here on success
println!("
                        ; S U C C E S S ************************************************
                        ; -------------
                                success         ;if you get here everything went well
3469 : 4c6934          >        jmp *           ;test passed, no errors");
                        break;
                    }
                    cpu.show_cpu_debug();
                    println!("Error: {}", e);
                    break;
                }
            };
        }
        let elapsed_time = now.elapsed();
        println!("Run {} instructions in {:?} per sec {}", cnt, elapsed_time, cnt * 1000 / elapsed_time.as_millis());*/


}
