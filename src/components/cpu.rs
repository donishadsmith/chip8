use crate::components::ram::RAM;

const STARTING_ADDRESS: u16 = 0x200;
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
        self.increment();
        self.address += 2 * step;
    }

    // Remember to jump back to return address
    pub fn call(&mut self, address: u16) -> u16 {
        let return_address = self.address;
        self.address = address;

        return_address
    }
}

pub struct CPU {
    program_counter: ProgramCounter,
    registers: [u8; 16], // Each register holds 1 byte, u8 -> (2^8) - 1 = 0 to 255, registers are V0 to VF/ 0 -> 15
    instruction_register: u16, // Current instruction
    stack_pointer: Option<usize>, // point to top of stack
}

impl CPU {
    pub fn start() -> Self {
        Self {
            program_counter: ProgramCounter::start(),
            registers: [0; 16],
            instruction_register: STARTING_ADDRESS,
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

    pub fn cycle(&mut self) {
        //self.fetch();
        //self.decode();
        //self.execute();
    }

    pub fn fetch() {}

    pub fn decode() {}

    pub fn execute() {}
}
