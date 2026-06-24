use crate::error_message;
use std::path::PathBuf;

// Each instruction has 4 hexedecimals, each 4 bits = 2 bytes
pub struct Cartridge {
    pub buffer: [u8; 3584], // Max 4096 bytes of RAM in VM, first 512 bytes is for font + interpreter, pad with 0
}

impl Cartridge {
    pub fn load(filename: Option<PathBuf>) -> Result<Self, std::io::Error> {
        let Some(rom_path) = filename else {
            let file_error_msg = "Issue occured with file selection".to_string();

            return Err(error_message(file_error_msg));
        };

        // Only accept the standard extension
        if rom_path.extension().and_then(|e| e.to_str()) != Some("ch8") {
            let ext_msg = format!("ROM does not end in .ch8 extension: {:?}", rom_path);

            return Err(error_message(ext_msg));
        }

        let heap_buffer = std::fs::read(rom_path)?;
        if heap_buffer.len() > 3584 || heap_buffer.len() < 2 {
            let buffer_msg = format!(
                "ROM is {} bytes, max should be 3584 and min is theoretically 2.",
                heap_buffer.len()
            );

            return Err(error_message(buffer_msg));
        }

        Ok(Self {
            buffer: Self::to_stack(&heap_buffer),
        })
    }

    fn to_stack(heap_buffer: &Vec<u8>) -> [u8; 3584] {
        let mut stack_buffer = [0u8; 3584];
        stack_buffer[..heap_buffer.len()].copy_from_slice(heap_buffer);

        stack_buffer
    }
}
