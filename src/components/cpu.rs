use crate::components::ram::RAM;

pub const STARTING_ADDRESS: u16 = 0x200;
pub struct ProgramCounter {
    address: u16, // memory address of subsequent instruction, goes from 0x000 to 0xFFF (..4095).
                  // Start is 0x200 = 2 * 16^2 + 0 * 16^2 + 0 * 16^0 = 512
}

impl ProgramCounter {
    pub fn start() -> Self {
        Self {
            address: STARTING_ADDRESS,
        }
    }

    pub fn increment(&mut self) {
        self.address += 2 as u16;
    }

    pub fn jump(&mut self, address: u16) {
        self.address = address;
    }

    pub fn skip(&mut self, step: u16) {
        self.address += 2 * step;
    }

    // Remember to jump back to return address
    pub fn call(&mut self, address: u16) -> u16 {
        let return_address = self.address;
        self.address = address;

        return_address
    }
}

struct ControlUnit {
    instruction_register: Option<u16>, // Current instruction
    program_counter: ProgramCounter,
    stack_pointer: Option<usize>, // point to top of stack
}

impl ControlUnit {
    pub fn start() -> Self {
        Self {
            instruction_register: None,
            program_counter: ProgramCounter::start(),
            stack_pointer: None,
        }
    }

    pub fn push(&mut self, ram: &mut RAM, address: u16) {
        let return_address = self.program_counter.call(address);

        let index = match self.stack_pointer {
            Some(mut index) => {
                index = index + 1;
                ram.stack[index] = return_address;
                index
            }
            None => {
                let index = 0;
                ram.stack[index] = return_address;
                index
            }
        };

        self.stack_pointer = Some(index);
    }

    pub fn pop(&mut self, ram: &RAM) {
        let return_address = ram.stack[self.stack_pointer.unwrap()];
        self.program_counter.jump(return_address);

        // Should never be None by the time pop is called
        let index = match self.stack_pointer {
            Some(0) => None,
            _ => Some(self.stack_pointer.unwrap() - 1),
        };

        self.stack_pointer = index;
    }

    pub fn cycle(&mut self, ram: &RAM) {
        self.fetch(ram);
        self.decode();
        self.execute();
    }

    pub fn fetch(&mut self, ram: &RAM) {
        let pc = self.program_counter.address as usize;
        let opcode = (ram.code_segment[pc] as u16) << 8 | (ram.code_segment[pc + 1] as u16);
        self.instruction_register = Some(opcode);

        self.program_counter.increment();
    }

    pub fn decode(&self) {}

    pub fn execute(&self) {}
}

pub struct CPU {
    registers: [u8; 16], // Each register holds 1 byte, u8 -> (2^8) - 1 = 0 to 255, registers are V0 to VF/ 0 -> 15
    control_unit: ControlUnit,
}

impl CPU {
    pub fn start() -> Self {
        Self {
            registers: [0; 16],
            control_unit: ControlUnit::start(),
        }
    }
}
