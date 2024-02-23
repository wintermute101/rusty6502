pub mod memory;
use std::collections::VecDeque;
use std::error::Error;

use self::memory::{Memory6502, Memory6502Debug};

#[derive(Clone,Copy)]
struct StatusRegister{
    value: u8,
}

#[allow(non_snake_case)]
impl StatusRegister {
    fn set_NZ(&mut self, val: u8){
        self.value = (self.value & 0b0111_1111) | (val & 0b1000_0000);
        self.value = (self.value & 0b1111_1101) | ((val == 0) as u8) << 1;
    }

    fn set_Z(&mut self, val: bool){
        self.value = (self.value & 0b1111_1101) | (val as u8) << 1;
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

#[derive(Debug)]
pub struct CpuError{
    pub pc: u16,
    error_string: String,
}

impl CpuError {
    fn new(error: &str, pc: u16) -> Self{
        CpuError{error_string: error.to_owned(), pc: pc}
    }
}

impl std::fmt::Display for CpuError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>{
        fmt.write_str(&format!("[{} PC={:#06x}]", self.error_string, self.pc))
    }
}

impl Error for CpuError {
}

#[derive(PartialEq)]
pub enum InterruptType {
    INT,
    NMI,
    BRK,
}

#[derive(Clone)]
#[allow(non_snake_case)]
struct CPUState{
    ins: u8,
    op1: u8,
    op2: u8,

    A:  u8,
    X:  u8,
    Y:  u8,
    P:  StatusRegister,
    SP:  u8,
    PC: u16,
    adr: u16,
}

impl CPUState {

    fn instruction_name(&self) -> String{
        match self.ins{
            0x00 => "BRK",
            0x01 => "ORA IndirectX",
            0x05 => "ORA ZeroPage",
            0x06 => "ASL ZeroPage",
            0x08 => "PHP",
            0x09 => "ORA Immediate",
            0x0a => "ASL Accumulator",
            0x0d => "ORA Absolute",
            0x0e => "ASL Absolute",
            0x10 => "BPL",
            0x11 => "ORA AbsoluteX",
            0x15 => "ORA ZeroPageX",
            0x16 => "ASL ZeroPageX",
            0x18 => "CLC",
            0x19 => "ORA AbsoluteY",
            0x1d => "ORA AbsoluteX",
            0x1e => "ASL AbsoluteX",
            0x20 => "JSR",
            0x21 => "AND IndirectX",
            0x24 => "BIT ZeroPage",
            0x25 => "AND ZeroPage",
            0x26 => "ROL ZeroPage",
            0x28 => "PLP",
            0x29 => "AND Immidiate",
            0x2a => "ROL Accumulator",
            0x2c => "BIT Absolute",
            0x2d => "AND Absolute",
            0x2e => "ROL Absolute",
            0x30 => "BMI Relative",
            0x31 => "AND IndirectY",
            0x35 => "AND ZeroPageX",
            0x36 => "ROL ZeroPageX",
            0x38 => "SEC",
            0x39 => "AND AbsoluteY",
            0x3d => "AND AbsoluteX",
            0x3e => "ROL AbsoluteX",
            0x40 => "RTI",
            0x41 => "EOR IndirectX",
            0x45 => "EOR ZeroPage",
            0x46 => "LSR ZeroPage",
            0x48 => "PHA",
            0x49 => "EOR Immediate",
            0x4a => "LSR Accumulator",
            0x4c => "JMP Absolute",
            0x4d => "EOR Absolute",
            0x4e => "LSR Absolute",
            0x50 => "BVC",
            0x51 => "EOR IndirectY",
            0x55 => "EOR ZeroPageX",
            0x56 => "LSR ZeroPageX",
            0x58 => "CLI",
            0x59 => "EOR AbsoluteY",
            0x5d => "EOR AbsoluteX",
            0x5e => "LSR AbsoluteX",
            0x60 => "RTS",
            0x61 => "ADC IndirectX",
            0x65 => "ADC ZeroPage",
            0x66 => "ROR ZeroPage",
            0x68 => "PLA",
            0x69 => "ADC Immediate",
            0x6a => "ROR Accumulator",
            0x6c => "JMP Indirect",
            0x6d => "ADC Absolute",
            0x6e => "ROR Absolute",
            0x70 => "BVS",
            0x71 => "ADC IndirectY",
            0x75 => "ADC ZeroPageX",
            0x76 => "ROR ZeroPageX",
            0x78 => "SEI",
            0x79 => "ADC AbsoluteY",
            0x7d => "ADC AbsoluteX",
            0x7e => "ROR AbsoluteX",
            0x81 => "STA IndirectX",
            0x84 => "STY ZeroPage",
            0x85 => "STA ZeroPage",
            0x86 => "STX ZeroPage",
            0x88 => "DEY",
            0x8a => "TXA",
            0x8c => "STY Absolute",
            0x8d => "STA Absolute",
            0x8e => "STX Absolute",
            0x90 => "BCC",
            0x91 => "STA IndirectY",
            0x94 => "STY ZeroPageX",
            0x95 => "STA ZeroPageX",
            0x96 => "STX ZeroPageY",
            0x98 => "TYA",
            0x99 => "STA AbsoluteY",
            0x9a => "TXS",
            0x9d => "STA AbsoluteX",
            0xa0 => "LDY Immediate",
            0xa1 => "LDA IndirectX",
            0xa2 => "LDX Immediate",
            0xa4 => "LDY ZeroPage",
            0xa5 => "LDA ZeroPage",
            0xa6 => "LDX ZeroPage",
            0xa8 => "TAY",
            0xa9 => "LDA Immediate",
            0xaa => "TAX",
            0xac => "LDY Absolute",
            0xad => "LDA Absolute",
            0xae => "LDX Absolute",
            0xb0 => "BCS",
            0xb1 => "LDA IndirectY",
            0xb4 => "LDY ZeroPageX",
            0xb5 => "LDA ZeroPageX",
            0xb6 => "LDX ZeroPageY",
            0xb8 => "CLV",
            0xb9 => "LDA AbsoluteY",
            0xba => "TSX",
            0xbc => "LDY AbsoluteX",
            0xbd => "LDA AbsoluteX",
            0xbe => "LDX AbsoluteY",
            0xc0 => "CPY Immediate",
            0xc1 => "CMP IndirectX",
            0xc4 => "CPY ZeroPage",
            0xc5 => "CMP ZeroPage",
            0xc6 => "DEC ZeroPage",
            0xc8 => "INY",
            0xc9 => "CMP Immediate",
            0xca => "DEX",
            0xcc => "CPY Absolute",
            0xcd => "CMP Absolute",
            0xce => "DEC Absolute",
            0xd0 => "BNE Relative",
            0xd1 => "CMP IndirectY",
            0xd5 => "CMP ZeroPageX",
            0xd6 => "DEC ZeroPageX",
            0xd8 => "CLD",
            0xd9 => "CMP AbsoluteY",
            0xdd => "CMP AbsoluteX",
            0xde => "DEC AbsoluteX",
            0xe0 => "CPX Immediate",
            0xe1 => "SBC IndirectX",
            0xe4 => "CPX ZeroPage",
            0xe5 => "SBC ZeroPage",
            0xe6 => "INC ZeroPage",
            0xe8 => "INX",
            0xe9 => "SBC Immidiate",
            0xea => "NOP",
            0xec => "CPX Absolute",
            0xed => "SBC Absolute",
            0xee => "INC Absolute",
            0xf0 => "BEQ Relative",
            0xf1 => "SBC IndirectY",
            0xf5 => "SBC ZeroPageX",
            0xf6 => "INC ZeroPageX",
            0xf8 => "SED",
            0xf9 => "SBC AbsoluteY",
            0xfd => "SBC AbsoluteX",
            0xfe => "INC AbsoluteX",
            _    => "INV Invalid"
        }.to_owned()
    }

