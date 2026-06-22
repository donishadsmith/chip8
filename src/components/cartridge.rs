// Each instruction has 4 hexedecimals, each 4 bits = 2 bytes
pub struct Cartridge {
    pub buffer: [u8; 3584], // Max 4096 bytes of RAM in VM, first 512 bytes is for font + interpreter, pad with 0
    pub n_bytes: usize,
}

impl Cartridge {
    pub fn load() -> Result<Self, std::io::Error> {
        let rom_path = Self::terminal_reader()?;

        // Only accept the standard extension
        if !rom_path.ends_with(".ch8") {
            let ext_msg = format!("ROM does not end in .ch8 extension: {rom_path}");

            return Err(Self::error_message(ext_msg));
        }

        let heap_buffer = std::fs::read(rom_path)?;
        if heap_buffer.len() > 3584 || heap_buffer.len() < 2 {
            let buffer_msg = format!(
                "ROM is {} bytes, max should be 3584 and min is theoretically 2.",
                heap_buffer.len()
            );

            return Err(Self::error_message(buffer_msg));
        }

        Ok(Self {
            buffer: Self::to_stack(&heap_buffer),
            n_bytes: heap_buffer.len(),
        })
    }

    fn terminal_reader() -> Result<String, std::io::Error> {
        println!("Input path to ROM:");

        let mut filename = String::new();
        std::io::stdin().read_line(&mut filename)?;

        Ok(filename.trim().to_string())
    }

    fn to_stack(heap_buffer: &Vec<u8>) -> [u8; 3584] {
        let mut stack_buffer = [0u8; 3584];
        stack_buffer[..heap_buffer.len()].copy_from_slice(heap_buffer);

        stack_buffer
    }

    fn error_message(message: String) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, message)
    }
}
