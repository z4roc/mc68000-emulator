pub mod cpu;
pub mod memory;
pub mod assembler;
pub mod gui;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_initialization() {
        let cpu = cpu::CPU::new();
        
        // Test initial register values
        for i in 0..8 {
            assert_eq!(cpu.get_data_register(i), 0, "Data register D{} should be 0", i);
            assert_eq!(cpu.get_address_register(i), 0, "Address register A{} should be 0", i);
        }
        
        // Test initial PC and flags
        assert_eq!(cpu.get_pc(), 0, "Program counter should start at 0");
        assert_eq!(cpu.get_ccr(), 0, "Condition code register should start at 0");
    }

    #[test]
    fn test_memory_initialization() {
        let memory = memory::Memory::new();
        
        // Test reading uninitialized memory (should be 0)
        assert_eq!(memory.read_word(0), 0, "Uninitialized memory should be 0");
        assert_eq!(memory.read_word(0x1000), 0, "Uninitialized memory should be 0");
    }

    #[test]
    fn test_memory_read_write() {
        let mut memory = memory::Memory::new();
        
        // Test word operations
        memory.write_word(0x1000, 0x1234);
        assert_eq!(memory.read_word(0x1000), 0x1234, "Should read back written word");
        
        // Test different address
        memory.write_word(0x2000, 0xABCD);
        assert_eq!(memory.read_word(0x2000), 0xABCD, "Should read back written word");
        
        // Ensure first write is still intact
        assert_eq!(memory.read_word(0x1000), 0x1234, "Previous write should be preserved");
    }

    #[test]
    fn test_assembler_initialization() {
        let mut assembler = assembler::Assembler::new();
        
        // Test empty assembly
        let result = assembler.assemble(&[]);
        assert!(result.is_empty(), "Empty assembly should return empty result");
    }

    #[test]
    fn test_assembler_moveq() {
        let mut assembler = assembler::Assembler::new();
        
        // Test MOVEQ instruction
        let lines = vec!["MOVEQ #42, D0"];
        let result = assembler.assemble(&lines);
        
        assert!(!result.is_empty(), "MOVEQ should generate machine code");
        assert_eq!(result.len(), 1, "MOVEQ should generate one instruction");
        
        let (address, instruction) = result[0];
        assert_eq!(address, 0, "First instruction should be at address 0");
        
        // MOVEQ #42, D0 should be: 0111 000 0 00101010 = 0x702A
        assert_eq!(instruction, 0x702A, "MOVEQ #42, D0 should be 0x702A");
    }

    #[test]
    fn test_cpu_moveq_execution() {
        let mut cpu = cpu::CPU::new();
        let mut memory = memory::Memory::new();
        
        // Load MOVEQ #42, D0 instruction (0x702A) at address 0
        memory.write_word(0, 0x702A);
        
        // Execute one instruction
        cpu.execute_instruction(&mut memory);
        
        // Check that D0 now contains 42
        assert_eq!(cpu.get_data_register(0), 42, "D0 should contain 42 after MOVEQ");
        
        // Check that PC advanced
        assert_eq!(cpu.get_pc(), 2, "PC should advance to next instruction");
    }
}