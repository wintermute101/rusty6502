use super::cpu6502::memory::{Memory6502,Memory6502Debug};
use std::fs::File;
use std::io::prelude::*;

pub struct C64Memory{
    ram: [u8; 64*1024],
    kernal: Vec<u8>,
    basic_rom: Vec<u8>,
    color_ram: [u8; 1024]
}

impl C64Memory{

    fn load_rom(path: &str) -> std::io::Result<Vec<u8>>{
        let mut file = File::open(path)?;
        let file_size = file.metadata()?.len();
        let mut data = Vec::with_capacity(file_size as usize);
        file.read_to_end(&mut data)?;
        Ok(data)
    }

    pub fn new() -> Self{
        let kernal = C64Memory::load_rom("roms/kernal.901227-02.bin").expect("no kernal");
        let basic = C64Memory::load_rom("roms/basic.901226-01.bin").expect("no basic");

        C64Memory { ram: [0; 64*1024], kernal: kernal, basic_rom: basic, color_ram: [0; 1024] }
    }
}

impl Memory6502 for C64Memory{
    fn write_memory(&mut self, address: u16, value: u8) {
        match address {
            0xd000 ..= 0xdfff => { //IO
                if address >= 0xd800 && address <= 0xdbff{
                    println!("IO Color RAM Write {:#06x} => {:#04x}", address, value);
                    let adr = address - 0xd800;
                    self.color_ram[adr as usize] = value;
                }
                else{
                    println!("IO Write {:#06x} => {:#04x}", address, value);
                }
            }
            _ => {
                self.ram[address as usize] = value;
            }
        }
    }

    fn read_memory(&self, address: u16) -> u8 {
        match address{
            0xe000 ..= 0xffff => { //Kernal
                let adr = address - 0xe000;
                self.kernal[adr as usize]
            }
            0xd000 ..= 0xdfff => { //IO
                println!("IO Read {:#06x}", address);
                0x00
            }
            0xa000 ..= 0xbfff => { //Basic
                let adr = address - 0xa000;
                self.basic_rom[adr as usize]
            }

            _ => {
                self.ram[address as usize]
            }
        }
    }

    fn read_memory_word(&self, address: u16) -> u16 {
        let lo = self.read_memory(address);
        let hi = self.read_memory(address.overflowing_add(1).0);

        (hi as u16) << 8 | lo as u16
    }
}

impl Memory6502Debug for C64Memory{
    fn show_stack(&self){
        let mslicee: [u8; 16] = self.ram[0x01f0 .. 0x0200].try_into().unwrap();
        println!("{:04x}: {:02x?}", 0x01f0, mslicee);
    }

    fn show_zero_page(&self){
        let mut last = [0xff; 16];
        let mut lasti = 0;

        for i in 0..256/16{
            let mslicee: [u8; 16] = self.ram[i*16 .. (i+1)*16].try_into().unwrap();

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
