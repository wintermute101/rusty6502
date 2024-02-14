mod memory;
pub use memory::Memory;

struct StatusRegister{
    value: u8,
}

#[allow(non_snake_case)]
impl StatusRegister {
    fn set_NZ(&mut self, val: u8){
        self.value = (self.value & 0b0111_1111) | (val & 0b1000_0000);
        self.value = (self.value & 0b1111_1101) | ((val == 0) as u8) << 1;
    }

    fn set_C(&mut self, val: bool){
        self.value = (self.value & 0b1111_1110) | (val as u8);
    }

    fn get_Z(&self) -> bool{
        self.value & 0b0000_0010 != 0
    }

    fn get_N(&self) -> bool{
        self.value & 0b1000_0000 != 0
    }

    fn get_C(&self) -> bool{
        self.value & 0b0000_0001 != 0
    }

    fn get_D(&self) -> bool{
        self.value & 0b0000_1000 != 0
    }

    fn set_D(&mut self, val: bool){
        self.value = (self.value & 0b1111_0111) | ((val as u8) << 3);
    }

    fn get_V(&mut self) -> bool{
        self.value & 0b0100_0000 != 0
    }

    fn set_V(&mut self, val: bool){
        self.value = (self.value & 0b1011_1111) | ((val as u8) << 6);
    }

    fn get_I(&mut self) -> bool{
        self.value & 0b0000_0100 != 0
    }

    fn set_I(&mut self, val: bool){
        self.value = (self.value & 0b1111_1011) | ((val as u8) << 2);
    }
}

impl std::fmt::Debug for StatusRegister{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>{
        fmt.write_str(&format!("NV-BDIZC {:08b}", self.value))
    }
}

#[derive(Debug)]
enum AdressingType{
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
}

#[derive(PartialEq)]
enum InterruptType {
    INT,
    NMI,
    BRK,
}

#[allow(non_snake_case)]
pub struct CPU6502{
    A:  u8,
    X:  u8,
    Y:  u8,
    PC: u16,
    SP:  u8,
    P:  StatusRegister,

    prev_PC: u16,

    memory: Memory,

}

impl std::fmt::Debug for CPU6502 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>{
        fmt.write_str(&format!("PC={:#06x} A={:#04x} X={:#04x} Y={:#04x} SP={:#04x} P={:?}\n"
                                        , self.PC, self.A, self.X, self.Y, self.SP, self.P))
        //fmt.write_str(&format!("{:?}", self.memory))?;
        //fmt.write_str(&format!(""))
    }
}

impl CPU6502 {
    pub fn new(mem: Memory) -> Self{
        CPU6502 { A: 0, X: 0, Y: 0, PC: 0, SP: 0xff, P: StatusRegister { value: 0 }, prev_PC: 0, memory: mem }
    }

    pub fn reset(&mut self) {
        let resetvec_addr = self.memory.read_memory_word(0xfffc);
        self.PC = resetvec_addr;
    }

    pub fn reset_at(&mut self, start_address: u16) {
        self.PC = start_address;
    }

    fn get_address(&mut self, adrtype: AdressingType) -> u16{
        match adrtype {
            AdressingType::ZeroPage => {
                let addr = self.memory.read_memory(self.PC);
                self.PC += 1;
                let ret = addr as u16;
                print!("Address=0x{:04x} ", ret);
                ret
            }
            AdressingType::ZeroPageX => {
                let addr = self.memory.read_memory(self.PC).overflowing_add(self.X).0;
                self.PC += 1;
                let ret = addr as u16;
                print!("Address=0x{:04x} ", ret);
                ret
            }
            AdressingType::ZeroPageY => {
                let addr = self.memory.read_memory(self.PC).overflowing_add(self.Y).0;
                self.PC += 1;
                let ret = addr as u16;
                print!("Address=0x{:04x} ", ret);
                ret
            }
            AdressingType::Absolute => {
                let ret = self.memory.read_memory_word(self.PC);
                self.PC += 2;
                print!("Address={:#06x} ", ret as u16);
                ret
            }
            AdressingType::AbsoluteX => {
                let ret = self.memory.read_memory_word(self.PC).overflowing_add(self.X as u16).0;
                self.PC += 2;
                print!("Address={:#06x} ", ret as u16);
                ret
            }
            AdressingType::AbsoluteY => {
                let ret = self.memory.read_memory_word(self.PC).overflowing_add(self.Y as u16).0;
                self.PC += 2;
                print!("Address={:#06x} ", ret as u16);
                ret
            }
            AdressingType::Indirect => {
                let addr1 = self.memory.read_memory_word(self.PC);
                self.PC += 2;
                print!("Address1={:#06x}", addr1 as u16);
                let ret = self.memory.read_memory_word(addr1);
                print!(" Address={:#06x} ", ret as u16);
                ret
            }
            AdressingType::IndirectX => {
                let addr1 = self.memory.read_memory(self.PC).overflowing_add(self.X.into()).0;
                self.PC += 1;
                print!("Address1={:#06x}", addr1 as u16);
                let ret = self.memory.read_memory_word(addr1 as u16);
                print!(" Address={:#06x} ", ret as u16);
                ret
            }
            AdressingType::IndirectY => {
                let addr1 = self.memory.read_memory(self.PC);
                self.PC += 1;
                print!("Address1={:#06x}", addr1 as u16);
                let ret = self.memory.read_memory_word(addr1 as u16).overflowing_add(self.Y as u16);
                print!(" Address={:#06x}", ret.0);
                ret.0
            }
            _ => {
                todo!("Addressing Type {:?} ", adrtype);
            }
        }
    }

