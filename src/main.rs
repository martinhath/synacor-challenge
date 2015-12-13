#[allow(dead_code)]

const MAX_NUM: u16 = 32768;
const MAX_MEM: usize = 32768;

use std::env;
use std::fs::File;
use std::io::Read;
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
}

impl SystemState {
    fn value(&self, unit: Unit) -> u16 {
        match unit {
            Register(r)   => self.registers[r as usize],
            Number(n)     => n,
            Uninitialized => panic!("Try to access unitilialized value"),
        }
    }
}

fn arithmetic_instr(f: Box<Fn(u16, u16) -> u16>, a: Unit, b: Unit) -> Option<u16> {
    if let (Number(x), Number(y)) = (a, b) {
        let res = f(x, y);
        Some(res)
    } else {
        None
    }
}

fn arithmetic_instr_mod(f: Box<Fn(u16, u16) -> u16>, a: Unit, b: Unit) -> Option<u16> {
    arithmetic_instr(f, a, b)
        .map(|n| n % MAX_NUM)
}

#[allow(dead_code)]
fn run_instruction(s: SystemState, instr: Instruction) -> SystemState {
    let mut state = s;
    match instr.itype {
        InstructionType::Halt => {
            state.halt = true;
            return state;
        }
        InstructionType::Set => {
            let reg = instr.a;
            if let Register(r) = reg {
                let num = instr.b;
                if let Number(n) = num {
                    state.registers[r as usize] = n;
                }
            }
        }
        InstructionType::Push => {
            state.stack.push(instr.a);
        }
        InstructionType::Pop => {
            println!("pop instr: a = {:?}", instr.a);
        }
        InstructionType::Eq => {
            if let Register(r) = instr.a {
                state.registers[r as usize] = if instr.b == instr.c {1} else {0};
            }
        }
        InstructionType::Gt => {
            if let Register(r) = instr.a {
                if let (Number(b), Number(c)) = (instr.b, instr.c) {
                    state.registers[r as usize] = if b > c {1} else {0};
                }
            }
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
            let res = a + b % MAX_NUM;

            match instr.a {
                Register(reg) => state.registers[reg as usize] = res,
                Number(n)     => state.memory[n as usize] = Unit::from_u16(res).unwrap(),
                Uninitialized => panic!("asd"),
            }
        }
        InstructionType::Mult => {
            if let Register(a) = instr.a {
                let f = Box::new(|a, b| {a * b});
                let res = arithmetic_instr_mod(f, instr.b, instr.c);
                if let Some(n) = res {
                    state.registers[a as usize] = n;
                }
            }
        }
        InstructionType::Mod => {
            if let Register(a) = instr.a {
                let f = Box::new(|a, b| {a % b});
                let res = arithmetic_instr(f, instr.b, instr.c);
                if let Some(n) = res {
                    state.registers[a as usize] = n;
                }
            }
        }
        InstructionType::And => {
            if let Register(a) = instr.a {
                let f = Box::new(|a, b| {a & b});
                let res = arithmetic_instr(f, instr.b, instr.c);
                if let Some(n) = res {
                    state.registers[a as usize] = n;
                }
            }
        }
        InstructionType::Or => {
            if let Register(a) = instr.a {
                let f = Box::new(|a, b| {a | b});
                let res = arithmetic_instr(f, instr.b, instr.c);
                if let Some(n) = res {
                    state.registers[a as usize] = n;
                }
            }
        }
        InstructionType::Not => {
            if let Register(a) = instr.a {
                if let Number(n) = instr.b {
                    state.registers[a as usize] = !n;
                }
            }
        }
        InstructionType::Rmem => {
            if let Register(a) = instr.a {
                if let Number(mem) = instr.b {
                    let data = state.memory[mem as usize];
                    state.registers[a as usize] = data.value();
                }
            }
        }
        InstructionType::Wmem => {
            if let Register(r) = instr.a {
                if let Number(val) = instr.b {
                    let addr = state.registers[r as usize] as usize;
                    let unit = Unit::from_u16(val);
                    if let Some(u) = unit {
                        state.memory[addr] = u;
                    }
                }
            }
        }
        InstructionType::Call => {
            if let Number(n) = instr.a {
                state.stack.push(Number(state.pc as u16));
                state.pc = n as usize;
                state.jumped = true;
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
            }
        }
        InstructionType::Out => {
            let c = instr.a;
            if let Number(n) = c {
                let c = n as u8;
                print!("{}", c as char);
            } else {
                panic!("error");
            }
        }
        InstructionType::In => {
            println!("instruction 'in'");
        }
        InstructionType::Noop => {
        }
    };
    state
}


#[allow(dead_code)]
fn run_file(s: SystemState, filename: String) {
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
            println!("pc = {} (op={:?})", state.pc, instruction);
            n_args = instruction.n_args;

            state = run_instruction(state, instruction);

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

    // Filename should be first argument
    for filename in env::args().skip(1).take(1) {
        let system_state = SystemState {
            registers: [0; 8],
            memory:    [Number(0); 32768],
            stack: Vec::new(),
            pc: 0,
            halt: false,
            jumped: false,
        };
        run_file(system_state, filename);
    }
}
