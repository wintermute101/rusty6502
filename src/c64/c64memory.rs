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

    fn screen_code_to_char(screen_code: u8) -> char{

        match (screen_code & 0x7f) {
            0x00 => '@',
            0x01 => 'A',
            0x02 => 'B',
            0x03 => 'C',
            0x04 => 'D',
            0x05 => 'E',
            0x06 => 'F',
            0x07 => 'G',
            0x08 => 'H',
            0x09 => 'I',
            0x0a => 'J',
            0x0b => 'K',
            0x0c => 'L',
            0x0d => 'M',
            0x0e => 'N',
            0x0f => 'O',
            0x10 => 'P',
            0x11 => 'Q',
            0x12 => 'R',
            0x13 => 'S',
            0x14 => 'T',
            0x15 => 'U',
            0x16 => 'V',
            0x17 => 'W',
            0x18 => 'X',
            0x19 => 'Y',
            0x1a => 'Z',
            0x1b => '[',
            0x1c => '£',
            0x1d => ']',

            0x20 => ' ',
            0x21 => '!',
            0x22 => '"',
            0x23 => '#',
            0x24 => '$',
            0x25 => '%',
            0x26 => '&',
            0x27 => '`',
            0x28 => '(',
            0x29 => ')',
            0x2a => '*',
            0x2b => '+',
            0x2c => ',',
            0x2d => '-',
            0x2e => '.',
            0x2f => '/',
            0x30 => '0',
            0x31 => '1',
            0x32 => '2',
            0x33 => '3',
            0x34 => '4',
            0x35 => '5',
            0x36 => '6',
            0x37 => '7',
            0x38 => '8',
            0x39 => '9',
            0x3a => ':',
            0x3b => ';',
            0x3c => '<',
            0x3d => '=',
            0x3e => '>',
            0x3f => '?',

            _    => '.',
        }
    }

    pub fn show_screen_ram(&self, translate: bool){
        for i in 0..1000/40{
            let addr = 0x0400 + i*40;
            let mslicee: [u8; 40] = self.ram[addr .. addr+40].try_into().unwrap();
            if translate{
                let translated: Vec<_> = mslicee.iter().map(|c| C64Memory::screen_code_to_char(*c)).collect();
                print!("{:04x}: ", addr);
                for i in translated{
                    print!("{}", i);
                }
                println!("");
            }
            else{
                println!("{:04x}: {:02x?}", addr, mslicee);
            }
        }
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