    pub fn run_single(&mut self){
        let ins = self.memory.read_memory(self.PC);
        print!("Running INS={:#04x} PC={:#06x} ", ins, self.PC);
        self.PC += 1;

        match ins {
            0x00 => { //BRK
                self.PC += 1;
                self.interrupt(InterruptType::BRK);
                println!("BRK");
            }

            0x01 => { //ORA IndirectX
                let address = self.get_address(AdressingType::IndirectX);
                let data = self.memory.read_memory(address);
                self.A |= data;
                self.P.set_NZ(self.A);
                println!("ORA IndirectX");
            }

            0x05 => { //ORA ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                self.A |= data;
                self.P.set_NZ(self.A);
                println!("ORA ZeroPage");
            }

            0x06 => { //ASL ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let mut data = self.memory.read_memory(address);
                self.P.set_C(data & 0b1000_0000 != 0);
                data = data << 1;
                self.P.set_NZ(data);
                self.memory.write_memory(address, data);
                println!("ASL ZeroPage");
            }

            0x08 => { //PHP
                let address = 0x0100 | self.SP as u16;
                self.memory.write_memory(address, self.P.value | 0b0011_0000); //push brk and ignored as 1
                let r = self.SP.overflowing_sub(1);
                self.SP = r.0;
                if r.1{
                    println!("Stack Overflow!");
                }
                println!("PHP Pushed P={:#04x} ADDR={:#06x}", self.P.value, address);
            }

            0x09 => { //ORA Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                self.A |= data;
                self.P.set_NZ(self.A);
                println!("ORA Immediate");
            }

            0x10 => { //BPL
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;
                
                if !self.P.get_N(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    print!("Branch to PC={:#06x} ", r.0);
                    self.PC = r.0 as u16;
                }
                else {
                    print!("NOT Branching ");
                }
                println!("BPL Relative [{}]", data);
            }

            0x18 => { //CLC
                self.P.set_C(false);
                println!("CLC");
            }

            0x20 => { //JSR
                let address = self.get_address(AdressingType::Absolute);
                let pc = self.PC.overflowing_sub(1).0; //need to push PC+2 not 3 RTS will add 1
                let sp = 0x0100 | self.SP as u16;
                self.SP = self.SP.overflowing_sub(1).0;
                self.memory.write_memory(sp, (pc >> 8) as u8);
                let sp = 0x0100 | self.SP as u16;
                self.SP = self.SP.overflowing_sub(1).0;
                self.memory.write_memory(sp, (pc & 0x0ff) as u8);
                println!("JSR {:#06x} PC={:#06x}", address, self.PC);
                self.PC = address;
            }

            0x28 => { //PLP
                let r = self.SP.overflowing_add(1);
                self.SP = r.0;
                if r.1{
                    println!("Stack Overflow!");
                }
                let address = 0x0100 | self.SP as u16;
                let data = self.memory.read_memory(address);
                self.P.value = data & 0b1100_1111; //ignore B and bit 5
                println!("PLP Pop P={:#04x} ADDR={:#06x}", self.P.value, address);
            }

            0x30 => {
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;

                if self.P.get_N(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    print!("Branch to PC={:#06x} ", r.0);
                    self.PC = r.0 as u16;
                }
                else {
                    print!("NOT Branching ");
                }
                println!("BMI Relative [{}]", data);
            }

            0x48 => { //PHA
                let address = 0x0100 | self.SP as u16;
                let data = self.memory.read_memory(address);
                self.memory.write_memory(address, self.A);
                let r = self.SP.overflowing_sub(1);
                self.SP = r.0;
                if r.1{
                    println!("Stack Overflow!");
                }
                println!("PHA Pushed A={:#04x} ADDR={:#06x}", data, address);
            }

            0x49 => { //EOR Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                self.A = self.A ^ data;
                self.P.set_NZ(self.A);
                println!("EOR Immediate ({:#04x} => {:#04x})", data, self.A);
            }

            0x4c => { //JMP Absolute
                let address = self.get_address(AdressingType::Absolute);
                self.PC = address;
                println!("JMP Absolute");
            }

            0x50 => { //BVC
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;
                if !self.P.get_V(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    print!("Branch to PC={:#06x} ", r.0);
                    self.PC = r.0 as u16;
                }
                else {
                    print!("NOT Branching ");
                }
                println!("BVC Relative [{}]", data);
            }

            0x60 => { //RTS
                self.SP = self.SP.overflowing_add(1).0;
                let sp = 0x0100 | self.SP as u16;
                let addr = self.memory.read_memory_word(sp).overflowing_add(1).0;
                self.SP = self.SP.overflowing_add(1).0;
                self.PC = addr;
                println!("RTS {:#06x}", addr);
            }

            0x68 => {
                let r = self.SP.overflowing_add(1);
                self.SP = r.0;
                if r.1{
                    println!("Stack Overflow!");
                }
                let address = 0x0100 | self.SP as u16;
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);
                println!("PLA Pop A={:#04x} ADDR={:#06x}", data, address);
            }

            0x69 => { //ADC Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                let r = self.A.overflowing_add(data + self.P.get_C() as u8);
                self.A = r.0;
                self.P.set_NZ(r.0);
                self.P.set_C(r.1);
                println!("ADC Immediate ({:#04x} => {:#04x})", data, self.A);
            }

            0x6c => { //JMP Indirect
                let address = self.get_address(AdressingType::Indirect);
                self.PC = address;
                println!("JMP Indirect PC={:#06x} ", address);
            }

            0x70 => { //BVS
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;
                
                if self.P.get_V(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    print!("Branch to PC={:#06x} ", r.0);
                    self.PC = r.0 as u16;
                }
                else {
                    print!("NOT Branching ");
                }
                println!("BVS Relative [{}]", data);
            }

            0x85 => { //STA ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                self.memory.write_memory(address, self.A);
                println!("STA ZeroPage ({:#04x})", self.A);
            }

            0x86 => { //STX ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                self.memory.write_memory(address, self.X);
                println!("STX ZeroPage ({:#04x})", self.X);
            }

            0x88 => { //DEY
                self.Y = self.Y.overflowing_sub(1).0;
                self.P.set_NZ(self.Y);
                println!("DEY");
            }

            0x8a => { //TXA
                self.A = self.X;
                self.P.set_NZ(self.A);
                println!("TXA");
            }

            0x8c => { //STY Absolute
                let address = self.get_address(AdressingType::Absolute);
                self.memory.write_memory(address, self.Y);
                println!("STY Absolute ({:#04x})", self.A);
            }

            0x8d => { //STA Absolute
                let address = self.get_address(AdressingType::Absolute);
                self.memory.write_memory(address, self.A);
                println!("STA Absolute ({:#04x})", self.A);
            }

            0x8e => { //STX Absolute
                let address = self.get_address(AdressingType::Absolute);
                self.memory.write_memory(address, self.X);
                println!("STX Absolute ({:#04x})", self.X);
            }

            0x90 => { //BCC
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;

                if !self.P.get_C(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    print!("Branch to PC={:#06x} ", r.0);
                    self.PC = r.0 as u16;
                }
                else {
                    print!("NOT Branching ");
                }
                println!("BCC Relative [{}]", data);
            }

            0x98 => { //TYA
                self.A = self.Y;
                self.P.set_NZ(self.A);
                println!("TYA");
            }

            0x99 => { //STA AbsoluteY
                let address = self.get_address(AdressingType::AbsoluteY);
                self.memory.write_memory(address, self.A);
                println!("STX AbsoluteY ({:#04x})", self.Y);
            }

            0x9a => {//TXS
                self.SP = self.X;
                println!("TXS");
            }

            0xa0 => { //LDY Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                self.Y = data;
                self.P.set_NZ(data);
                println!("LDY Immediate ({:#04x})", data);
            }

            0xa1 => { //LDA IndirectX
                let address = self.get_address(AdressingType::IndirectX);
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);
                println!("LDA IndirectX ({:#04x})", data);
            }

