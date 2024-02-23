mod cpu6502;
use cpu6502::CPU6502;
use cpu6502::memory::{Memory6502,Memory6502Debug};

struct C64Memory{

}

impl Memory6502 for C64Memory{
    fn write_memory(&mut self, address: u16, value: u8) {
        
    }

    fn read_memory(&self, address: u16) -> u8 {
        0xaa
    }

    fn read_memory_word(&self, address: u16) -> u16 {
        0xaa55
    }
}

impl Memory6502Debug for C64Memory{
    fn show_stack(&self) {
        
    }

    fn show_zero_page(&self) {
        
    }
}

struct C64{
    cpu: CPU6502<C64Memory>,
}