    fn new<MemT>(cpu: &CPU6502<MemT>, ins: u8) -> Self{
        CPUState { ins: ins, op1: 0, op2: 0, A: cpu.A, X: cpu.X, Y: cpu.Y, P: cpu.P, SP: cpu.SP, PC: cpu.PC, adr: 0 }
    }
}

impl std::fmt::Debug for CPUState{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>{
        fmt.write_str(&format!("INS={:#04x} A={:#04x} X={:#04x} Y={:#04x} P={:?} SP={:#04x} PC={:#06x} OP1={:#04x} OP2={:#04x} ADDR={:#06x} {}\n",
        self.ins, self.A, self.X, self.Y, self.P, self.SP, self.PC, self.op1, self.op2, self.adr, self.instruction_name()))
    }
}

#[allow(non_snake_case)]
pub struct CPU6502<MemT>{
    A:  u8,
    X:  u8,
    Y:  u8,
    PC: u16,
    SP:  u8,
    P:  StatusRegister,

    prev_PC: u16,
    memory: MemT,

    trace_line_limit : usize,
    trace: Option<VecDeque<CPUState>>,
}

impl<MemT: Memory6502> std::fmt::Debug for CPU6502<MemT> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>{
        fmt.write_str(&format!("PC={:#06x} A={:#04x} X={:#04x} Y={:#04x} SP={:#04x} P={:?}"
                                        , self.PC, self.A, self.X, self.Y, self.SP, self.P))
        //fmt.write_str(&format!("{:?}", self.memory))
        //fmt.write_str(&format!(""))
    }
}

impl<MemT: Memory6502 + Memory6502Debug> CPU6502<MemT> {
    pub fn new(mem: MemT) -> Self{
        CPU6502 { A: 0, X: 0, Y: 0, PC: 0, SP: 0xff, P: StatusRegister { value: 0 }, prev_PC: 0, memory: mem, trace: None, trace_line_limit: 0 }
    }

    pub fn enable_trace(&mut self, trace_size_limit: usize){
        self.trace = Some(VecDeque::with_capacity(trace_size_limit));
        self.trace_line_limit = trace_size_limit;
    }

    pub fn reset(&mut self) {
        let resetvec_addr = self.memory.read_memory_word(0xfffc);
        self.PC = resetvec_addr;
    }

    pub fn reset_at(&mut self, start_address: u16) {
        self.PC = start_address;
    }

    fn adc(&mut self, mut state: CPUState, value: u8){
        state.op1 = value;
        if self.P.get_D(){
            self.P.set_V(false);

            let lowv = value & 0x0f;
            let lowa = self.A & 0x0f;

            let highv = value >> 4;
            let higha = self.A >> 4;

            let rem = if (lowv + lowa + self.P.get_C() as u8) > 9{
                1
            }
            else{
                0
            };

            self.A = (lowa + lowv + self.P.get_C() as u8) % 10 | ((higha + highv + rem) % 10 << 4);
            self.P.set_C(highv + higha + rem > 9);

            state.P = self.P;
            state.A = self.A;
            self.add_trace(state);
            return;
        }

        let data = value as u16 + self.A as u16 + self.P.get_C() as u16;

        let a = value & 0x80 != 0;
        let b = self.A & 0x80 != 0;
        let c = data as u8 & 0x80 != 0;

        self.P.set_C(data > 0xff);
        self.P.set_NZ(data as u8);
        self.P.set_V(!a & !b & c | a & b & !c);
        self.A = data as u8;

        state.P = self.P;
        state.A = self.A;
        self.add_trace(state);
    }