            0xa2 => { //LDX Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                self.X = data;
                self.P.set_NZ(data);
                println!("LDX Immediate ({:#04x})", data);
            }

            0xa5 => { //LDA ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);
                println!("LDA ZeroPage ({:#04x})", data);
            }

            0xa6 => { //LDX ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                self.X = data;
                self.P.set_NZ(data);
                println!("LDX ZeroPage ({:#04x})", data);
            }

            0xa8 => { //TAY
                self.Y = self.A;
                self.P.set_NZ(self.Y);
                println!("TAY");
            }

            0xa9 => { //LDA Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                self.A = data;
                self.P.set_NZ(data);
                println!("LDA Immediate ({:#04x})", data);
            }

            0xaa => { //TAX
                self.X = self.A;
                self.P.set_NZ(self.X);
                println!("TAX");
            }

            0xad => { //LDA Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);
                println!("LDA Absolute ({:#04x})", data);
            }

            0xb0 => {
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;

                if self.P.get_C(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    print!("Branch to PC={:#06x} ", r.0);
                    self.PC = r.0 as u16;
                }
                else {
                    print!("NOT Branching ");
                }
                println!("BCS Relative [{}]", data);
            }

            0xb1 => { //LDA IndirectY
                let address = self.get_address(AdressingType::IndirectY);
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);
                println!("LDA IndirectY ({:#04x})", data);
            }

            0xba => { //TSX
                self.X = self.SP;
                self.P.set_NZ(self.X);
                println!("TSX({:#04x})", self.X);
            }

            0xbd => { //LDA AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);
                println!("LDA AbsoluteX ({:#04x})", data);
            }

            0xc0 => { //CPY Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                let r = self.Y.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);
                println!("CPY Immediate");
            }

            0xc8 => { //INY
                self.Y = self.Y.overflowing_add(1).0;
                self.P.set_NZ(self.Y);
                println!("INY");
            }

            0xc9 => { //CMP Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                let r = self.A.overflowing_sub(data);
                
                print!("CMP v={:?} ", r);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);
                println!("CMP Immediate ({:#04x})", data);
            }

            0xca => { //DEX
                self.X = self.X.overflowing_sub(1).0;
                self.P.set_NZ(self.X);
                println!("DEX");
            }

            0xcd => { //CMP Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                let r = self.A.overflowing_sub(data);
                print!("CMP v={:?} ", r);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);
                println!("CMP Absolute ({:#04x})", data);
            }

            0xd0 => { //BNE Relative
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;
                
                if !self.P.get_Z(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    print!("Branch to PC={:#06x} ", r.0);
                    self.PC = r.0 as u16;
                }
                else {
                    print!("NOT Branching ");
                }
                println!("BNE Relative [{}]", data);
            }

            0xd8 => { //CLD Clear Decimal Mode
                self.P.set_D(false);
                println!("CLD");
            }

            0xe0 => { //CPX Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                let r = self.X.overflowing_sub(data);
                print!("CPX v={:?} ", r);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);
                println!("CPX Immediate ({:#06x})", data);
            }

            0xe8 => { //INX
                let r = self.X.overflowing_add(1);
                self.X = r.0;
                self.P.set_NZ(r.0);
                println!("INX");
            }

            0xea => { //NOP
                println!("NOP");
            }

            0xf0 => { //BEQ Relative
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;

                if self.P.get_Z(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    print!("Branch to PC={:#06x} ", r.0);
                    self.PC = r.0 as u16;
                }
                else {
                    print!("NOT Branching ");
                }
                println!("BEQ Relative [{}]", data);
            }

            0xfe => { //INC AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let mut data = self.memory.read_memory(address);
                data = data.overflowing_add(1).0;
                self.memory.write_memory(address, data);
                self.P.set_NZ(data);
                println!("INC AbsoluteX ({:#04x})", data);
            }

            _  => {
                todo!("INS={:#04x}", ins);
            }
        }

        if self.PC == self.prev_PC{
            println!("{:?}", self);
            panic!("LOOP Detected PC={:#06x}", self.PC);
        }
        self.prev_PC = self.PC;
    }

    fn interrupt(&mut self, int: InterruptType){
        if self.P.get_I() && int == InterruptType::INT{
            println!("INT while innterrupts are disabled");
            return;
        }
        let sp = 0x0100 | self.SP as u16;
        self.SP = self.SP.overflowing_sub(1).0;
        self.memory.write_memory(sp, (self.PC >> 8) as u8);
        let sp = 0x0100 | self.SP as u16;
        self.SP = self.SP.overflowing_sub(1).0;
        self.memory.write_memory(sp, (self.PC & 0x0ff) as u8);
        let sp = 0x0100 | self.SP as u16;
        self.SP = self.SP.overflowing_sub(1).0;
        if int == InterruptType::BRK{
            //print!(" Write P={:#04x} on stack ({:#06x})", self.P.value | 0b0011_0000, sp);
            self.memory.write_memory(sp, self.P.value | 0b0011_0000); //Set Interrupt flag
        }
        else{
            self.memory.write_memory(sp, self.P.value | 0b0010_0000); //Set Interrupt flag
        }
        let address = match int {
            InterruptType::NMI | InterruptType::BRK => {
                self.memory.read_memory_word(0xfffe) // NMI int vec
            }
            InterruptType::INT => {
                self.memory.read_memory_word(0xfffa)
            }
        };
        println!("INT PC={:#06x} SP={:#04x} INTVEC={:#06x}", self.PC, self.P.value, address);
        self.PC = address;
        self.P.set_I(true); //Disable Interupts
    }
}

