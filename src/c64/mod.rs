mod cpu6502;
mod c64memory;
use cpu6502::{CPU6502,CpuError,InterruptType};
use c64memory::C64Memory;

pub struct C64{
    cpu: CPU6502,
    memory: C64Memory,
}

impl C64{
    pub fn new() -> Self{
        let mem = C64Memory::new();
        let cpu = CPU6502::new();

        C64 { cpu: cpu, memory: mem }

    }

    pub fn reset(&mut self){
        self.cpu.reset(&mut self.memory);
    }

    pub fn run(&mut self) -> Result<(), CpuError>{
        loop{
            self.cpu.run_single(&mut self.memory)?
        }
    }

    pub fn run_single(&mut self) -> Result<(), CpuError>{
        self.cpu.run_single(&mut self.memory)
    }

    pub fn enable_trace(&mut self, trace_size_limit: usize){
        self.cpu.enable_trace(trace_size_limit)
    }

    pub fn show_debug(&self){
        self.cpu.show_cpu_debug();
        use super::c64::cpu6502::memory::Memory6502Debug;
        println!("**** ZeroPage ****");
        self.memory.show_zero_page();
        println!("****  Stack   ****");
        self.memory.show_stack();
    }

    pub fn interrupt(&mut self){
        self.cpu.interrupt(InterruptType::INT, &mut self.memory);
    }

    pub fn show_screen_ram(&self, translated: bool){
        println!("****  Screen  ****");
        self.memory.show_screen_ram(translated);
    }
}