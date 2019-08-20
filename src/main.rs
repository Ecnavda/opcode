use std::fs;
use std::io;

#[allow(non_snake_case)]
#[derive(Debug)]
struct Registers {
    V0: u8, V1: u8, V2: u8, V3: u8, V4: u8, V5: u8, V6: u8, V7: u8,
    V8: u8, V9: u8, VA: u8, VB: u8, VC: u8, VD: u8, VE: u8, VF: u8,
    I: u16, PC: u16,
}

impl Registers {
    fn new() -> Registers {
        Registers {
            V0: 0, V1: 0, V2: 0, V3: 0, V4: 0, V5: 0, V6: 0, V7: 0,
            V8: 0, V9: 0, VA: 0, VB: 0, VC: 0, VD: 0, VE: 0, VF: 0,
            I: 0, PC: 0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum Target_Register {
    V0, V1, V2, V3, V4, V5, V6, V7,
    V8, V9, VA, VB, VC, VD, VE, VF,
    I, PC,
}

impl Target_Register {
    fn u8_to_register(value: u8) -> Target_Register {
        match value {
            0x0 => Target_Register::V0,
            0x1 => Target_Register::V1,
            0x2 => Target_Register::V2,
            0x3 => Target_Register::V3,
            0x4 => Target_Register::V4,
            0x5 => Target_Register::V5,
            0x6 => Target_Register::V6,
            0x7 => Target_Register::V7,
            0x8 => Target_Register::V8,
            0x9 => Target_Register::V9,
            0xA => Target_Register::VA,
            0xB => Target_Register::VB,
            0xC => Target_Register::VC,
            0xD => Target_Register::VD,
            0xE => Target_Register::VE,
            0xF => Target_Register::VF,
            _ => Target_Register::PC, // TODO: Handle values outside of 0-F
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum Instruction {
    // X, Y represent registers
    // N represents values
    NOP,
    _Call { address: u16 }, // 0NNN
    Display, // 00E0 - Clear Screen
    Return, // 00EE - Return from subroutine
    JUMP { address: u16 }, // 1NNN - Jump to
    Call { address: u16 }, // 2NNN - Call subroutine
    SKEQ { register: Target_Register, value: u8 }, // 3XNN - Skip next instruction if equal
    SKNEQ { register: Target_Register, value: u8 }, // 4XNN - Skip next instruction if not equal
    SKREQ { register1: Target_Register, register2: Target_Register }, // 5XY0 - Skip next instruction if X and Y registers are equal
    SET { register: Target_Register, value: u8 }, // 6XNN - Sets X to NN
    ADD { register: Target_Register, value: u8 }, // 7XNN - Adds NN to X, doesn't affect carry flag
    COPYR { register1: Target_Register, register2: Target_Register }, // 8XY0 - Copy Y to X
    OR { register1: Target_Register, register2: Target_Register }, // 8XY1 - Set X to X | Y (Bitwise OR)
    AND { register1: Target_Register, register2: Target_Register }, // 8XY2 - Set X to X & Y (Bitwise AND)
    XOR { register1: Target_Register, register2: Target_Register }, // 8XY3 - Set X to X ^ Y (Bitwise XOR)
    ADDR { register1: Target_Register, register2: Target_Register }, // 8XY4 - Add Y to X, affects carry flag
    SUBX { register1: Target_Register, register2: Target_Register }, // 8XY5 - Subtract Y from X in X, affects borrow flag
    SHFTR { register1: Target_Register, register2: Target_Register }, // 8XY6 - Stores LSB in flag register then shifts X to the right 1
    SUBY { register1: Target_Register, register2: Target_Register }, // 8XY7 - Subtract X from Y in X, affects borrow flag
    SHFTL { register1: Target_Register, register2: Target_Register }, // 8XYE - Stores MSB in flag register then shifts X to the left 1
    SKRNEQ { register1: Target_Register, register2: Target_Register }, // 9XY0 - Skip next instruction if X and Y registers are not equal
    SETI { value: u16 }, // ANNN - Set I register to NNN
    JMP0 { address: u16 }, // BNNN - Jump to NNN plus V0 register
    RAND { register: Target_Register, value: u8 }, // CXNN - Set X to random number & NN
    DRAW { register1: Target_Register, register2: Target_Register, height: u8 }, // DXYN - Draw sprite at coords X register, Y register, of N height. Width fixed at 8 pixels. Check documentation for this.
    SKKEQ { register: Target_Register }, // EX9E - Skip next instruction if key stored in X is pressed
    SKKNEQ { register: Target_Register }, // EXA1 - Skip next instruction if key stored in X isn't pressed
    SETXD { register: Target_Register }, // FX07 - Set X to value of delay timer
    STORE { register: Target_Register }, // FX0A - Store key press in X (Blocks until key press)
    SETD { register: Target_Register }, // FX15 - Set delay timer to X
    SETS { register: Target_Register }, // FX18 - Set sound timer to X
    ADDI { register: Target_Register }, // FX1E - Add X to I
    SPRITE { register: Target_Register }, // FX29 - Set I to address of X for character sprite (Chars 0-F in hex are represented by 4x5 font)
    BCD { register: Target_Register }, // FX33 - Binary-Coded Decimal. Check documentation for this.
    DUMP { register: Target_Register }, // FX55 - Dumps registers, starting from V0 to X, beginning at memory address in I
    LOAD { register: Target_Register }, // FX65 - Fills registers, starting from V0 to X, with values beginning at memory address in I
}

struct CPU {
    registers: Registers,
    memory: [u8; 4096],
    stack: Vec<u16>,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
impl CPU {
    fn load_rom(&mut self, rom: &String) -> Result<&str, io::Error> {
        match fs::read(rom.trim()) {
            Ok(x) => {
                for y in 0..x.len() {
                    self.memory[0x200 + y] = x[y];
                };
                self.registers.PC = 0x200; //Programs begin at this address
                Ok("ROM loaded successfully.")
            },
            Err(e) => Err(e),
        }
    }
    
    fn fetch_instruction(&mut self) -> u16 {
       let mut opcode: u16 = self.memory[self.registers.PC as usize] as u16;
       opcode = opcode << 8;
       self.registers.PC += 1;
       opcode = opcode | self.memory[self.registers.PC as usize] as u16;
       self.registers.PC += 1;
       opcode
    }

    fn new() -> CPU {
        CPU {
            registers: Registers::new(),
            memory: [0u8; 4096],
            stack: vec![0u16; 16],
        }
    }

    fn initialize(&mut self) {
        self.registers.V0 = self.registers.V0 ^ self.registers.V0;
        self.registers.V1 = self.registers.V1 ^ self.registers.V1;
        self.registers.V2 = self.registers.V2 ^ self.registers.V2;
        self.registers.V3 = self.registers.V3 ^ self.registers.V3;
        self.registers.V4 = self.registers.V4 ^ self.registers.V4;
        self.registers.V5 = self.registers.V5 ^ self.registers.V5;
        self.registers.V6 = self.registers.V6 ^ self.registers.V6;
        self.registers.V7 = self.registers.V7 ^ self.registers.V7;
        self.registers.V8 = self.registers.V8 ^ self.registers.V8;
        self.registers.V9 = self.registers.V9 ^ self.registers.V9;
        self.registers.VA = self.registers.VA ^ self.registers.VA;
        self.registers.VB = self.registers.VB ^ self.registers.VB;
        self.registers.VC = self.registers.VC ^ self.registers.VC;
        self.registers.VD = self.registers.VD ^ self.registers.VD;
        self.registers.VE = self.registers.VE ^ self.registers.VE;
        self.registers.VF = self.registers.VF ^ self.registers.VF;
        self.registers.I = self.registers.I ^ self.registers.I;
        self.registers.PC = self.registers.PC ^ self.registers.PC;

        self.memory = [0u8; 4096];
        self.stack = vec![0u16; 16];
    }

    fn cycle(&mut self) {
        let opcode = self.fetch_instruction();
        let instruction = self.parse_opcode(opcode);
        self.execute(instruction);
    }

    fn parse_opcode(&mut self, opcode: u16) -> Instruction {
        // Decipher opcode and prepare registers accordingly
        let mut instruction = Instruction::JUMP { address: 0x200 };

        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    0x0000 => instruction = Instruction::NOP,
                    0x00E0 => instruction = Instruction::Display,
                    0x00EE => instruction = Instruction::Return,
                    _ => eprintln!("Unexpected opcode: {:X}", opcode),
                }
            },
            0x1000 => instruction = Instruction::JUMP { address: opcode & 0x0FFF },
            0x2000 => instruction = Instruction::Call { address: opcode & 0x0FFF },
            0x3000 => instruction = Instruction::SKEQ { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), value: (opcode & 0x00FF) as u8 },
            0x4000 => instruction = Instruction::SKNEQ { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), value: (opcode & 0x00FF) as u8},
            0x5000 => instruction = Instruction::SKREQ { register1: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), register2: Target_Register::u8_to_register(((opcode >> 4) & 0x0F) as u8)},
            0x6000 => instruction = Instruction::SET { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), value: (opcode & 0x00FF) as u8},
            0x7000 => instruction = Instruction::ADD { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), value: (opcode & 0x00FF) as u8},
            0x8000 => {
                match opcode & 0xF00F {
                    0x8000 => instruction = Instruction::COPYR { register1: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), register2: Target_Register::u8_to_register(((opcode >> 4) & 0x0F) as u8) },
                    0x8001 => instruction = Instruction::OR { register1: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), register2: Target_Register::u8_to_register(((opcode >> 4) & 0x0F) as u8) },
                    0x8002 => instruction = Instruction::AND { register1: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), register2: Target_Register::u8_to_register(((opcode >> 4) & 0x0F) as u8) },
                    0x8003 => instruction = Instruction::XOR { register1: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), register2: Target_Register::u8_to_register(((opcode >> 4) & 0x0F) as u8) },
                    0x8004 => instruction = Instruction::ADDR { register1: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), register2: Target_Register::u8_to_register(((opcode >> 4) & 0x0F) as u8) },
                    0x8005 => instruction = Instruction::SUBX { register1: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), register2: Target_Register::u8_to_register(((opcode >> 4) & 0x0F) as u8) },
                    0x8006 => instruction = Instruction::SHFTR { register1: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), register2: Target_Register::u8_to_register(((opcode >> 4) & 0x0F) as u8) },
                    0x8007 => instruction = Instruction::SUBY { register1: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), register2: Target_Register::u8_to_register(((opcode >> 4) & 0x0F) as u8) },
                    0x800E => instruction = Instruction::SHFTL { register1: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), register2: Target_Register::u8_to_register(((opcode >> 4) & 0x0F) as u8) },
                    _ => eprintln!("Unexpected opcode: {:X}", opcode),
                }
            },
            0x9000 => instruction = Instruction::SKRNEQ { register1: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), register2: Target_Register::u8_to_register(((opcode >> 4) & 0x0F) as u8)},
            0xA000 => instruction = Instruction::SETI { value: opcode & 0x0FFF },
            0xB000 => instruction = Instruction::JMP0 { address: opcode & 0x0FFF},
            0xC000 => instruction = Instruction::RAND { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), value: (opcode & 0x00FF) as u8},
            0xD000 => instruction = Instruction::DRAW { register1: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8), register2: Target_Register::u8_to_register(((opcode >> 4) & 0x0F) as u8), height: (opcode & 0x000F) as u8},
            0xE000 => {
                match opcode & 0xF0FF {
                    0xE09E => instruction = Instruction::SKKEQ { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8) },
                    0xE0A1 => instruction = Instruction::SKKNEQ { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8) },
                    _ => eprintln!("Unexpected opcode: {:X}", opcode),
                }
            },
            0xF000 => {
                match opcode & 0xF0FF {
                    0xF007 => instruction = Instruction::SETXD { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8) },
                    0xF00A => instruction = Instruction::STORE { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8) },
                    0xF015 => instruction = Instruction::SETD { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8) },
                    0xF018 => instruction = Instruction::SETS { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8) },
                    0xF01E => instruction = Instruction::ADDI { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8) },
                    0xF029 => instruction = Instruction::SPRITE { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8) },
                    0xF033 => instruction = Instruction::BCD { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8) },
                    0xF055 => instruction = Instruction::DUMP { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8) },
                    0xF065 => instruction = Instruction::LOAD { register: Target_Register::u8_to_register(((opcode >> 8) & 0x0F) as u8) },
                    _ => eprintln!("Unexpected opcode: {:X}", opcode),
                }
            },
            _ => eprintln!("Unexpected opcode: {:X}", opcode),
        };
        instruction
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Call { address: a } => self.Call(a),
            Instruction::JUMP { address: a } => self.JUMP(a),
            Instruction::SET { register: r, value: v } => self.SET(r, v),
            Instruction::SETI { value: v } => self.SETI(v),
            _ => eprintln!("Unexpected instruction. Last instruction received: {:?}", instruction),
        };
    }

    fn SETI(&mut self, value: u16) {
        self.registers.I = value;
    }

    fn Call(&mut self, address: u16) {
        self.stack.push(self.registers.PC);
        self.registers.PC = address;
    }

    fn SET(&mut self, register: Target_Register, value: u8) {
        match register {
            Target_Register::V0 => self.registers.V0 = value,
            Target_Register::V1 => self.registers.V1 = value,
            Target_Register::V2 => self.registers.V2 = value,
            Target_Register::V3 => self.registers.V3 = value,
            Target_Register::V4 => self.registers.V4 = value,
            Target_Register::V5 => self.registers.V5 = value,
            Target_Register::V6 => self.registers.V6 = value,
            Target_Register::V7 => self.registers.V7 = value,
            Target_Register::V8 => self.registers.V8 = value,
            Target_Register::V9 => self.registers.V9 = value,
            Target_Register::VA => self.registers.VA = value,
            Target_Register::VB => self.registers.VB = value,
            Target_Register::VC => self.registers.VC = value,
            Target_Register::VD => self.registers.VD = value,
            Target_Register::VE => self.registers.VE = value,
            Target_Register::VF => self.registers.VF = value,
            Target_Register::I => self.registers.I = value as u16,
            Target_Register::PC => self.registers.PC = value as u16,
            _ => eprintln!("Error setting register."),
        };
    }

    fn JUMP(&mut self, address: u16) {
        self.registers.PC = address;
    }
    
    fn print_registers_state(&self) {
        println!("Current CPU registers
        {:?}", self.registers);
    }
}

fn main() {
    let mut chip8 = CPU::new();

    let mut input = String::new();
    println!("Name of file: ");
    let input_result = io::stdin().read_line(&mut input);
    if let Ok(x) = input_result {
        println!("Input grabbed successfully: return value - {}", x);
    };
    
    match input_result {
        Ok(x) => {
            if let Ok(x) = chip8.load_rom(&input) {
                // Loop in here
                chip8.cycle();
                chip8.cycle();
                chip8.cycle();
                chip8.cycle();
                chip8.cycle();

                chip8.print_registers_state();
            } else {
                eprintln!("Error opening the file.");
            };
        },
        Err(e) => eprintln!("Something went wrong with your input. Please try again."),
    };
}


