use crate::components::{cartridge::Cartridge, cpu::CPU, ram::RAM};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct VirtualMachine {
    pub cpu: CPU,
    pub ram: RAM,
    pub screen: [bool; WIDTH * HEIGHT],
}

impl VirtualMachine {
    pub fn boot(cartridge: Cartridge) {}
}
