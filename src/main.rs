mod components;
mod fontset;
mod vm;

use crate::components::cartridge::Cartridge;

fn main() -> Result<(), std::io::Error> {
    let cartridge = Cartridge::load()?;

    println!("N bytes {}", cartridge.n_bytes);
    println!("Bytes: {:02X?}", &cartridge.buffer);

    Ok(())
}