    fn sbc(&mut self, mut state: CPUState, value: u8){
        if self.P.get_D(){
            self.P.set_V(false);
            let lowv = (value & 0x0f) as i8;
            let lowa = (self.A & 0x0f) as i8;

            let highv = (value >> 4) as i8;
            let higha = (self.A >> 4) as i8;

            let rem = if (lowa - lowv - !self.P.get_C() as i8) < 0{
                1
            }
            else{
                0
            };

            self.A = (lowa - lowv - !self.P.get_C() as i8).rem_euclid(10) as u8 | (((higha - highv - rem).rem_euclid(10) as u8) << 4);
            self.P.set_C(!(higha - highv - rem < 0));

            state.P = self.P;
            state.op1 = value;
            state.A = self.A;
            self.add_trace(state);
            return;
        }

        self.adc(state, !value);
    }

    fn add_trace(&mut self, state: CPUState){
        if let Some(buf) = self.trace.as_mut(){
            if buf.len() == self.trace_line_limit{
                buf.pop_front();
            }
            buf.push_back(state);
        }
    }

    pub fn show_trace(&self){
        if let Some(buf) = self.trace.as_ref(){
            println!("****  Trace   ****");
            for i in buf{
                print!("{:?}", i);
            }
        }
    }

    pub fn show_cpu_debug(&self){
            self.show_trace();
            println!("{:?}", self);
            println!("**** ZeroPage ****");
            self.memory.show_zero_page();
            println!("****  Stack   ****");
            self.memory.show_stack();
    }

    fn get_address(&mut self, adrtype: AdressingType) -> u16{
        match adrtype {
            AdressingType::ZeroPage => {
                let addr = self.memory.read_memory(self.PC);
                self.PC += 1;
                let ret = addr as u16;
                ret
            }
            AdressingType::ZeroPageX => {
                let addr = self.memory.read_memory(self.PC).overflowing_add(self.X).0;
                self.PC += 1;
                let ret = addr as u16;
                ret
            }
            AdressingType::ZeroPageY => {
                let addr = self.memory.read_memory(self.PC).overflowing_add(self.Y).0;
                self.PC += 1;
                let ret = addr as u16;
                ret
            }
            AdressingType::Absolute => {
                let ret = self.memory.read_memory_word(self.PC);
                self.PC += 2;
                ret
            }
            AdressingType::AbsoluteX => {
                let ret = self.memory.read_memory_word(self.PC).overflowing_add(self.X as u16).0;
                self.PC += 2;
                ret
            }
            AdressingType::AbsoluteY => {
                let ret = self.memory.read_memory_word(self.PC).overflowing_add(self.Y as u16).0;
                self.PC += 2;
                ret
            }
            AdressingType::Indirect => {
                let addr1 = self.memory.read_memory_word(self.PC);
                self.PC += 2;
                let ret = self.memory.read_memory_word(addr1);
                ret
            }
            AdressingType::IndirectX => {
                let addr1 = self.memory.read_memory(self.PC).overflowing_add(self.X.into()).0;
                self.PC += 1;
                let ret = self.memory.read_memory_word(addr1 as u16);
                ret
            }
            AdressingType::IndirectY => {
                let addr1 = self.memory.read_memory(self.PC);
                self.PC += 1;
                let ret = self.memory.read_memory_word(addr1 as u16).overflowing_add(self.Y as u16);
                ret.0
            }
        }
    }

