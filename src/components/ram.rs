pub struct RAM {
    pub memory: [u8; 4096],
    pub stack: [u16; 16],
}

impl RAM {
    pub fn start() -> Self {
        Self {
            memory: [0; 4096],
            stack: [0; 16],
        }
    }
}
