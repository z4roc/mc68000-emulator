
// Foliensatz 2 S.33, Adressraum
/*
    24 Bit Adressraum = 16 MB
 */
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            data: vec![0; 16 * 1024 * 1024], // 16 MB Adressraum
        }
    }

    pub fn read_byte(&self, address: u32) -> u8 {
        self.data[address as usize]
    }

    pub fn write_byte(&mut self, address: u32, value: u8) {
        self.data[address as usize] = value;
    }

    // MC68000 ist Big-Endian
    pub fn read_word(&self, address: u32) -> u16 {
        let high_byte = self.data[address as usize] as u16;
        let low_byte = self.data[(address + 1) as usize] as u16;
        (high_byte << 8) | low_byte
    }

    pub fn write_word(&mut self, address: u32, value: u16) {
        self.data[address as usize] = (value >> 8) as u8;        // High Byte
        self.data[(address + 1) as usize] = (value & 0xFF) as u8; // Low Byte
    }

    pub fn read_long(&self, address: u32) -> u32 {
        let high_word = self.read_word(address) as u32;
        let low_word = self.read_word(address + 2) as u32;
        (high_word << 16) | low_word
    }

    pub fn write_long(&mut self, address: u32, value: u32) {
        self.write_word(address, (value >> 16) as u16);      // High Word
        self.write_word(address + 2, (value & 0xFFFF) as u16); // Low Word
    }
}