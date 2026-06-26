use macroquad::prelude::*;

use crate::{
    Variant,
    components::bus::Bus,
    utils::{is_chip8_key_down, key_event_pressed_only},
};

pub const STARTING_MEMORY_ADDRESS: usize = 0x000;
pub const STARTING_ROM_ADDRESS: u16 = 0x200;

pub struct ProgramCounter {
    pub address: u16, // memory address of subsequent instruction, goes from 0x000 to 0xFFF (..4095).
                      // Start is 0x200 = 2 * 16^2 + 0 * 16^1 + 0 * 16^0 = 512
}

impl ProgramCounter {
    pub fn start() -> Self {
        Self {
            address: STARTING_ROM_ADDRESS,
        }
    }

    pub fn increment(&mut self) {
        self.address += 2 as u16;
    }

    pub fn decrement(&mut self) {
        self.address -= 2 as u16;
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

// Won't have a separate ALU;
pub struct ControlUnit {
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

    pub fn push(&mut self, address: u16, bus: &mut Bus) {
        let return_address = self.program_counter.call(address);

        let index = match self.stack_pointer {
            Some(mut index) => {
                index = index + 1;
                bus.ram.stack[index] = return_address;
                index
            }
            None => {
                let index = 0;
                bus.ram.stack[index] = return_address;
                index
            }
        };

        self.stack_pointer = Some(index);
    }

    pub fn pop(&mut self, bus: &Bus) {
        let return_address = bus.ram.stack[self.stack_pointer.unwrap()];
        self.program_counter.jump(return_address);

        // Should never be None by the time pop is called
        let index = match self.stack_pointer {
            Some(0) => None,
            _ => Some(self.stack_pointer.unwrap() - 1),
        };

        self.stack_pointer = index;
    }

    pub fn cycle(
        &mut self,
        variant: &Variant,
        bus: &mut Bus,
        index_register: &mut usize,
        registers: &mut [u8; 16],
        delay_timer: &mut u8,
        sound_timer: &mut u8,
    ) {
        self.fetch(&bus);
        let nibbles = self.decode();
        self.execute(
            nibbles,
            variant,
            bus,
            index_register,
            registers,
            delay_timer,
            sound_timer,
        );
    }

    pub fn fetch(&mut self, bus: &Bus) {
        let pc = self.program_counter.address as usize;
        let opcode = (bus.read(pc) as u16) << 8 | (bus.read(pc + 1) as u16);
        self.instruction_register = Some(opcode);

        self.program_counter.increment();
    }

    pub fn decode(&self) -> Option<[u8; 4]> {
        if let Some(opcode) = self.instruction_register {
            Some(self.separate_opcode(opcode))
        } else {
            None
        }
    }

    // https://chip8.gulrak.net/ - The classic CHIP-8 for the COSMAC VIP by Joseph Weisbecker, 1977
    // and CHIP-48 - The initial CHIP-8 port to the HP-48SX calculator by Andreas Gustafsson, 1990
    pub fn execute(
        &mut self,
        nibbles: Option<[u8; 4]>,
        variant: &Variant,
        bus: &mut Bus,
        index_register: &mut usize,
        registers: &mut [u8; 16],
        delay_timer: &mut u8,
        sound_timer: &mut u8,
    ) {
        if let Some(nibbles) = nibbles {
            let x = nibbles[1] as usize;
            let y = nibbles[2] as usize;

            let opcode = self.instruction_register.unwrap();
            let nnn = opcode & 0x0FFF;
            let nn = (opcode & 0x00FF) as u8;
            let n = (opcode & 0x000F) as u8;

            match nibbles {
                [0x0, 0x0, 0xE, 0x0] => self.op_0x00e0(bus),
                [0x0, 0x0, 0xE, 0xE] => self.op_0x00ee(bus),
                [0x0, _, _, _] => {
                    // put formal return
                    return;
                }
                [0x1, _, _, _] => self.op_0x1nnn(nnn),
                [0x2, _, _, _] => self.op_0x2nnn(nnn, bus),
                [0x3, _, _, _] | [0x4, _, _, _] => {
                    if nibbles[0] == 0x3 {
                        self.op_0x3xnn(x, nn, registers);
                    } else {
                        self.op_0x4xnn(x, nn, registers);
                    }
                }
                [0x5, _, _, 0x0] => self.op_0x5xy0(x, y, registers),
                [0x6, _, _, _] | [0x7, _, _, _] => {
                    if nibbles[0] == 0x6 {
                        self.op_0x6xnn(x, nn, registers);
                    } else {
                        self.op_0x7xnn(x, nn, registers);
                    }
                }
                [0x8, _, _, 0x0] => self.op_0x8xy0(x, y, registers),
                [0x8, _, _, 0x1] => self.op_0x8xy1(x, y, registers),
                [0x8, _, _, 0x2] => self.op_0x8xy2(x, y, registers),
                [0x8, _, _, 0x3] => self.op_0x8xy3(x, y, registers),
                [0x8, _, _, 0x4] => self.op_0x8xy4(x, y, registers),
                [0x8, _, _, 0x5] => self.op_0x8xy5(x, y, registers),
                [0x8, _, _, 0x6] => self.op_0x8xy6(x, y, registers),
                [0x8, _, _, 0x7] => self.op_0x8xy7(x, y, registers),
                [0x8, _, _, 0xE] => self.op_0x8xye(x, y, registers),
                [0x9, _, _, 0x0] => self.op_0x9xy0(x, y, registers),
                [0xA, _, _, _] => self.op_0xannn(nnn, index_register),
                [0xB, _, _, _] => {
                    if *variant == Variant::CHIP8 {
                        self.op_0xbnnn(nnn, registers);
                    } else {
                        self.op_0xbxnn(x, nn, registers);
                    }
                }
                [0xC, _, _, _] => self.op_0xcxnn(x, nn, registers),
                [0xD, _, _, _] => self.op_dxyn(x, y, n, registers, bus, index_register),
                [0xE, _, 0x9, 0xE] => self.op_0xex9e(x, registers),
                [0xE, _, 0xA, 0x1] => self.op_0xexa1(x, registers),
                [0xF, _, 0x0, 0x7] => self.op_0xfx07(x, delay_timer, registers),
                [0xF, _, 0x0, 0xA] => self.op_0xfx0a(x, registers),
                [0xF, _, 0x1, 0x5] => self.op_0xfx15(x, delay_timer, registers),
                [0xF, _, 0x1, 0x8] => self.op_0xfx18(x, sound_timer, registers),
                [0xF, _, 0x1, 0xE] => self.op_0xfx1e(x, index_register, registers),
                [0xF, _, 0x2, 0x9] => self.op_0xfx29(x, index_register, registers),
                [0xF, _, 0x3, 0x3] => self.op_0xfx33(x, index_register, registers, bus),
                [0xF, _, 0x5, 0x5] => self.op_0xfx55(x, index_register, registers, bus),
                [0xF, _, 0x6, 0x5] => self.op_0xfx65(x, index_register, registers, bus),
                _ => {}
            }
        }
    }

    fn separate_opcode(&self, opcode: u16) -> [u8; 4] {
        [
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        ]
    }

    fn op_0x00ee(&mut self, bus: &Bus) {
        self.pop(bus);
    }

    fn op_0x00e0(&self, bus: &mut Bus) {
        bus.display.clear();
    }

    fn op_0x1nnn(&mut self, nnn: u16) {
        self.program_counter.jump(nnn as u16);
    }

    fn op_0x2nnn(&mut self, nnn: u16, bus: &mut Bus) {
        self.push(nnn as u16, bus);
    }

    fn op_0x3xnn(&mut self, x: usize, nn: u8, registers: &[u8; 16]) {
        if registers[x] == nn {
            self.program_counter.skip(1);
        }
    }

    fn op_0x4xnn(&mut self, x: usize, nn: u8, registers: &[u8; 16]) {
        if registers[x] != nn {
            self.program_counter.skip(1);
        }
    }

    fn op_0x5xy0(&mut self, x: usize, y: usize, registers: &[u8; 16]) {
        if registers[x] == registers[y] {
            self.program_counter.skip(1);
        }
    }

    fn op_0x6xnn(&self, x: usize, nn: u8, registers: &mut [u8; 16]) {
        registers[x] = nn;
    }

    fn op_0x7xnn(&self, x: usize, nn: u8, registers: &mut [u8; 16]) {
        registers[x] = registers[x].wrapping_add(nn);
    }

    fn op_0x8xy0(&self, x: usize, y: usize, registers: &mut [u8; 16]) {
        registers[x] = registers[y];
    }

    fn op_0x8xy1(&self, x: usize, y: usize, registers: &mut [u8; 16]) {
        // bitwise or; 0,0 = 0; 0,1 | 1,0 = 1; 1,1 = 1
        registers[x] = registers[x] | registers[y];
    }

    fn op_0x8xy2(&self, x: usize, y: usize, registers: &mut [u8; 16]) {
        // bitwise and; 0,0 = 0; 0,1 | 1,0 = 0; 1,1 = 1
        registers[x] = registers[x] & registers[y];
    }

    fn op_0x8xy3(&self, x: usize, y: usize, registers: &mut [u8; 16]) {
        // bitwise xor; 0,0 = 0; 0,1 | 1,0 = 1; 1,1 = 0
        registers[x] = registers[x] ^ registers[y];
    }

    fn op_0x8xy4(&self, x: usize, y: usize, registers: &mut [u8; 16]) {
        let (result, overflowed) = registers[x].overflowing_add(registers[y]);
        registers[x] = result;

        registers[0xF] = if overflowed { 1 } else { 0 };
    }

    fn op_0x8xy5(&self, x: usize, y: usize, registers: &mut [u8; 16]) {
        let (result, underflowed) = registers[x].overflowing_sub(registers[y]);
        registers[x] = result;

        registers[0xF] = if underflowed { 0 } else { 1 };
    }

    fn op_0x8xy6(&self, x: usize, y: usize, registers: &mut [u8; 16]) {
        let dropped_bit = registers[y] & 1;
        registers[x] = registers[y] >> 1;
        registers[0xF] = dropped_bit;
    }

    fn op_0x8xy7(&self, x: usize, y: usize, registers: &mut [u8; 16]) {
        let (result, underflowed) = registers[y].overflowing_sub(registers[x]);
        registers[x] = result;

        registers[0xF] = if underflowed { 0 } else { 1 };
    }

    fn op_0x8xye(&self, x: usize, y: usize, registers: &mut [u8; 16]) {
        //let dropped_bit = registers[y] & 128;
        //registers[0xF] = if dropped_bit == 128 {1} else {0};
        let dropped_bit = (registers[y] >> 7) & 1;
        registers[x] = registers[y] << 1;

        registers[0xF] = dropped_bit;
    }

    fn op_0x9xy0(&mut self, x: usize, y: usize, registers: &[u8; 16]) {
        if registers[x] != registers[y] {
            self.program_counter.skip(1);
        }
    }

    fn op_0xannn(&self, nnn: u16, index_register: &mut usize) {
        *index_register = nnn as usize;
    }

    fn op_0xbnnn(&mut self, nnn: u16, registers: &[u8; 16]) {
        self.program_counter.jump(registers[0] as u16 + nnn as u16);
    }

    fn op_0xbxnn(&mut self, x: usize, nn: u8, registers: &[u8; 16]) {
        self.program_counter.jump(registers[x] as u16 + nn as u16);
    }

    fn op_0xcxnn(&self, x: usize, nn: u8, registers: &mut [u8; 16]) {
        let random_byte = (rand::rand() % 256) as u8;

        registers[x] = random_byte & nn;
    }

    fn op_dxyn(
        &mut self,
        x: usize,
        y: usize,
        n: u8,
        registers: &mut [u8; 16],
        bus: &mut Bus,
        index_register: &usize,
    ) {
        registers[0xF] = 0;
        for row in 0..(n as usize) {
            let sprite_byte = bus.read(*index_register + row);
            let pixel_y = (registers[y] as usize % bus.display.height) + row;

            if pixel_y >= bus.display.height {
                break;
            }

            for col in 0..8 {
                let pixel_x = (registers[x] as usize % bus.display.width) + col;

                if pixel_x >= bus.display.width {
                    break;
                }

                let pixel_on = (sprite_byte >> (7 - col)) & 1 == 1;

                if pixel_on {
                    if bus.display.panel[pixel_y][pixel_x] {
                        registers[0xF] = 1;
                        bus.display.panel[pixel_y][pixel_x] = false;
                    } else {
                        bus.display.panel[pixel_y][pixel_x] = true;
                    }
                }
            }
        }
    }

    fn op_0xex9e(&mut self, x: usize, registers: &[u8; 16]) {
        if is_chip8_key_down(registers[x] & 0xF) {
            self.program_counter.skip(1);
        }
    }

    fn op_0xexa1(&mut self, x: usize, registers: &[u8; 16]) {
        if !is_chip8_key_down(registers[x] & 0xF) {
            self.program_counter.skip(1);
        }
    }

    fn op_0xfx0a(&mut self, x: usize, registers: &mut [u8; 16]) {
        if let Some(key) = key_event_pressed_only() {
            registers[x] = key as u8;
        } else {
            self.program_counter.decrement();
        }
    }

    fn op_0xfx07(&self, x: usize, delay_timer: &u8, registers: &mut [u8; 16]) {
        registers[x] = *delay_timer;
    }

    fn op_0xfx15(&self, x: usize, delay_timer: &mut u8, registers: &[u8; 16]) {
        *delay_timer = registers[x];
    }

    fn op_0xfx18(&self, x: usize, sound_timer: &mut u8, registers: &[u8; 16]) {
        *sound_timer = registers[x];
    }

    fn op_0xfx1e(&self, x: usize, index_register: &mut usize, registers: &[u8; 16]) {
        *index_register += registers[x] as usize;
    }

    fn op_0xfx29(&self, x: usize, index_register: &mut usize, registers: &[u8; 16]) {
        *index_register = (registers[x] & 0x0F) as usize * 5;
    }

    fn op_0xfx33(&self, x: usize, index_register: &usize, registers: &[u8; 16], bus: &mut Bus) {
        bus.write(*index_register, registers[x] / 100);
        bus.write(*index_register + 1, (registers[x] / 10) % 10);
        bus.write(*index_register + 2, registers[x] % 10);
    }

    fn op_0xfx55(&self, x: usize, index_register: &mut usize, registers: &[u8; 16], bus: &mut Bus) {
        for index in 0..=x {
            bus.write(*index_register, registers[index]);
            *index_register += 1;
        }
    }

    fn op_0xfx65(&self, x: usize, index_register: &mut usize, registers: &mut [u8; 16], bus: &Bus) {
        for index in 0..=x {
            registers[index] = bus.read(*index_register);
            *index_register += 1;
        }
    }
}

pub struct CPU {
    pub registers: [u8; 16], // Each register holds 1 byte, u8 -> (2^8) - 1 = 0 to 255, registers are V0 to VF/ 0 -> 15
    pub index_register: usize, // Points to data in RAM
    pub control_unit: ControlUnit,
}

impl CPU {
    pub fn start() -> Self {
        Self {
            registers: [0; 16],
            index_register: STARTING_MEMORY_ADDRESS,
            control_unit: ControlUnit::start(),
        }
    }
}
