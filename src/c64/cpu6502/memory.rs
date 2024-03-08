use std::fs::File;
use std::io::prelude::*;

pub trait Memory6502 {
    fn write_memory(&mut self, address: u16, value: u8);
    fn read_memory(&mut self, address: u16) -> u8;
    fn read_memory_word(&mut self, address: u16) -> u16;
}

pub trait Memory6502Debug {
    fn show_stack(&self);
    fn show_zero_page(&self);
}

pub struct Memory{
    memory: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self{
        Memory{ memory: vec![0; size]}
    }

    pub fn from_file(file: &str) -> std::io::Result<Self>{
        let mut file = File::open(file)?;
        let file_size = file.metadata()?.len();
        let mut data = Vec::with_capacity(file_size as usize);
        file.read_to_end(&mut data)?;
        Ok(Memory{memory: data})
    }
}

impl Memory6502Debug for Memory {
    fn show_stack(&self){
        let mslicee: [u8; 16] = self.memory[0x01f0 .. 0x0200].try_into().unwrap();
        println!("{:04x}: {:02x?}", 0x01f0, mslicee);
    }

    fn show_zero_page(&self){
        let mut last = [0xff; 16];
        let mut lasti = 0;

        for i in 0..256/16{
            let mslicee: [u8; 16] = self.memory[i*16 .. (i+1)*16].try_into().unwrap();

            if mslicee != last{
                if lasti+1 != i && i != 0{
                    println!("*");
                }
                println!("{:04x}: {:02x?}", i*16, mslicee);
                lasti = i;
            }
            else if i == 256/16-1 {
                println!("*\n{:04x}", i*16);
            }
            last = mslicee;
        }
    }
}

impl Memory6502 for Memory{
    fn write_memory(&mut self, address: u16, value: u8){
        if let Some(mem) = self.memory.get_mut(address as usize){
            *mem = value;
        }
        else{
            println!("Write to address out of range ADDR={:#06x} VAL={:#04x}", address, value);
        }
    }

    fn read_memory(&mut self, address: u16) -> u8{
        if let Some(mem) = self.memory.get(address as usize){
            *mem
        }
        else
        {
            println!("Read from address out of range ADDR={:#06x}", address);
            0
        }
    }

    fn read_memory_word(&mut self, address: u16) -> u16{
        if let Ok(m) = self.memory[address as usize .. (address as usize) + 2].try_into(){
            u16::from_le_bytes(m)
        }
        else{
            println!("Read from address out of range ADDR={:#06x}", address);
            0
        }
    }

}

impl std::fmt::Debug for Memory {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>{
        let l = self.memory.len() / 16;

        let mut last = [0xff; 16];
        let mut lasti = 0;

        for i in 0..l{
            let mslicee: [u8; 16] = self.memory[i*16 .. (i+1)*16].try_into().unwrap();

            if mslicee != last{
                if lasti+1 != i && i != 0{
                    fmt.write_str(&format!("*\n"))?;
                }
                fmt.write_str(&format!("{:04x}: {:02x?}\n", i*16, mslicee))?;
                lasti = i;
            }
            else if i == l-1 {
                fmt.write_str(&format!("*\n{:04x}\n", i*16))?;
            }

            last = mslicee;
        }

        fmt.write_str(&format!(""))
    }
}