#[cfg(test)]
mod tests{
    use crate::cpu6502::{Memory,CPU6502};
    #[test]
    fn test1(){
        let mut mem = Memory::new(4*1024);

        mem.write_memory(0x600,0xa9); //LDA
        mem.write_memory(0x601,0x10); //#$10
        mem.write_memory(0x602,0x85); //STA
        mem.write_memory(0x603,0x00); //$00
        mem.write_memory(0x604,0x85); //STA
        mem.write_memory(0x605,0x01); //$01
        mem.write_memory(0x606,0x06); //ASL
        mem.write_memory(0x607,0x00); //$00
        mem.write_memory(0x608,0xa9); //LDA
        mem.write_memory(0x609,0x30); //#$30
        mem.write_memory(0x60a,0x85); //STA
        mem.write_memory(0x60b,0x02); //$02
        mem.write_memory(0x60c,0xa9); //LDA
        mem.write_memory(0x60d,0x02); //#$02
        mem.write_memory(0x60e,0x85); //STA
        mem.write_memory(0x60f,0x04); //$04
        mem.write_memory(0x610,0xa9); //LDA
        mem.write_memory(0x611,0x01); //#$01
        mem.write_memory(0x612,0xaa); //TAX
        mem.write_memory(0x613,0x01); //ORA
        mem.write_memory(0x614,0x03); //($03,X)

        let mut cpu = CPU6502::new(mem);

        cpu.reset_at(0x0600);

        println!("CPU: {:?}", cpu);

        for _i in 0..11{
            cpu.run_single();
            println!("CPU: {:?}", cpu);
        }
        assert_eq!(cpu.A, 0x31);
        assert_eq!(cpu.X, 0x01);
        assert_eq!(cpu.Y, 0x00);
        assert_eq!(cpu.memory.read_memory(0), 0x20);
        assert_eq!(cpu.memory.read_memory(1), 0x10);
        assert_eq!(cpu.memory.read_memory(2), 0x30);
        assert_eq!(cpu.memory.read_memory(3), 0x00);
        assert_eq!(cpu.memory.read_memory(4), 0x02);
    }

