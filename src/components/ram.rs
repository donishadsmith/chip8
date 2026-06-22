pub struct RAM {
    pub code_segment: [u8; 4096],
    pub stack: [u16; 16],
}

impl RAM {
    pub fn start() -> Self {
        Self {
            code_segment: [0; 4096],
            stack: [0; 16],
        }
    }
}
