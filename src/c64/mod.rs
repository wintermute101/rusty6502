mod cpu6502;
mod c64memory;
use cpu6502::{CPU6502,CpuError};
use c64memory::C64Memory;

pub struct C64{
    cpu: CPU6502<C64Memory>,
}

impl C64{
    pub fn new() -> Self{
        let mem = C64Memory::new();
        let cpu = CPU6502::new(mem);

        C64 { cpu: cpu }
        
    }

    pub fn reset(&mut self){
        self.cpu.reset();
    }

    pub fn run(&mut self) -> Result<(), CpuError>{
        loop{
            self.cpu.run_single()?
        }
    }

    pub fn enable_trace(&mut self, trace_size_limit: usize){
        self.cpu.enable_trace(trace_size_limit)
    }

    pub fn show_cpu_debug(&self){
        self.cpu.show_cpu_debug();
    }
}