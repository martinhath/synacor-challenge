#[allow(dead_code)]

const MAX_NUM: u16 = 32768;
const MAX_MEM: usize = 32768;

use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::fmt;


// Only used for parsing input
#[derive(PartialEq, Eq, Clone, Copy)]
enum Unit {
    Register(u16),
    Number(u16),
    Uninitialized,
}
use Unit::*;

impl Unit {

    fn value(&self) -> u16 {
        match *self {
            Register(r) => {r + MAX_NUM}
            Number(n)   => {n}
            Uninitialized => {panic!("value() on uninitialized.")}
        }
    }

    fn from_u16(u: u16) -> Option<Unit> {
        if u > 32767 {
            if u > 32775 {
                None
            } else {
                Some(Register(u - 32768))
            }
        } else {
            Some(Number(u))
        }
    }

    fn from_bytes(lower: u8, upper: u8) -> Option<Unit> {
        let res: u16 = lower as u16 + ((upper as u16) << 8);
        if res > 32767 {
            if res > 32775 {
                // invalid
                println!("[from_bytes]: illegal number: {} {} (res = {})", lower, upper, res);
                None
            } else {
                Some(Register(res - 32768))
            }
        } else {
            Some(Number(res))
        }
    }
}

impl Default for Unit {
    fn default() -> Unit { Uninitialized }
}

impl std::fmt::Debug for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Register(r)   => {write!(f, "r{}", r)},
            Number(n)     => {write!(f, "{}", n)},
            Uninitialized => {write!(f, "<_>")}

        }
    }
}


#[allow(dead_code)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum InstructionType {
    Halt,
    Set,
    Push,
    Pop,
    Eq,
    Gt,
    Jmp,
    Jt,
    Jf,
    Add,
    Mult,
    Mod,
    And,
    Or,
    Not,
    Rmem,
    Wmem,
    Call,
    Ret,
    Out,
    In,
    Noop
}