    pub fn run_single(&mut self) -> Result<(), CpuError>{
        let ins = self.memory.read_memory(self.PC);
        let mut cpu_state = CPUState::new(self, ins);
        //build cpu state before we mess PC
        let pc = self.PC.overflowing_add(1);
        if pc.1{
            return Err(CpuError::new("CPU Windup", self.PC));
        }
        self.PC = pc.0;

        match ins {
            0x00 => { //BRK
                self.PC += 1;
                self.add_trace(cpu_state);
                self.interrupt(InterruptType::BRK);

            }

            0x01 => { //ORA IndirectX
                let address = self.get_address(AdressingType::IndirectX);
                let data = self.memory.read_memory(address);
                self.A |= data;
                self.P.set_NZ(self.A);

                cpu_state.A = self.A;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0x05 => { //ORA ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                self.A |= data;
                self.P.set_NZ(self.A);

                cpu_state.A = self.A;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0x06 => { //ASL ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                self.P.set_C(data & 0b1000_0000 != 0);
                data = data << 1;
                self.P.set_NZ(data);
                self.memory.write_memory(address, data);

                cpu_state.adr = address;
                cpu_state.op2 = data;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0x08 => { //PHP
                let address = 0x0100 | self.SP as u16;
                self.memory.write_memory(address, self.P.value | 0b0011_0000); //push brk and ignored as 1
                let r = self.SP.overflowing_sub(1);
                self.SP = r.0;

                cpu_state.adr = address;
                cpu_state.SP = self.SP;
                self.add_trace(cpu_state);
            }

            0x09 => { //ORA Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                self.A |= data;
                self.P.set_NZ(self.A);

                cpu_state.A = self.A;
                cpu_state.P = self.P;
                cpu_state.op1 = data;
                self.add_trace(cpu_state);
            }

            0x0a => { //ASL Accumulator
                let mut data = self.A;
                self.P.set_C(data & 0b1000_0000 != 0);
                data = data << 1;
                self.P.set_NZ(data);
                self.A = data;

                cpu_state.A = data;
                self.add_trace(cpu_state);
            }

            0x0d => { //ORA Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                self.A |= data;
                self.P.set_NZ(self.A);

                cpu_state.A = self.A;
                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x0e => { //ASL Absolute
                let address = self.get_address(AdressingType::Absolute);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                self.P.set_C(data & 0b1000_0000 != 0);
                data = data << 1;
                self.P.set_NZ(data);
                self.memory.write_memory(address, data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x10 => { //BPL
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;

                if !self.P.get_N(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    self.PC = r.0 as u16;
                    cpu_state.adr = self.PC;
                }

                cpu_state.op1 = data as u8;
                self.add_trace(cpu_state);
            }

            0x11 => { //ORA AbsoluteX
                let address = self.get_address(AdressingType::IndirectY);
                let data = self.memory.read_memory(address);
                self.A |= data;
                self.P.set_NZ(self.A);


                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x15 => { //ORA ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let data = self.memory.read_memory(address);
                self.A |= data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x16 => { //ASL ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                self.P.set_C(data & 0b1000_0000 != 0);
                data = data << 1;
                self.P.set_NZ(data);
                self.memory.write_memory(address, data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x18 => { //CLC
                self.P.set_C(false);

                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0x19 => { //ORA AbsoluteY
                let address = self.get_address(AdressingType::AbsoluteY);
                let data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                self.A |= data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x1d => { //ORA AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let data = self.memory.read_memory(address);
                self.A |= data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.adr = address;
                cpu_state.A = self.A;
                cpu_state.op1 = data;
                self.add_trace(cpu_state);
            }

            0x1e => { //ASL AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                self.P.set_C(data & 0b1000_0000 != 0);
                data = data << 1;
                self.P.set_NZ(data);
                self.memory.write_memory(address, data);

                cpu_state.P = self.P;
                cpu_state.adr = address;
                cpu_state.op2 = data;
                self.add_trace(cpu_state);
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

                cpu_state.P = self.P;
                cpu_state.adr = address;
                cpu_state.SP = self.SP;
                self.add_trace(cpu_state);
                self.PC = address;
            }

            0x21 => { //AND IndirectX
                let address = self.get_address(AdressingType::IndirectX);
                let data = self.memory.read_memory(address);
                self.A = self.A & data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.adr = address;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                self.add_trace(cpu_state);
            }

            0x24 => { //BIT ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                let res = data & self.A;
                self.P.set_Z(res == 0);
                self.P.value = (self.P.value & 0b0011_1111) | (data & 0b1100_0000);

                cpu_state.P = self.P;
                cpu_state.adr = address;
                cpu_state.op1 = data;
                self.add_trace(cpu_state);
            }

            0x25 => { //AND ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                self.A = self.A & data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
            }

            0x26 => { //ROL ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                let c = self.A & 0b1000_0000 != 0;
                data = data << 1 | (self.P.get_C() as u8);
                self.P.set_C(c);
                self.P.set_NZ(data);
                self.memory.write_memory(address, data);

                cpu_state.P = self.P;
                cpu_state.adr = address;
                cpu_state.op2 = data;
                self.add_trace(cpu_state);
            }

            0x28 => { //PLP
                let r = self.SP.overflowing_add(1);
                self.SP = r.0;
                let address = 0x0100 | self.SP as u16;
                let data = self.memory.read_memory(address);
                self.P.value = data & 0b1100_1111; //ignore B and bit 5

                cpu_state.P = self.P;
                cpu_state.adr = address;
                cpu_state.SP = self.SP;
                self.add_trace(cpu_state);
            }

            0x29 => { //AND Immidiate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                self.A = self.A & data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                self.add_trace(cpu_state);
            }

            0x2a => { //ROL Accumulator
                let mut data = self.A;
                data = data << 1 | (self.P.get_C() as u8);
                self.P.set_C(self.A & 0b1000_0000 != 0);
                self.P.set_NZ(data);
                self.A = data;

                cpu_state.P = self.P;
                cpu_state.A = self.A;
                self.add_trace(cpu_state);
            }

            0x2c => { //BIT Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                let res = data & self.A;
                self.P.set_Z(res == 0);
                self.P.value = (self.P.value & 0b0011_1111) | (data & 0b1100_0000);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x2d => { //AND Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                self.A = self.A & data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x2e => { //ROL Absolute
                let address = self.get_address(AdressingType::Absolute);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                data = data << 1 | (self.P.get_C() as u8);
                self.P.set_C(self.A & 0b1000_0000 != 0);
                self.P.set_NZ(data);
                self.memory.write_memory(address, data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x30 => { //BMI Relative
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;

                if self.P.get_N(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                                        self.PC = r.0 as u16;
                    cpu_state.adr = self.PC;
                }

                cpu_state.op1 = data as u8;
                self.add_trace(cpu_state);
            }

            0x31 => { //AND IndirectY
                let address = self.get_address(AdressingType::IndirectY);
                let data = self.memory.read_memory(address);
                self.A = self.A & data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x35 => { //AND ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let data = self.memory.read_memory(address);
                self.A = self.A & data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x36 => { //ROL ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                data = data << 1 | (self.P.get_C() as u8);
                self.P.set_C(self.A & 0b1000_0000 != 0);
                self.P.set_NZ(data);
                self.memory.write_memory(address, data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x38 => { //SEC
                self.P.set_C(true);

                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0x39 => { //AND AbsoluteY
                let address = self.get_address(AdressingType::AbsoluteY);
                let data = self.memory.read_memory(address);
                self.A = self.A & data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x3d => { //AND AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let data = self.memory.read_memory(address);
                self.A = self.A & data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x3e => { //ROL AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                data = data << 1 | (self.P.get_C() as u8);
                self.P.set_C(self.A & 0b1000_0000 != 0);
                self.P.set_NZ(data);
                self.memory.write_memory(address, data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x40 => { //RTI
                self.SP = self.SP.overflowing_add(1).0;
                let address = 0x0100 | self.SP as u16;
                let data = self.memory.read_memory(address);
                self.SP = self.SP.overflowing_add(1).0;
                self.P.value = (data & 0b1101_1111) | 0b0001_0000; //ignore bit 5 set B
                let sp = 0x0100 | self.SP as u16;
                let addr = self.memory.read_memory_word(sp);
                self.SP = self.SP.overflowing_add(1).0;
                self.PC = addr;

                cpu_state.SP = self.SP;
                cpu_state.adr = addr;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0x41 => { //EOR IndirectX
                let address = self.get_address(AdressingType::IndirectX);
                let data = self.memory.read_memory(address);
                self.A = self.A ^ data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x45 => { //EOR ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                self.A = self.A ^ data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x46 => { //LSR ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                self.P.set_C(data & 0b0000_0001 != 0);
                data = data >> 1;
                self.memory.write_memory(address, data);
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x48 => { //PHA
                let address = 0x0100 | self.SP as u16;
                self.memory.write_memory(address, self.A);
                let r = self.SP.overflowing_sub(1);
                self.SP = r.0;

                cpu_state.adr = address;
                cpu_state.SP = self.SP;
                self.add_trace(cpu_state);

            }

            0x49 => { //EOR Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                self.A = self.A ^ data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                self.add_trace(cpu_state);
            }

            0x4a => { //LSR Accumulator
                let mut data = self.A;
                self.P.set_C(data & 0b0000_0001 != 0);
                data = data >> 1;
                self.A = data;
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.A = self.A;
                self.add_trace(cpu_state);
            }

            0x4c => { //JMP Absolute
                let address = self.get_address(AdressingType::Absolute);
                self.PC = address;

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x4d => { //EOR Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                self.A = self.A ^ data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x4e => { //LSR Absolute
                let address = self.get_address(AdressingType::Absolute);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                self.P.set_C(data & 0b0000_0001 != 0);
                data = data >> 1;
                self.memory.write_memory(address, data);
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x50 => { //BVC
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;
                if !self.P.get_V(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    self.PC = r.0 as u16;
                    cpu_state.adr = self.PC;
                }

                cpu_state.op1 = data as u8;
                self.add_trace(cpu_state);
            }

            0x51 => { //EOR IndirectY
                let address = self.get_address(AdressingType::IndirectY);
                let data = self.memory.read_memory(address);
                self.A = self.A ^ data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x55 => { //EOR ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let data = self.memory.read_memory(address);
                self.A = self.A ^ data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x56 => { //LSR ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                self.P.set_C(data & 0b0000_0001 != 0);
                data = data >> 1;
                self.memory.write_memory(address, data);
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x58 => { //CLI
                self.P.set_I(false);

                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0x59 => { //EOR AbsoluteY
                let address = self.get_address(AdressingType::AbsoluteY);
                let data = self.memory.read_memory(address);
                self.A = self.A ^ data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x5d => { //EOR AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let data = self.memory.read_memory(address);
                self.A = self.A ^ data;
                self.P.set_NZ(self.A);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x5e => { //LSR AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                self.P.set_C(data & 0b0000_0001 != 0);
                data = data >> 1;
                self.memory.write_memory(address, data);
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x60 => { //RTS
                self.SP = self.SP.overflowing_add(1).0;
                let sp = 0x0100 | self.SP as u16;
                let addr = self.memory.read_memory_word(sp).overflowing_add(1).0;
                self.SP = self.SP.overflowing_add(1).0;
                self.PC = addr;

                cpu_state.P = self.P;
                cpu_state.SP = self.SP;
                cpu_state.adr = addr;
                self.add_trace(cpu_state);
            }

            0x61 => { //ADC IndirectX
                let address = self.get_address(AdressingType::IndirectX);
                let data = self.memory.read_memory(address);
                cpu_state.adr = address;
                self.adc(cpu_state, data);
            }

            0x65 => { //ADC ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                cpu_state.adr = address;
                self.adc(cpu_state, data);
            }

            0x66 => { //ROR ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                let c = data & 0b0000_0001 != 0;
                data = data >> 1 | ((self.P.get_C() as u8) << 7);
                self.P.set_C(c);
                self.P.set_NZ(data);
                self.memory.write_memory(address, data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x68 => {  //PLA
                let r = self.SP.overflowing_add(1);
                self.SP = r.0;
                let address = 0x0100 | self.SP as u16;
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.A = self.A;
                cpu_state.adr = address;
                cpu_state.SP = self.SP;
                self.add_trace(cpu_state);
            }

            0x69 => { //ADC Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                self.adc(cpu_state, data);
            }

            0x6a => { //ROR Accumulator
                let mut data = self.A;
                data = data >> 1 | ((self.P.get_C() as u8) << 7);
                self.P.set_C(self.A & 0b0000_0001 != 0);
                self.P.set_NZ(data);
                self.A = data;

                cpu_state.P = self.P;
                cpu_state.A = self.A;
                self.add_trace(cpu_state);
            }

            0x6c => { //JMP Indirect
                let address = self.get_address(AdressingType::Indirect);
                self.PC = address;

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x6d => { //ADC Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                cpu_state.adr = address;
                self.adc(cpu_state, data);
            }

            0x6e => { //ROR Absolute
                let address = self.get_address(AdressingType::Absolute);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                let c = data & 0b0000_0001 != 0;
                data = data >> 1 | ((self.P.get_C() as u8) << 7);
                self.P.set_C(c);
                self.P.set_NZ(data);
                self.memory.write_memory(address, data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x70 => { //BVS
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;

                if self.P.get_V(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    self.PC = r.0 as u16;
                    cpu_state.adr = self.PC;
                }

                cpu_state.op1 = data as u8;
                self.add_trace(cpu_state);
            }

            0x71 => { //ADC IndirectY
                let address = self.get_address(AdressingType::IndirectY);
                let data = self.memory.read_memory(address);
                cpu_state.adr = address;
                self.adc(cpu_state, data);
            }

            0x75 => { //ADC ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let data = self.memory.read_memory(address);
                cpu_state.adr = address;
                self.adc(cpu_state, data);
            }

            0x76 => { //ROR ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                let c = data & 0b0000_0001 != 0;
                data = data >> 1 | ((self.P.get_C() as u8) << 7);
                self.P.set_C(c);
                self.P.set_NZ(data);
                self.memory.write_memory(address, data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x78 => { //SEI
                self.P.set_I(true);

                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0x79 => { //ADC AbsoluteY
                let address = self.get_address(AdressingType::AbsoluteY);
                let data = self.memory.read_memory(address);
                cpu_state.adr = address;
                self.adc(cpu_state, data);
            }

            0x7d => { //ADC AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let data = self.memory.read_memory(address);
                cpu_state.adr = address;
                self.adc(cpu_state, data);
            }

            0x7e => { //ROR AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                let c = data & 0b0000_0001 != 0;
                data = data >> 1 | ((self.P.get_C() as u8) << 7);
                self.P.set_C(c);
                self.P.set_NZ(data);
                self.memory.write_memory(address, data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x81 => { //STA IndirectX
                let address = self.get_address(AdressingType::IndirectX);
                self.memory.write_memory(address, self.A);

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x84 => { //STY ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                self.memory.write_memory(address, self.Y);

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x85 => { //STA ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                self.memory.write_memory(address, self.A);

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x86 => { //STX ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                self.memory.write_memory(address, self.X);

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x88 => { //DEY
                self.Y = self.Y.overflowing_sub(1).0;
                self.P.set_NZ(self.Y);

                cpu_state.P = self.P;
                cpu_state.Y = self.Y;
                self.add_trace(cpu_state);
            }

            0x8a => { //TXA
                self.A = self.X;
                self.P.set_NZ(self.A);

                cpu_state.A = self.A;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0x8c => { //STY Absolute
                let address = self.get_address(AdressingType::Absolute);
                self.memory.write_memory(address, self.Y);

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x8d => { //STA Absolute
                let address = self.get_address(AdressingType::Absolute);
                self.memory.write_memory(address, self.A);

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x8e => { //STX Absolute
                let address = self.get_address(AdressingType::Absolute);
                self.memory.write_memory(address, self.X);

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x90 => { //BCC
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;

                if !self.P.get_C(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    self.PC = r.0 as u16;
                    cpu_state.adr = self.PC;
                }

                cpu_state.op1 = data as u8;
                self.add_trace(cpu_state);
            }

            0x91 => { //STA IndirectY
                let address = self.get_address(AdressingType::IndirectY);
                self.memory.write_memory(address, self.A);

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x94 => { //STY ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                self.memory.write_memory(address, self.Y);

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x95 => { //STA ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                self.memory.write_memory(address, self.A);

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x96 => { //STX ZeroPageY
                let address = self.get_address(AdressingType::ZeroPageY);
                self.memory.write_memory(address, self.X);

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x98 => { //TYA
                self.A = self.Y;
                self.P.set_NZ(self.A);

                cpu_state.A = self.A;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0x99 => { //STA AbsoluteY
                let address = self.get_address(AdressingType::AbsoluteY);
                self.memory.write_memory(address, self.A);

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0x9a => {//TXS
                self.SP = self.X;

                cpu_state.SP = self.SP;
                self.add_trace(cpu_state);
            }

            0x9d => { //STA AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                self.memory.write_memory(address, self.A);

                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xa0 => { //LDY Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                self.Y = data;
                self.P.set_NZ(data);

                cpu_state.Y = self.Y;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0xa1 => { //LDA IndirectX
                let address = self.get_address(AdressingType::IndirectX);
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);

                cpu_state.A = self.A;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xa2 => { //LDX Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                self.X = data;
                self.P.set_NZ(data);

                cpu_state.X = self.X;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0xa4 => { //LDY ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                self.Y = data;
                self.P.set_NZ(data);

                cpu_state.Y = self.Y;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xa5 => { //LDA ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);

                cpu_state.A = self.A;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xa6 => { //LDX ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                self.X = data;
                self.P.set_NZ(data);

                cpu_state.X = self.X;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xa8 => { //TAY
                self.Y = self.A;
                self.P.set_NZ(self.Y);

                cpu_state.Y = self.Y;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0xa9 => { //LDA Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                self.A = data;
                self.P.set_NZ(data);

                cpu_state.A = self.A;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0xaa => { //TAX
                self.X = self.A;
                self.P.set_NZ(self.X);

                cpu_state.X = self.X;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0xac => { //LDY Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                self.Y = data;
                self.P.set_NZ(data);

                cpu_state.Y = self.Y;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xad => { //LDA Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);

                cpu_state.A = self.A;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xae => { //LDX Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                self.X = data;
                self.P.set_NZ(data);

                cpu_state.X = self.X;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xb0 => {
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;

                if self.P.get_C(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    self.PC = r.0 as u16;
                    cpu_state.adr = self.PC;
                }

                cpu_state.op1 = data as u8;
                self.add_trace(cpu_state);
            }

            0xb1 => { //LDA IndirectY
                let address = self.get_address(AdressingType::IndirectY);
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);

                cpu_state.A = self.A;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
                //elf.add_trace(format!("LDA IndirectY ({:#04x})\n", data));
            }

            0xb4 => { //LDY ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let data = self.memory.read_memory(address);
                self.Y = data;
                self.P.set_NZ(data);

                cpu_state.Y = self.Y;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xb5 => { //LDA ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);

                cpu_state.A = self.A;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xb6 => { //LDX ZeroPageY
                let address = self.get_address(AdressingType::ZeroPageY);
                let data = self.memory.read_memory(address);
                self.X = data;
                self.P.set_NZ(data);

                cpu_state.X = self.X;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xb8 => { //CLV
                self.P.set_V(false);

                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0xb9 => { //LDA AbsoluteY
                let address = self.get_address(AdressingType::AbsoluteY);
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);

                cpu_state.Y = self.Y;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xba => { //TSX
                self.X = self.SP;
                self.P.set_NZ(self.X);


            }

            0xbc => { //LDY AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let data = self.memory.read_memory(address);
                self.Y = data;
                self.P.set_NZ(data);

                cpu_state.Y = self.Y;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xbd => { //LDA AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let data = self.memory.read_memory(address);
                self.A = data;
                self.P.set_NZ(data);

                cpu_state.A = self.A;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xbe => { //LDX AbsoluteY
                let address = self.get_address(AdressingType::AbsoluteY);
                let data = self.memory.read_memory(address);
                self.X = data;
                self.P.set_NZ(data);

                cpu_state.X = self.X;
                cpu_state.P = self.P;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xc0 => { //CPY Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                let r = self.Y.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                self.add_trace(cpu_state);
            }

            0xc1 => { //CMP IndirectX
                let address = self.get_address(AdressingType::IndirectX);
                let data = self.memory.read_memory(address);
                let r = self.A.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xc4 => { //CPY ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                let r = self.Y.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xc5 => { //CMP ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                let r = self.A.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xc6 => { //DEC ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                data = data.overflowing_sub(1).0;
                self.memory.write_memory(address, data);
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xc8 => { //INY
                self.Y = self.Y.overflowing_add(1).0;
                self.P.set_NZ(self.Y);

                cpu_state.Y = self.Y;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0xc9 => { //CMP Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                let r = self.A.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                self.add_trace(cpu_state);
            }

            0xca => { //DEX
                self.X = self.X.overflowing_sub(1).0;
                self.P.set_NZ(self.X);

                cpu_state.X = self.X;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0xcc => { //CPY Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                let r = self.Y.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xcd => { //CMP Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                let r = self.A.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xce => { //DEC Absolute
                let address = self.get_address(AdressingType::Absolute);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                data = data.overflowing_sub(1).0;
                self.memory.write_memory(address, data);
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xd0 => { //BNE Relative
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;

                if !self.P.get_Z(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    self.PC = r.0 as u16;
                    cpu_state.adr = self.PC;
                }

                cpu_state.op1 = data as u8;
                self.add_trace(cpu_state);
            }

            0xd1 => { //CMP IndirectY
                let address = self.get_address(AdressingType::IndirectY);
                let data = self.memory.read_memory(address);
                let r = self.A.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xd5 => { //CMP ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let data = self.memory.read_memory(address);
                let r = self.A.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xd6 => { //DEC ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                data = data.overflowing_sub(1).0;
                self.memory.write_memory(address, data);
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xd8 => { //CLD Clear Decimal Mode
                self.P.set_D(false);

                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0xd9 => { //CMP AbsoluteY
                let address = self.get_address(AdressingType::AbsoluteY);
                let data = self.memory.read_memory(address);
                let r = self.A.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xdd => { //CMP AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let data = self.memory.read_memory(address);
                let r = self.A.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xde => { //DEC AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                data = data.overflowing_sub(1).0;
                self.memory.write_memory(address, data);
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xe0 => { //CPX Immediate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                let r = self.X.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                self.add_trace(cpu_state);
            }

            0xe1 => { //SBC IndirectX
                let address = self.get_address(AdressingType::IndirectX);
                let data = self.memory.read_memory(address);
                cpu_state.adr = address;
                self.sbc(cpu_state, data);
            }

            0xe4 => { //CPX ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                let r = self.X.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xe5 => { //SBC ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let data = self.memory.read_memory(address);
                cpu_state.adr = address;
                self.sbc(cpu_state, data);
            }

            0xe6 => { //INC ZeroPage
                let address = self.get_address(AdressingType::ZeroPage);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                data = data.overflowing_add(1).0;
                self.memory.write_memory(address, data);
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xe8 => { //INX
                let r = self.X.overflowing_add(1);
                self.X = r.0;
                self.P.set_NZ(r.0);

                cpu_state.X = self.X;
                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0xe9 => { //SBC Immidiate
                let data = self.memory.read_memory(self.PC);
                self.PC += 1;
                self.sbc(cpu_state, data);
            }

            0xea => { //NOP
                self.add_trace(cpu_state);
            }

            0xec => { //CPX Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                let r = self.X.overflowing_sub(data);
                self.P.set_NZ(r.0);
                self.P.set_C(!r.1);

                cpu_state.P = self.P;
                cpu_state.op1 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xed => { //SBC Absolute
                let address = self.get_address(AdressingType::Absolute);
                let data = self.memory.read_memory(address);
                cpu_state.adr = address;
                self.sbc(cpu_state, data);
            }

            0xee => { //INC Absolute
                let address = self.get_address(AdressingType::Absolute);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                data = data.overflowing_add(1).0;
                self.memory.write_memory(address, data);
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xf0 => { //BEQ Relative
                let data = self.memory.read_memory(self.PC) as i8;
                self.PC += 1;

                if self.P.get_Z(){
                    let r = (self.PC as i16).overflowing_add(data as i16);
                    self.PC = r.0 as u16;
                    cpu_state.adr = self.PC;
                }

                cpu_state.op1 = data as u8;
                self.add_trace(cpu_state);
            }

            0xf1 => { //SBC IndirectY
                let address = self.get_address(AdressingType::IndirectY);
                let data = self.memory.read_memory(address);
                cpu_state.adr = address;
                self.sbc(cpu_state, data);
            }

            0xf5 => { //SBC ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let data = self.memory.read_memory(address);

                cpu_state.adr = address;
                self.sbc(cpu_state, data);
            }

            0xf6 => { //INC ZeroPageX
                let address = self.get_address(AdressingType::ZeroPageX);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                data = data.overflowing_add(1).0;
                self.memory.write_memory(address, data);
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            0xf8 => { //SED
                self.P.set_D(true);

                cpu_state.P = self.P;
                self.add_trace(cpu_state);
            }

            0xf9 => { //SBC AbsoluteY
                let address = self.get_address(AdressingType::AbsoluteY);
                let data = self.memory.read_memory(address);
                cpu_state.adr = address;
                self.sbc(cpu_state, data);
            }

            0xfd => { //SBC AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let data = self.memory.read_memory(address);
                cpu_state.adr = address;
                self.sbc(cpu_state, data);
            }

            0xfe => { //INC AbsoluteX
                let address = self.get_address(AdressingType::AbsoluteX);
                let mut data = self.memory.read_memory(address);
                cpu_state.op1 = data;
                data = data.overflowing_add(1).0;
                self.memory.write_memory(address, data);
                self.P.set_NZ(data);

                cpu_state.P = self.P;
                cpu_state.op2 = data;
                cpu_state.adr = address;
                self.add_trace(cpu_state);
            }

            _  => {
                let pc = cpu_state.PC;
                self.add_trace(cpu_state);
                return Err(CpuError::new(&format!("Unknown instruction: INS={:#04x}", ins), pc));
            }
        }

        if self.PC == self.prev_PC{
            return Err(CpuError::new(&format!("LOOP Detected"), self.PC));
        }
        self.prev_PC = self.PC;

        Ok(())
    }

    pub fn interrupt(&mut self, int: InterruptType){
        if self.P.get_I() && int == InterruptType::INT{
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

        self.PC = address;
        self.P.set_I(true); //Disable Interupts
    }
}

#[cfg(test)]
mod tests{
    use crate::c64::cpu6502::memory::{Memory,Memory6502};
    use crate::c64::cpu6502::CPU6502;
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
            match cpu.run_single() {
                Ok(_) => {},
                Err(e) => {
                    cpu.show_cpu_debug();
                    println!("Error: {}", e);
                    break;
                }
            };
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
            match cpu.run_single() {
                Ok(_) => {},
                Err(e) => {
                    cpu.show_cpu_debug();
                    println!("Error: {}", e);
                    break;
                }
            };
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
            match cpu.run_single() {
                Ok(_) => {},
                Err(e) => {
                    cpu.show_cpu_debug();
                    println!("Error: {}", e);
                    break;
                }
            };
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
            match cpu.run_single() {
                Ok(_) => {},
                Err(e) => {
                    cpu.show_cpu_debug();
                    println!("Error: {}", e);
                    break;
                }
            };
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
            match cpu.run_single() {
                Ok(_) => {},
                Err(e) => {
                    cpu.show_cpu_debug();
                    println!("Error: {}", e);
                    break;
                }
            };
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
            match cpu.run_single() {
                Ok(_) => {},
                Err(e) => {
                    cpu.show_cpu_debug();
                    println!("Error: {}", e);
                    break;
                }
            };
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
            match cpu.run_single() {
                Ok(_) => {},
                Err(e) => {
                    cpu.show_cpu_debug();
                    println!("Error: {}", e);
                    break;
                }
            };
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
            match cpu.run_single() {
                Ok(_) => {},
                Err(e) => {
                    cpu.show_cpu_debug();
                    println!("Error: {}", e);
                    break;
                }
            };
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
            match cpu.run_single() {
                Ok(_) => {},
                Err(e) => {
                    cpu.show_cpu_debug();
                    println!("Error: {}", e);
                    break;
                }
            };
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
            match cpu.run_single() {
                Ok(_) => {},
                Err(e) => {
                    cpu.show_cpu_debug();
                    println!("Error: {}", e);
                    break;
                }
            };
        }

        assert_eq!(cpu.A, 0x00);
        assert_eq!(cpu.X, 0x05);
        assert_eq!(cpu.Y, 0x00);
        assert!(!cpu.P.get_N());
        assert!(cpu.P.get_C());
        assert!(cpu.P.get_Z());
    }

    #[test]
    fn test_all() -> Result<(), crate::c64::cpu6502::CpuError>{
        let mem = Memory::from_file("./tests/6502_functional_test.bin").unwrap();
        let mut cpu = CPU6502::new(mem);
        cpu.reset_at(0x0400);
        cpu.enable_trace(32);

        let mut cnt = 0;

        loop{
            cnt += 1;
            match cpu.run_single() {
                Ok(_) => {},
                Err(e) => {
                    if e.pc == 0x3469{ //This test program loops here on success
                        return Ok(());
                    }

                    println!("Error run {} instructions", cnt);
                    cpu.show_cpu_debug();
                    return Err(e);
                    //assert!(false);
                }
            };
        }
    }
}
