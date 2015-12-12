#[allow(dead_code)]

const MAX_MEM: usize = 32768;

use std::env;
use std::fs::File;
use std::io::Read;

// Only used for parsing input
#[derive(PartialEq, Eq, Clone, Debug, Copy)]
enum Unit {
    Register(u16),
    Number(u16),
    Uninitialized,
}

impl Default for Unit {
    fn default() -> Unit { Unit::Uninitialized }
}

fn bytes_to_unit(lower: u8, upper: u8) -> Option<Unit> {
    let res: u16 = lower as u16 + upper as u16;
    if res > 32767 {
        if res > 32775 {
            // invalid
            println!("[bytes_to_unit]: illegal number: {} {}", lower, upper);
            None
        } else {
            Some(Unit::Register(res - 32768))
        }
    } else {
        Some(Unit::Number(res))
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Instruction {
    itype: InstructionType,
    // Hot argument handing
    a: Unit,
    b: Unit,
    c: Unit, // what is memory??
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
        if let Unit::Number(n) = u {
            let t = InstructionType::from_u16(n);
            if t == None {
                return None;
            }
            Some(Instruction {
                itype: t.unwrap(),
                a: Default::default(),
                b: Default::default(),
                c: Default::default(),
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
            1 => {instruction.a = data[1];}
            2 => {instruction.a = data[1];
                  instruction.b = data[2];}
            3 => {instruction.a = data[1];
                  instruction.b = data[2];
                  instruction.c = data[3];}
            _ => {}
        };
        Some(instruction)
    }
}

#[allow(dead_code)]
struct SystemState {
    registers: [u16; 8],
    memory: [Unit; MAX_MEM],
    stack: Vec<u16>,
    halt: bool,
}

#[allow(dead_code)]
fn run_instruction(s: SystemState, instr: Instruction) -> SystemState {
    let mut state = s;

    match instr.itype {
        InstructionType::Halt => {
            state.halt = true;
            return state;
        }
        InstructionType::Out => {
            let c = instr.a;
            if let Unit::Number(n) = c {
                let c = n as u8;
                print!("{}", c as char);
            } else {
                panic!("error");
            }
            state
        }
        _ => {
            return state;
        }
    }
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
        state.memory[j] = match bytes_to_unit(b1, b2) {
            Some(e) => e,
            None    => {
                panic!("Invalid bytes at offset {}: {} {}", i, b1, b2);
            }
        };
        j += 1;
        i += 2;
    }

    // Program loop.
    let mut pc = 0;
    let pc_end = state.memory.len();
    while pc < pc_end {
        let instr = Instruction::next_instruction(&state.memory[pc..]);
        if let Some(instruction) = instr {
            // println!("{:?}", instruction);

            state = run_instruction(state, instruction);

            if state.halt {
                break;
            }
        } // silent fail errors
        pc += 1;
    }
}


fn main() {

    // Filename should be first argument
    for filename in env::args().skip(1).take(1) {
        let system_state = SystemState {
            registers: [0; 8],
            memory:    [Unit::Number(0); 32768],
            stack: Vec::new(),
            halt: false,
        };
        run_file(system_state, filename);
    }
}