#[allow(dead_code)]
impl InstructionType {
    fn from_u16(u: u16) -> Option<InstructionType> {
        match u {
            0  => Some(InstructionType::Halt),
            1  => Some(InstructionType::Set),
            2  => Some(InstructionType::Push),
            3  => Some(InstructionType::Pop),
            4  => Some(InstructionType::Eq),
            5  => Some(InstructionType::Gt),
            6  => Some(InstructionType::Jmp),
            7  => Some(InstructionType::Jt),
            8  => Some(InstructionType::Jf),
            9  => Some(InstructionType::Add),
            10 => Some(InstructionType::Mult),
            11 => Some(InstructionType::Mod),
            12 => Some(InstructionType::And),
            13 => Some(InstructionType::Or),
            14 => Some(InstructionType::Not),
            15 => Some(InstructionType::Rmem),
            16 => Some(InstructionType::Wmem),
            17 => Some(InstructionType::Call),
            18 => Some(InstructionType::Ret),
            19 => Some(InstructionType::Out),
            20 => Some(InstructionType::In),
            21 => Some(InstructionType::Noop),
            _  => None

        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct Instruction {
    itype: InstructionType,
    // Hot argument handing
    a: Unit,
    b: Unit,
    c: Unit, // what is memory??
    n_args: usize,
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // @TODO: look into macros
        match self.n_args {
            0 => { write!(f, "{:?}", self.itype) },
            1 => { write!(f, "{:?} {:?}", self.itype, self.a) },
            2 => { write!(f, "{:?} {:?} {:?}", self.itype, self.a, self.b) },
            3 => { write!(f, "{:?} {:?} {:?} {:?}", self.itype, self.a, self.b, self.c) },
            _ => { write!(f, "ERROR ERROR ERROR") },
        }
    }
}

#[allow(dead_code)]
impl Instruction {

    fn num_args(n: u32) -> usize {
        if [0, 18, 21].contains(&n) {
            0
        } else if [2, 3, 6, 17, 19, 20].contains(&n) {
            1
        } else if [1, 7, 8, 14, 15, 16].contains(&n) {
            2
        } else if [4, 5, 9, 10, 11, 12, 13].contains(&n) {
            3
        } else {
            panic!("illegal register number: {}", n);
        }
    }

    fn get(u: Unit) -> Option<Instruction> {
        if let Number(n) = u {
            let t = InstructionType::from_u16(n);
            if t.is_none() {
                return None;
            }
            Some(Instruction {
                itype: t.unwrap(),
                a: Default::default(),
                b: Default::default(),
                c: Default::default(),
                n_args: 0,
            })
        } else {
            None
        }
    }

    fn next_instruction(data: &[Unit]) -> Option<Instruction> {
        let instruction = Instruction::get(data[0]);

        if instruction == None {
            return None;
        }
        let mut instruction = instruction.unwrap();
        // @TODO: bounds checks
        match Instruction::num_args(instruction.itype as u32) {
            1 => {instruction.a = data[1];
                  instruction.n_args = 1;}
            2 => {instruction.a = data[1];
                  instruction.b = data[2];
                  instruction.n_args = 2;}
            3 => {instruction.a = data[1];
                  instruction.b = data[2];
                  instruction.c = data[3];
                  instruction.n_args = 3;}
            _ => {}
        };
        Some(instruction)
    }
}

#[allow(dead_code)]
struct SystemState {
    registers: [u16; 8],
    memory: [Unit; MAX_MEM],
    stack: Vec<Unit>,
    pc: usize,
    halt: bool,
    jumped: bool,
    input_string: String,
    debug: bool,
}

impl SystemState {

    fn value(&self, unit: Unit) -> u16 {
        match unit {
            Register(r)   => self.registers[r as usize],
            Number(n)     => n,
            Uninitialized => panic!("Try to access unitilialized value"),
        }
    }

    fn value_mut(&mut self, unit: Unit) -> &mut u16 {
        match unit {
            Register(r)   => self.registers.get_mut(r as usize).unwrap(),
            Number(n)     => {
                let data = self.memory.get_mut(n as usize).unwrap();
                match data {
                    &mut Register(_)   => panic!("try to get value_mut on register value in memory"),
                    &mut Number(ref mut n)     => n,
                    &mut Uninitialized => panic!("Try to access unitilialized value"),
                }
            }
            Uninitialized => panic!("Try to access unitilialized value"),
        }
    }

    fn handle_command(&mut self, string: String) {
        if string.len() < 4 { return; }
        let mut split = string.split_whitespace();
        let command = split.next().unwrap();
        let r = split.next();

        match command {
            "/get" => {
                let reg = r.unwrap().parse::<usize>().expect("Error reading register");
                println!(">>> r{} = {}", reg, self.registers[reg])
            }
            "/set" => {
                let reg = r.unwrap().parse::<usize>().expect("Error reading register");
                let val = split.next().unwrap().parse::<u16>().expect("Error reading value");
                self.registers[reg] = val;
            }
            // "/sav" => {
            //     let fname = split.next().unwrap();
            //     let dump = serialize(&self);
            //     file = File::new(fname)
            //     file.write(dump);
            // }
            // "/lod" => {
            //     let fname = split.next().unwrap();
            //     let file = File::open(fname);
            //     let str = String::new();
            //     file.read_to_string(str);
            //     self = deserialize(str);
            // }
            "/dmp" => {
                println!("dumping whole memory to strerr (pc = {})", self.pc);
                let mut i = 0;
                let end = self.memory.len();
                while i < end  {
                    let instr = Instruction::next_instruction(&self.memory[i..]);
                    if let Some(instruction) = instr {
                        let _ = write!(std::io::stderr(), "{:>5}| {:?}\n", i, instruction);
                        i += instruction.n_args;
                    }
                    i += 1;
                }
            }
            _      => {}
        }
    }
}

#[allow(dead_code)]
fn run_instruction(s: &mut SystemState, instr: Instruction) -> &SystemState {
    let mut state = s;
    match instr.itype {
        InstructionType::Halt => {
            state.halt = true;
            return state;
        }
        InstructionType::Set => {
            let val = state.value(instr.b);
            let res = state.value_mut(instr.a);
            *res = val;
        }
        InstructionType::Push => {
            let val = state.value(instr.a);
            state.stack.push(Number(val));
        }
        InstructionType::Pop => {
            let top = state.stack.pop();
            if top.is_none() {
                state.halt = true;
                return state;
            }
            let res = state.value_mut(instr.a);
            *res = top.unwrap().value();
        }
        InstructionType::Eq => {
            let b = state.value(instr.b);
            let c = state.value(instr.c);
            let res = state.value_mut(instr.a);
            *res = if b == c { 1 } else { 0 };
        }
        InstructionType::Gt => {
            let b = state.value(instr.b);
            let c = state.value(instr.c);
            let res = state.value_mut(instr.a);
            *res = if b > c { 1 } else { 0 };
        }
        InstructionType::Jmp => {
            // yolo mode engage
            if let Number(pc) = instr.a {
                state.pc = pc as usize;
                state.jumped = true;
            }
        }
        InstructionType::Jt => {
            let n = state.value(instr.a);
            if n != 0 {
                let pc = state.value(instr.b) as usize;
                state.pc = pc;
                state.jumped = true;
            }
        }
        InstructionType::Jf => {
            let n = state.value(instr.a);
            if n == 0 {
                let pc = state.value(instr.b) as usize;
                state.pc = pc;
                state.jumped = true;
            }
        }
        InstructionType::Add => {
            let a = state.value(instr.b);
            let b = state.value(instr.c);
            let res = state.value_mut(instr.a);
            *res = (a + b) % MAX_NUM;
        }
        InstructionType::Mult => {
            // @TODO: Fix 'as u64' stuff ?
            let a = state.value(instr.b) as u64;
            let b = state.value(instr.c) as u64;
            let res = state.value_mut(instr.a);
            *res = ((a * b) % MAX_NUM as u64) as u16;
        }
        InstructionType::Mod => {
            let a = state.value(instr.b);
            let b = state.value(instr.c);
            let res = state.value_mut(instr.a);
            *res = a % b;
        }
        InstructionType::And => {
            let a = state.value(instr.b);
            let b = state.value(instr.c);
            let res = state.value_mut(instr.a);
            *res = a & b;
        }
        InstructionType::Or => {
            let a = state.value(instr.b);
            let b = state.value(instr.c);
            let res = state.value_mut(instr.a);
            *res = a | b;
        }
        InstructionType::Not => {
            let n = state.value(instr.b);
            let res = state.value_mut(instr.a);
            *res = !n % MAX_NUM;
        }
        InstructionType::Rmem => {
            let addr = state.value(instr.b) as usize;
            let data = state.memory[addr];
            let res = state.value_mut(instr.a);
            *res = data.value();
        }
        InstructionType::Wmem => {
            let addr = state.value(instr.a) as usize;
            let data = state.value(instr.b);
            state.memory[addr] = Unit::from_u16(data).unwrap();
        }
        InstructionType::Call => {
            let addr = state.value(instr.a) as usize;
            let unit = Unit::from_u16((state.pc + 1 + instr.n_args) as u16);
            if let Some(u) = unit {
                state.stack.push(u);
                state.pc = addr;
                state.jumped = true;
            } else {
                println!("PC was too high: {}! Shit design fix pls", state.pc);
            }
        }
        InstructionType::Ret => {
            let num = state.stack.pop();
            if num.is_none() {
                state.halt = true;
                return state;
            }
            let num = num.unwrap();
            if let Number(n) = num {
                state.pc = n as usize;
                state.jumped = true;
            }
        }
        InstructionType::Out => {
            let data = state.value(instr.a) as u8;
            print!("{}", data as char);
        }
        InstructionType::In => {
            let char;
            { // rust werks
                let ref mut is = state.input_string;
                if is.len() == 0 {
                    let mut buffer = String::new();
                    let mut result = std::io::stdin().read_line(&mut buffer);
                    while result.is_err() {
                        result = std::io::stdin().read_line(&mut buffer);
                    }
                    for char in buffer.chars().rev() {
                        is.push(char);
                    }
                }
                char = is.pop().unwrap() as u16;
            }
            let res = state.value_mut(instr.a);
            *res = char;
        }
        InstructionType::Noop => {
        }
    };
    state
}

#[allow(dead_code)]
fn run_file(s: SystemState, filename: String, reg7: u16) {
    let mut state = s;
    let file = File::open(filename.clone());
    if let Err(e) = file {
        panic!("Failed to open file '{}': {}", filename, e);
    }

    let mut input_buffer = Vec::new();
    if let Err(e) = file.unwrap().read_to_end(&mut input_buffer) {
        panic!("Failed to read file: {}", e);
    }
    let input_len = input_buffer.len();
    if input_len/2 > MAX_MEM {
        panic!("Not enough memory to store program: {}/{}", input_len, MAX_MEM);
    }

    // Copy program to memory
    let mut i = 0;
    let mut j = 0;
    while i < input_len { // off by one with i + 1?
        let b1 = input_buffer[i];
        let b2 = input_buffer[i + 1];
        state.memory[j] = match Unit::from_bytes(b1, b2) {
            Some(e) => e,
            None    => {
                panic!("Invalid bytes at offset {}: {} {}", i, b1, b2);
            }
        };
        j += 1;
        i += 2;
    }

    // Program loop.
    let pc_end = state.memory.len();

    while state.pc < pc_end {

        let instr = Instruction::next_instruction(&state.memory[state.pc..]);
        let mut n_args = 0;

        if let Some(instruction) = instr {
            n_args = instruction.n_args;

            run_instruction(&mut state, instruction);

            if state.debug {
                let _ = write!(std::io::stderr(), "{:>5}| {:?}\n", state.pc, instruction);
            }

            let is: String = state.input_string.chars().rev().collect();
            if is.starts_with("/") {
                state.handle_command(is);
                state.pc -= 1;
                state.input_string = String::new();
            } else if is.starts_with("se teleporter") {
                state.registers[7] = reg7;
            }

            if state.pc == 6050 {
                break;
            }

            if state.halt {
                break;
            }
        } // silent fail errors
        if state.jumped {
            state.jumped = false;
        } else {
            state.pc += 1 + n_args;
        }
    }
}


fn main() {
    let start_string = "doorway\nnorth\nnorth\nbridge\ncontinue\ndown\neast\ntake empty lantern\nwest\nwest\npassage\nladder\nwest\nsouth\nnorth\ntake can\nwest\nladder\ndarkness\nuse can\nuse lantern\ncontinue\nwest\nwest\nwest\nwest\nnorth\ntake red coin\nnorth\neast\ntake concave coin\ndown\ntake corroded coin\nup\nwest\nwest\ntake blue coin\nup\ntake shiny coin\ndown\neast\nuse blue coin\nuse red coin\nuse shiny coin\nuse concave coin\nuse corroded coin\nnorth\ntake teleporter\nuse teleporter\n";
    let s: String = start_string.chars().rev().collect();
    // Filename should be first argument
    let args: Vec<_> = env::args().skip(1).collect();
    let filename = args[0].clone();
    let reg7 = args[1].parse::<u16>().unwrap();

    let system_state = SystemState {
        registers: [0; 8],
        memory:    [Number(0); 32768],
        stack: Vec::new(),
        pc: 0,
        halt: false,
        jumped: false,
        input_string: s.clone(),
        debug: false,
    };
    run_file(system_state, filename, reg7);
}
