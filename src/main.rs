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
}

impl std::fmt::Debug for StatusRegister{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>{
        fmt.write_str(&format!("NV-BDIZC {:08b}", self.value))
    }
}

#[derive(Debug)]
struct Memory{
    memory: Vec<u8>,
}

impl Memory {
    fn new(size: usize) -> Self{
        Memory{ memory: vec![0; size]}
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

#[allow(non_snake_case)]
struct CPU6502{
    A:  u8,
    X:  u8,
    Y:  u8,
    PC: u16,
    SP:  u8,
    P:  StatusRegister,

    memory: Memory,

}

impl std::fmt::Debug for CPU6502 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>{
        fmt.write_str(&format!("PC={:#06x} A={:#04x} X={:#04x} Y={:#04x} SP={:#04x} P={:?}\n"
                                        , self.PC, self.A, self.X, self.Y, self.SP, self.P))?;
        let l = self.memory.memory.len() / 16;

        let mut last = [0xff; 16];
        let mut lasti = 0;

        for i in 0..l{
            let mslicee: [u8; 16] = self.memory.memory[i*16 .. (i+1)*16].try_into().unwrap();

            if mslicee != last{
                if lasti != i{
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

impl CPU6502 {
    fn new(mem: Memory) -> Self{
        CPU6502 { A: 0, X: 0, Y: 0, PC: 0, SP: 0xff, P: StatusRegister { value: 0 }, memory: mem }
    }

    fn reset(&mut self) {

    }

    fn get_address(&mut self, adrtype: AdressingType) -> u16{
        match adrtype {
            AdressingType::ZeroPage => {
                print!("PC={:#06x}", self.PC);
                let addr = self.memory.memory[self.PC as usize];
                self.PC += 1;
                let ret = addr as u16;
                println!(" Address=0x{:04x}", ret);
                ret
            }
            AdressingType::Absolute => {
                print!("PC={:#06x}", self.PC);
                let addr1 = self.PC as usize;
                self.PC += 2;
                print!(" Address1={:#06x}", addr1 as u16);
                let ret = u16::from_le_bytes(self.memory.memory[addr1 .. addr1+2].try_into().unwrap());
                println!(" Address={:#06x}", ret as u16);
                ret
            }
            AdressingType::AbsoluteX => {
                print!("PC={:#06x}", self.PC);
                let addr1 = self.PC as usize;
                self.PC += 2;
                print!(" Address1={:#06x}", addr1 as u16);
                let ret = u16::from_le_bytes(self.memory.memory[addr1 .. addr1+2].try_into().unwrap()).overflowing_add(self.X as u16).0;
                println!(" Address={:#06x}", ret as u16); //TODO check if overflow - page change +1 cycle
                ret
            }
            AdressingType::AbsoluteY => {
                print!("PC={:#06x}", self.PC);
                let addr1 = self.PC as usize;
                self.PC += 2;
                print!(" Address1={:#06x}", addr1 as u16);
                let ret = u16::from_le_bytes(self.memory.memory[addr1 .. addr1+2].try_into().unwrap()).overflowing_add(self.Y as u16).0;
                println!(" Address={:#06x}", ret as u16); //TODO check if overflow - page change +1 cycle
                ret
            }
            AdressingType::Indirect => {
                print!("PC={:#06x}", self.PC);
                let addr1 = self.PC as usize;
                self.PC += 2;
                print!(" Address1={:#06x}", addr1 as u16);
                let addr2 = u16::from_le_bytes(self.memory.memory[addr1 .. addr1+2].try_into().unwrap());
                print!(" Address2={:#06x}", addr2 as u16);
                let ret = u16::from_le_bytes(self.memory.memory[addr2 as usize .. (addr2+2) as usize].try_into().unwrap());
                println!(" Address={:#06x}", ret as u16);
                ret
            }
            AdressingType::IndirectX => {
                print!("PC={:#06x}", self.PC);
                let addr1 = self.memory.memory[self.PC as usize].overflowing_add(self.X.into()).0 as usize;
                self.PC += 1;
                print!(" Address1={:#06x}", addr1 as u16);
                let ret = u16::from_le_bytes(self.memory.memory[addr1 .. addr1+2].try_into().unwrap());
                println!(" Address={:#06x}", ret as u16);
                ret
            }
            AdressingType::IndirectY => {
                print!("PC={:#06x}", self.PC);
                let addr1 = self.memory.memory[self.PC as usize] as usize;
                self.PC += 1;
                print!(" Address1={:#06x}", addr1 as u16);
                let ret = u16::from_le_bytes(self.memory.memory[addr1 .. addr1+2].try_into().unwrap()).overflowing_add(self.Y as u16);
                println!(" Address={:#06x}", ret.0);
                ret.0
            }
            _ => {
                todo!("Addressing Type {:?}", adrtype);
            }
        }
    }

    fn run_single(&mut self){
        let ins = self.memory.memory[self.PC as usize];
        self.PC += 1;

        print!("Running INS={:#04x} PC={:#06x} ", ins, self.PC);

        match ins {
            0x00 => { //BRK
                println!("BRK");
                todo!("BRK");
            }

            0x01 => { //ORA IndirectX
                println!("ORA IndirectX");
                let address = self.get_address(AdressingType::IndirectX);
                let data = self.memory.memory[address as usize];
                self.A |= data;
                self.P.set_NZ(self.A);
            }

            0x05 => { //ORA ZeroPage
                println!("ORA ZeroPage");
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.memory[address as usize];
                self.A |= data;
                self.P.set_NZ(self.A);
            }

            0x06 => { //ASL ZeroPage
                println!("ASL ZeroPage");
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.memory.get_mut(address as usize).unwrap();
                self.P.set_C(*data & 0b1000_0000 != 0);
                *data = *data << 1;
                self.P.set_NZ(*data);
            }

            0x08 => { //PHP
                println!("PHP");
                todo!("PHP={:02x}", ins);
            }

            0x09 => { //ORA Immediate
                println!("ORA Immediate");
                let data = self.memory.memory[self.PC as usize];
                self.PC += 1;
                self.A |= data;
                self.P.set_NZ(self.A);
            }

            0x48 => { //PHA
                println!("PHA");
                let r = self.SP.overflowing_sub(1);
                self.SP = r.0;
                if r.1{
                    println!("Stack Overflow!");
                }
                let address = 0x0100 | self.SP as u16;
                let data = self.memory.memory.get_mut(address as usize).unwrap();
                *data = self.A;
                println!("Pushed A={:#04x} ADDR={:#06x}", data, address);
            }

            0x68 => {
                println!("PLA");
                let address = 0x0100 | self.SP as u16;
                let data = self.memory.memory[address as usize];
                self.A = data;
                let r = self.SP.overflowing_add(1);
                self.SP = r.0;
                self.P.set_NZ(data);
                if r.1{
                    println!("Stack Overflow!");
                }
                println!("Pop A={:#04x} ADDR={:#06x}", data, address);
            }

            0x69 => { //ADC Immediate
                println!("ADC Immediate");
                let data = self.memory.memory[self.PC as usize];
                self.PC += 1;
                let r = self.A.overflowing_add(data);
                self.A = r.0;
                self.P.set_NZ(r.0);
                self.P.set_C(r.1);
            }

            0x6c => { //JMP Indirect
                println!("JMP Indirect");
                let address = self.get_address(AdressingType::Indirect);
                self.PC = address;
            }

            0x85 => { //STA ZeroPage
                println!("STA ZeroPage");
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.memory.get_mut(address as usize).unwrap();
                *data = self.A;
            }

            0x8a => { //TXA
                println!("TXA");
                self.A = self.X;
                self.P.set_NZ(self.A);
            }

            0x8c => { //STY Absolute
                println!("STY Absolute");
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.memory.get_mut(address as usize).unwrap();
                *data = self.Y;
            }

            0x8e => { //STX Absolute
                println!("STY Absolute");
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.memory.get_mut(address as usize).unwrap();
                *data = self.X;
            }

            0x99 => { //STA 
                let address = self.get_address(AdressingType::AbsoluteY);
                let data = self.memory.memory.get_mut(address as usize).unwrap();
                *data = self.A;
            }

            0xa0 => { //LDY Immediate
                println!("LDY Immediate");
                let data = self.memory.memory[self.PC as usize];
                self.PC += 1;
                self.Y = data;
                self.P.set_NZ(data);
            }

            0xa1 => { //LDA IndirectX
                println!("LDA IndirectX");
                let address = self.get_address(AdressingType::IndirectX);
                let data = self.memory.memory[address as usize];
                self.A = data;
                self.P.set_NZ(data);
            }

            0xa2 => { //LDX Immediate
                println!("LDX Immediate");
                let data = self.memory.memory[self.PC as usize];
                self.PC += 1;
                self.X = data;
                self.P.set_NZ(data);
            }

            0xa9 => { //LDA Immediate
                println!("LDA Immediate");
                let data = self.memory.memory[self.PC as usize];
                self.PC += 1;
                self.A = data;
                self.P.set_NZ(data);
            }

            0xaa => { //TAX
                println!("TAX");
                self.X = self.A;
                self.P.set_NZ(self.X);
            }

            0xb1 => { //LDA IndirectY
                println!("LDA IndirectY");
                let address = self.get_address(AdressingType::IndirectY);
                let data = self.memory.memory[address as usize];
                self.A = data;
                self.P.set_NZ(data);
            }

            0xc0 => { //CPY Immediate
                println!("CPY Immediate");
                let data = self.memory.memory[self.PC as usize];
                self.PC += 1;
                let r = self.Y.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);
            }

            0xc8 => { //INY
                println!("INY");
                self.Y = self.Y.overflowing_add(1).0;
                self.P.set_NZ(self.Y);
            }

            0xc9 => { //CMP Immediate
                println!("CMP Immediate");
                let data = self.memory.memory[self.PC as usize];
                self.PC += 1;
                let r = self.A.overflowing_sub(data);

                println!("CMP v={:?}", r);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);
            }

            0xd0 => { //BNE Relative
                println!("BNE Relative");
                let data = self.memory.memory[self.PC as usize] as i8;
                self.PC += 1;

                if !self.P.get_Z(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    println!("Branch to PC={:#06x}", r.0);
                    self.PC = r.0 as u16;
                }
                else {
                    println!("NOT Branching");
                }
            }

            0xe8 => { //INX
                println!("INX");
                let r = self.X.overflowing_add(1);
                self.X = r.0;
                self.P.set_NZ(r.0);
                self.P.set_C(r.1);
            }

            0xea => { //NOP
                println!("NOP");
            }

            _  => {
                todo!("INS={:#04x}", ins);
            }
        }
    }
}

#[cfg(test)]
mod tests{
    use crate::{Memory,CPU6502};
    #[test]
    fn test1(){
        let mut mem = Memory::new(4*1024);

        mem.memory[0x600] = 0xa9; //LDA
        mem.memory[0x601] = 0x10; //#$10
        mem.memory[0x602] = 0x85; //STA
        mem.memory[0x603] = 0x00; //$00
        mem.memory[0x604] = 0x85; //STA
        mem.memory[0x605] = 0x01; //$01
        mem.memory[0x606] = 0x06; //ASL
        mem.memory[0x607] = 0x00; //$00
        mem.memory[0x608] = 0xa9; //LDA
        mem.memory[0x609] = 0x30; //#$30
        mem.memory[0x60a] = 0x85; //STA
        mem.memory[0x60b] = 0x02; //$02
        mem.memory[0x60c] = 0xa9; //LDA
        mem.memory[0x60d] = 0x02; //#$02
        mem.memory[0x60e] = 0x85; //STA
        mem.memory[0x60f] = 0x04; //$04
        mem.memory[0x610] = 0xa9; //LDA
        mem.memory[0x611] = 0x01; //#$01
        mem.memory[0x612] = 0xaa; //TAX
        mem.memory[0x613] = 0x01; //ORA
        mem.memory[0x614] = 0x03; //($03,X)

        let mut cpu = CPU6502::new(mem);

        cpu.reset();

        cpu.PC = 0x0600;

        println!("CPU: {:?}", cpu);

        for _i in 0..11{
            cpu.run_single();
            println!("CPU: {:?}", cpu);
        }
        assert_eq!(cpu.A, 0x31);
        assert_eq!(cpu.X, 0x01);
        assert_eq!(cpu.Y, 0x00);
        assert_eq!(cpu.memory.memory[0], 0x20);
        assert_eq!(cpu.memory.memory[1], 0x10);
        assert_eq!(cpu.memory.memory[2], 0x30);
        assert_eq!(cpu.memory.memory[3], 0x00);
        assert_eq!(cpu.memory.memory[4], 0x02);
    }

    #[test]
    fn test2(){
        let mut mem = Memory::new(4*1024);

        mem.memory[0x600] = 0xa9; //LDA
        mem.memory[0x601] = 0xc0; //#$c0
        mem.memory[0x602] = 0xaa; //TAX
        mem.memory[0x603] = 0xe8; //INX
        mem.memory[0x604] = 0x69; //ADC
        mem.memory[0x605] = 0xc4; //#$c4

        let mut cpu = CPU6502::new(mem);

        cpu.reset();

        cpu.PC = 0x0600;

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

        mem.memory[0x600] = 0xa9; //LDA
        mem.memory[0x601] = 0x01; //#$01
        mem.memory[0x602] = 0xc9; //CMP
        mem.memory[0x603] = 0x02; //#$02
        mem.memory[0x604] = 0xd0; //BNE
        mem.memory[0x605] = 0x02; //+2
        mem.memory[0x606] = 0x85; //STA
        mem.memory[0x607] = 0x22; //$22
        mem.memory[0x608] = 0xea; //NOP

        let mut cpu = CPU6502::new(mem);

        cpu.reset();

        cpu.PC = 0x0600;

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

        mem.memory[0x600] = 0xa9; //LDA
        mem.memory[0x601] = 0x01; //#$01
        mem.memory[0x602] = 0xc9; //CMP
        mem.memory[0x603] = 0x01; //#$02
        mem.memory[0x604] = 0xd0; //BNE
        mem.memory[0x605] = 0x02; //+2
        mem.memory[0x606] = 0x85; //STA
        mem.memory[0x607] = 0x22; //$22
        mem.memory[0x608] = 0xea; //NOP

        let mut cpu = CPU6502::new(mem);

        cpu.reset();

        cpu.PC = 0x0600;

        println!("CPU: {:?}", cpu);

        for _i in 0..4{
            cpu.run_single();
            println!("CPU: {:?}", cpu);
        }

        assert_eq!(cpu.A, 0x01);
        assert_eq!(cpu.X, 0x00);
        assert_eq!(cpu.Y, 0x00);
        assert_eq!(cpu.memory.memory[0x22], 0x01);
        assert!(!cpu.P.get_N());
        assert!(cpu.P.get_C());
        assert!(cpu.P.get_Z());
    }

    #[test]
    fn test_ind_jmp(){
        let mut mem = Memory::new(4*1024);

        mem.memory[0x600] = 0xa9; //LDA
        mem.memory[0x601] = 0x01; //#$01
        mem.memory[0x602] = 0x85; //STA
        mem.memory[0x603] = 0xf0; //$f0
        mem.memory[0x604] = 0xa9; //LDA
        mem.memory[0x605] = 0xcc; //#$cc
        mem.memory[0x606] = 0x85; //STA
        mem.memory[0x607] = 0xf1; //$f1
        mem.memory[0x608] = 0x6c; //JMP
        mem.memory[0x609] = 0xf0; //($f0)
        mem.memory[0x60a] = 0x00; //($00) //($00f0)

        let mut cpu = CPU6502::new(mem);

        cpu.reset();

        cpu.PC = 0x0600;

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

        mem.memory[0x600] = 0xa2; //LDX
        mem.memory[0x601] = 0x01; //#$01
        mem.memory[0x602] = 0xa9; //LDA
        mem.memory[0x603] = 0x05; //#$05
        mem.memory[0x604] = 0x85; //STA
        mem.memory[0x605] = 0x01; //$01
        mem.memory[0x606] = 0xa9; //LDA
        mem.memory[0x607] = 0x07; //#$07
        mem.memory[0x608] = 0x85; //STA
        mem.memory[0x609] = 0x02; //$02
        mem.memory[0x60a] = 0xa0; //LDY
        mem.memory[0x60b] = 0x0a; //#$0a
        mem.memory[0x60c] = 0x8c; //STY
        mem.memory[0x60d] = 0x05; //
        mem.memory[0x60e] = 0x07; //$0705
        mem.memory[0x60f] = 0xa1; //LDA
        mem.memory[0x610] = 0x00; //$(00,X)
        mem.memory[0x611] = 0xea; //NOP

        let mut cpu = CPU6502::new(mem);

        cpu.reset();

        cpu.PC = 0x0600;

        println!("CPU: {:?}", cpu);

        for _i in 0..9{
            cpu.run_single();
            println!("CPU: {:?}", cpu);
        }

        assert_eq!(cpu.A, 0x0a);
        assert_eq!(cpu.X, 0x01);
        assert_eq!(cpu.Y, 0x0a);
        assert_eq!(cpu.memory.memory[0x0705], 0x0a);
        assert!(!cpu.P.get_N());
        assert!(!cpu.P.get_C());
        assert!(!cpu.P.get_Z());

    }

    #[test]
    fn test_lda_indy(){
         let mut mem = Memory::new(4*1024);

        mem.memory[0x600] = 0xa0; //LDY
        mem.memory[0x601] = 0x01; //#$01
        mem.memory[0x602] = 0xa9; //LDA
        mem.memory[0x603] = 0x03; //#$03
        mem.memory[0x604] = 0x85; //STA
        mem.memory[0x605] = 0x01; //$01
        mem.memory[0x606] = 0xa9; //LDA
        mem.memory[0x607] = 0x07; //#$07
        mem.memory[0x608] = 0x85; //STA
        mem.memory[0x609] = 0x02; //$02
        mem.memory[0x60a] = 0xa2; //LDX
        mem.memory[0x60b] = 0x0a; //#$0a
        mem.memory[0x60c] = 0x8e; //STX
        mem.memory[0x60d] = 0x04; //
        mem.memory[0x60e] = 0x07; //$0704
        mem.memory[0x60f] = 0xb1; //LDA
        mem.memory[0x610] = 0x01; //$(01),Y
        mem.memory[0x611] = 0xea; //NOP

        let mut cpu = CPU6502::new(mem);

        cpu.reset();

        cpu.PC = 0x0600;

        println!("CPU: {:?}", cpu);

        for _i in 0..9{
            cpu.run_single();
            println!("CPU: {:?}", cpu);
        }

        assert_eq!(cpu.A, 0x0a);
        assert_eq!(cpu.X, 0x0a);
        assert_eq!(cpu.Y, 0x01);
        assert_eq!(cpu.memory.memory[0x0704], 0x0a);
        assert!(!cpu.P.get_N());
        assert!(!cpu.P.get_C());
        assert!(!cpu.P.get_Z());

    }

        #[test]
    fn test_stack(){
         let mut mem = Memory::new(4*1024);

        mem.memory[0x600] = 0xa2; //LDX
        mem.memory[0x601] = 0x00; //#$01
        mem.memory[0x602] = 0xa0; //LDY
        mem.memory[0x603] = 0x00; //#$00
        mem.memory[0x604] = 0x8a; //TXA
        mem.memory[0x605] = 0x99; //STA
        mem.memory[0x606] = 0x00;
        mem.memory[0x607] = 0x02; //$0200,Y
        mem.memory[0x608] = 0x48; //PHA
        mem.memory[0x609] = 0xe8; //INX
        mem.memory[0x60a] = 0xc8; //INY
        mem.memory[0x60b] = 0xc0; //CPY
        mem.memory[0x60c] = 0x10; //#$10
        mem.memory[0x60d] = 0xd0; //BNE
        mem.memory[0x60e] = 0xf5; //$0604
        mem.memory[0x60f] = 0x68; //PLA
        mem.memory[0x610] = 0x99; //STA
        mem.memory[0x611] = 0x00;
        mem.memory[0x612] = 0x02; //$0200,Y
        mem.memory[0x613] = 0xc8; //INY
        mem.memory[0x614] = 0xc0; //CPY
        mem.memory[0x615] = 0x20; //#$20
        mem.memory[0x616] = 0xd0; //BNE
        mem.memory[0x617] = 0xf7; //$060f
        mem.memory[0x618] = 0xea; //NOP

        let mut cpu = CPU6502::new(mem);

        cpu.reset();

        cpu.PC = 0x0600;

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
}

fn main() {

    let mut mem = Memory::new(4*1024);

    mem.memory[0x600] = 0xa2; //LDX
    mem.memory[0x601] = 0x01; //#$01
    mem.memory[0x602] = 0xa9; //LDA
    mem.memory[0x603] = 0x05; //#$05
    mem.memory[0x604] = 0x85; //STA
    mem.memory[0x605] = 0x01; //$01
    mem.memory[0x606] = 0xa9; //LDA
    mem.memory[0x607] = 0x07; //#$07
    mem.memory[0x608] = 0x85; //STA
    mem.memory[0x609] = 0x02; //$02
    mem.memory[0x60a] = 0xa0; //LDY
    mem.memory[0x60b] = 0x0a; //#$0a
    mem.memory[0x60c] = 0x8c; //STY
    mem.memory[0x60d] = 0x05; //
    mem.memory[0x60e] = 0x07; //$0705
    mem.memory[0x60f] = 0xa1; //LDA
    mem.memory[0x610] = 0x00; //$(00,X)
    mem.memory[0x611] = 0xea; //$(00,X)

    let mut cpu = CPU6502::new(mem);

    cpu.reset();

    cpu.PC = 0x0600;

    println!("CPU: {:?}", cpu);

    for _i in 0..9{
        cpu.run_single();
        println!("CPU: {:?}", cpu);
    }

    println!("Hello, world!");
}