    #[test]
    fn test2(){
        let mut mem = Memory::new(4*1024);

        mem.write_memory(0x600,0xa9); //LDA
        mem.write_memory(0x601,0xc0); //#$c0
        mem.write_memory(0x602,0xaa); //TAX
        mem.write_memory(0x603,0xe8); //INX
        mem.write_memory(0x604,0x69); //ADC
        mem.write_memory(0x605,0xc4); //#$c4

        let mut cpu = CPU6502::new(mem);

        cpu.reset_at(0x0600);

        println!("CPU: {:?}", cpu);

        for _i in 0..4{
            cpu.run_single();
            println!("CPU: {:?}", cpu);
        }
        assert_eq!(cpu.A, 0x84);
        assert_eq!(cpu.X, 0xc1);
        assert_eq!(cpu.Y, 0x00);
        assert!(cpu.P.get_N());
        assert!(cpu.P.get_C());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_BNE(){
        let mut mem = Memory::new(4*1024);

        mem.write_memory(0x600, 0xa9); //LDA
        mem.write_memory(0x601, 0x01); //#$01
        mem.write_memory(0x602, 0xc9); //CMP
        mem.write_memory(0x603, 0x02); //#$02
        mem.write_memory(0x604, 0xd0); //BNE
        mem.write_memory(0x605, 0x02); //+2
        mem.write_memory(0x606, 0x85); //STA
        mem.write_memory(0x607, 0x22); //$22
        mem.write_memory(0x608, 0xea); //NOP

        let mut cpu = CPU6502::new(mem);

        cpu.reset_at(0x0600);

        println!("CPU: {:?}", cpu);

        for _i in 0..4{
            cpu.run_single();
            println!("CPU: {:?}", cpu);
        }

        assert_eq!(cpu.A, 0x01);
        assert_eq!(cpu.X, 0x00);
        assert_eq!(cpu.Y, 0x00);
        assert!(cpu.P.get_N());
        assert!(!cpu.P.get_C());
        assert!(!cpu.P.get_Z());

    }

        #[test]
    #[allow(non_snake_case)]
    fn test_BNE2(){
        let mut mem = Memory::new(4*1024);

        mem.write_memory(0x600, 0xa9); //LDA
        mem.write_memory(0x601, 0x01); //#$01
        mem.write_memory(0x602, 0xc9); //CMP
        mem.write_memory(0x603, 0x01); //#$02
        mem.write_memory(0x604, 0xd0); //BNE
        mem.write_memory(0x605, 0x02); //+2
        mem.write_memory(0x606, 0x85); //STA
        mem.write_memory(0x607, 0x22); //$22
        mem.write_memory(0x608, 0xea); //NOP

        let mut cpu = CPU6502::new(mem);

        cpu.reset_at(0x0600);

        println!("CPU: {:?}", cpu);

        for _i in 0..4{
            cpu.run_single();
            println!("CPU: {:?}", cpu);
        }

        assert_eq!(cpu.A, 0x01);
        assert_eq!(cpu.X, 0x00);
        assert_eq!(cpu.Y, 0x00);
        assert_eq!(cpu.memory.read_memory(0x22), 0x01);
        assert!(!cpu.P.get_N());
        assert!(cpu.P.get_C());
        assert!(cpu.P.get_Z());
    }

    #[test]
    fn test_ind_jmp(){
        let mut mem = Memory::new(4*1024);

        mem.write_memory(0x600, 0xa9); //LDA
        mem.write_memory(0x601, 0x01); //#$01
        mem.write_memory(0x602, 0x85); //STA
        mem.write_memory(0x603, 0xf0); //$f0
        mem.write_memory(0x604, 0xa9); //LDA
        mem.write_memory(0x605, 0xcc); //#$cc
        mem.write_memory(0x606, 0x85); //STA
        mem.write_memory(0x607, 0xf1); //$f1
        mem.write_memory(0x608, 0x6c); //JMP
        mem.write_memory(0x609, 0xf0); //($f0)
        mem.write_memory(0x60a, 0x00); //($00) //($00f0)

        let mut cpu = CPU6502::new(mem);

        cpu.reset_at(0x0600);

        println!("CPU: {:?}", cpu);

        for _i in 0..5{
            cpu.run_single();
            println!("CPU: {:?}", cpu);
        }

        assert_eq!(cpu.PC, 0xcc01);
    }

    #[test]
    fn test_lda_indx(){
         let mut mem = Memory::new(4*1024);

        mem.write_memory(0x600, 0xa2); //LDX
        mem.write_memory(0x601, 0x01); //#$01
        mem.write_memory(0x602, 0xa9); //LDA
        mem.write_memory(0x603, 0x05); //#$05
        mem.write_memory(0x604, 0x85); //STA
        mem.write_memory(0x605, 0x01); //$01
        mem.write_memory(0x606, 0xa9); //LDA
        mem.write_memory(0x607, 0x07); //#$07
        mem.write_memory(0x608, 0x85); //STA
        mem.write_memory(0x609, 0x02); //$02
        mem.write_memory(0x60a, 0xa0); //LDY
        mem.write_memory(0x60b, 0x0a); //#$0a
        mem.write_memory(0x60c, 0x8c); //STY
        mem.write_memory(0x60d, 0x05); //
        mem.write_memory(0x60e, 0x07); //$0705
        mem.write_memory(0x60f, 0xa1); //LDA
        mem.write_memory(0x610, 0x00); //$(00,X)
        mem.write_memory(0x611, 0xea); //NOP

        let mut cpu = CPU6502::new(mem);

        cpu.reset_at(0x0600);

        println!("CPU: {:?}", cpu);

        for _i in 0..9{
            cpu.run_single();
            println!("CPU: {:?}", cpu);
        }

        assert_eq!(cpu.A, 0x0a);
        assert_eq!(cpu.X, 0x01);
        assert_eq!(cpu.Y, 0x0a);
        assert_eq!(cpu.memory.read_memory(0x0705), 0x0a);
        assert!(!cpu.P.get_N());
        assert!(!cpu.P.get_C());
        assert!(!cpu.P.get_Z());

    }

    #[test]
    fn test_lda_indy(){
         let mut mem = Memory::new(4*1024);

        mem.write_memory(0x600, 0xa0); //LDY
        mem.write_memory(0x601, 0x01); //#$01
        mem.write_memory(0x602, 0xa9); //LDA
        mem.write_memory(0x603, 0x03); //#$03
        mem.write_memory(0x604, 0x85); //STA
        mem.write_memory(0x605, 0x01); //$01
        mem.write_memory(0x606, 0xa9); //LDA
        mem.write_memory(0x607, 0x07); //#$07
        mem.write_memory(0x608, 0x85); //STA
        mem.write_memory(0x609, 0x02); //$02
        mem.write_memory(0x60a, 0xa2); //LDX
        mem.write_memory(0x60b, 0x0a); //#$0a
        mem.write_memory(0x60c, 0x8e); //STX
        mem.write_memory(0x60d, 0x04); //
        mem.write_memory(0x60e, 0x07); //$0704
        mem.write_memory(0x60f, 0xb1); //LDA
        mem.write_memory(0x610, 0x01); //$(01),Y
        mem.write_memory(0x611, 0xea); //NOP

        let mut cpu = CPU6502::new(mem);

        cpu.reset_at(0x0600);

        println!("CPU: {:?}", cpu);

        for _i in 0..9{
            cpu.run_single();
            println!("CPU: {:?}", cpu);
        }

        assert_eq!(cpu.A, 0x0a);
        assert_eq!(cpu.X, 0x0a);
        assert_eq!(cpu.Y, 0x01);
        assert_eq!(cpu.memory.read_memory(0x0704), 0x0a);
        assert!(!cpu.P.get_N());
        assert!(!cpu.P.get_C());
        assert!(!cpu.P.get_Z());

    }

        #[test]
    fn test_stack(){
         let mut mem = Memory::new(4*1024);

        mem.write_memory(0x600, 0xa2); //LDX
        mem.write_memory(0x601, 0x00); //#$01
        mem.write_memory(0x602, 0xa0); //LDY
        mem.write_memory(0x603, 0x00); //#$00
        mem.write_memory(0x604, 0x8a); //TXA
        mem.write_memory(0x605, 0x99); //STA
        mem.write_memory(0x606, 0x00);
        mem.write_memory(0x607, 0x02); //$0200,Y
        mem.write_memory(0x608, 0x48); //PHA
        mem.write_memory(0x609, 0xe8); //INX
        mem.write_memory(0x60a, 0xc8); //INY
        mem.write_memory(0x60b, 0xc0); //CPY
        mem.write_memory(0x60c, 0x10); //#$10
        mem.write_memory(0x60d, 0xd0); //BNE
        mem.write_memory(0x60e, 0xf5); //$0604
        mem.write_memory(0x60f, 0x68); //PLA
        mem.write_memory(0x610, 0x99); //STA
        mem.write_memory(0x611, 0x00);
        mem.write_memory(0x612, 0x02); //$0200,Y
        mem.write_memory(0x613, 0xc8); //INY
        mem.write_memory(0x614, 0xc0); //CPY
        mem.write_memory(0x615, 0x20); //#$20
        mem.write_memory(0x616, 0xd0); //BNE
        mem.write_memory(0x617, 0xf7); //$060f
        mem.write_memory(0x618, 0xea); //NOP

        let mut cpu = CPU6502::new(mem);

        cpu.reset_at(0x0600);

        for _i in 0..195{
            cpu.run_single();
        }

        println!("CPU: {:?}", cpu);

        assert_eq!(cpu.A, 0x00);
        assert_eq!(cpu.X, 0x10);
        assert_eq!(cpu.Y, 0x20);
        assert!(!cpu.P.get_N());
        assert!(cpu.P.get_C());
        assert!(cpu.P.get_Z());

    }

    #[test]
    fn test_jmp(){
         let mut mem = Memory::new(4*1024);

        mem.write_memory(0x600, 0xa9); //LDA
        mem.write_memory(0x601, 0x03); //#$03
        mem.write_memory(0x602, 0x4c); //JMP
        mem.write_memory(0x603, 0x08);
        mem.write_memory(0x604, 0x06); //$0608
        mem.write_memory(0x605, 0x00); //BRK
        mem.write_memory(0x606, 0x00); //BRK
        mem.write_memory(0x607, 0x00); //BRK
        mem.write_memory(0x608, 0x8d); //STA
        mem.write_memory(0x609, 0x00);
        mem.write_memory(0x60a, 0x02); //$0200
        mem.write_memory(0x60b, 0xea); //NOP

        let mut cpu = CPU6502::new(mem);

        cpu.reset_at(0x0600);

        println!("CPU: {:?}", cpu);

        for _i in 0..4{
            cpu.run_single();
            println!("CPU: {:?}", cpu);
        }

        assert_eq!(cpu.A, 0x03);
        assert_eq!(cpu.X, 0x00);
        assert_eq!(cpu.Y, 0x00);
        assert!(!cpu.P.get_N());
        assert!(!cpu.P.get_C());
        assert!(!cpu.P.get_Z());
        assert_eq!(cpu.memory.read_memory(0x0200), 0x03);
    }

    #[test]
    fn test_jsr_rts(){
         let mut mem = Memory::new(4*1024);

        mem.write_memory(0x600, 0x20); //JSR
        mem.write_memory(0x601, 0x09); //
        mem.write_memory(0x602, 0x06); //$0609
        mem.write_memory(0x603, 0x20); //JSR
        mem.write_memory(0x604, 0x0c); //
        mem.write_memory(0x605, 0x06); //$060c
        mem.write_memory(0x606, 0x20); //JSR
        mem.write_memory(0x607, 0x12); //
        mem.write_memory(0x608, 0x06); //$0612
        mem.write_memory(0x609, 0xa2); //LDX
        mem.write_memory(0x60a, 0x00); //#$00
        mem.write_memory(0x60b, 0x60); //RTS
        mem.write_memory(0x60c, 0xe8); //INX
        mem.write_memory(0x60d, 0xe0); //CPX
        mem.write_memory(0x60e, 0x05); //#$05
        mem.write_memory(0x60f, 0xd0); //BNE
        mem.write_memory(0x610, 0xfb); //$060c
        mem.write_memory(0x611, 0x60); //RTS
        mem.write_memory(0x612, 0xea); //NOP

        let mut cpu = CPU6502::new(mem);

        cpu.reset_at(0x0600);

        println!("CPU: {:?}", cpu);

        for _i in 0..22{
            cpu.run_single();
            //println!("CPU: {:?}", cpu);
        }

        assert_eq!(cpu.A, 0x00);
        assert_eq!(cpu.X, 0x05);
        assert_eq!(cpu.Y, 0x00);
        assert!(!cpu.P.get_N());
        assert!(cpu.P.get_C());
        assert!(cpu.P.get_Z());
    }

    #[test]
    fn test_all(){
        let mem = Memory::from_file("./tests/6502_functional_test.bin").unwrap();
        let mut cpu = CPU6502::new(mem);
        cpu.reset_at(0x0400);

        let mut cnt = 0;

        loop{
            cpu.run_single();
            println!("CPU: {:?}", cpu);
            cnt += 1;

            if cnt > 41000{
                assert!(false);
            }
        }
    }
}
