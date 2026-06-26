// No bus in Chip-8 but practice for future emulators
use crate::components::{display::Display, ram::RAM};

pub struct Bus {
    pub ram: RAM,
    pub display: Display,
}

impl Bus {
    pub fn read(&self, address: usize) -> u8 {
        self.ram.memory[address]
    }

    pub fn write(&mut self, address: usize, value: u8) {
        self.ram.memory[address] = value
    }
}
