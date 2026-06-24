use crate::{
    components::{
        cartridge::Cartridge,
        cpu::{CPU, STARTING_ROM_ADDRESS},
        display::Display,
        ram::RAM,
    },
    fontset::FONTSET,
};

pub struct VirtualMachine {
    pub cpu: CPU,
    pub ram: RAM,
    pub display: Display,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub variant: &'static str,
}

impl VirtualMachine {
    pub fn boot(cartridge: Cartridge, variant: &'static str) -> Self {
        Self {
            cpu: CPU::start(),
            ram: Self::controller(cartridge),
            display: Display::on(),
            delay_timer: 0,
            sound_timer: 0,
            variant: variant,
        }
    }

    pub fn controller(cartridge: Cartridge) -> RAM {
        let mut ram = RAM::start();
        for index in 0..FONTSET.len() {
            ram.memory[index] = FONTSET[index];
        }

        for (index, byte) in cartridge.buffer.iter().enumerate() {
            let address = STARTING_ROM_ADDRESS + index as u16;
            if address >= 4096 {
                break;
            }

            ram.memory[address as usize] = *byte;
        }

        ram
    }

    //https://www.reddit.com/r/EmuDev/comments/1ev3ool/chip8_instructions_per_second/
    const CYCLES_PER_FRAME: u8 = 10;
    pub fn process(&mut self) {
        for _ in 0..Self::CYCLES_PER_FRAME {
            self.cpu.control_unit.cycle(
                self.variant,
                &mut self.ram,
                &mut self.cpu.index_register,
                &mut self.cpu.registers,
                &mut self.delay_timer,
                &mut self.sound_timer,
                &mut self.display,
            );
        }
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                self.beep();
            }
            self.sound_timer -= 1;
        }
    }

    // Todo: to implement sound in the future
    pub fn beep(&self) {}
}
