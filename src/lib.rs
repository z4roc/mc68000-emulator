pub mod assembler;
pub mod cpu;
pub mod gui;
pub mod memory;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_initialization() {
        let cpu = cpu::CPU::new();

        // Test initial register values
        for i in 0..8 {
            assert_eq!(
                cpu.get_data_register(i),
                0,
                "Data register D{} should be 0",
                i
            );
            assert_eq!(
                cpu.get_address_register(i),
                0,
                "Address register A{} should be 0",
                i
            );
        }

        // Test initial PC and flags
        assert_eq!(cpu.get_pc(), 0, "Program counter should start at 0");
        assert_eq!(
            cpu.get_ccr(),
            0,
            "Condition code register should start at 0"
        );
    }

    #[test]
    fn test_memory_initialization() {
        let memory = memory::Memory::new();

        // Test reading uninitialized memory (should be 0)
        assert_eq!(memory.read_word(0), 0, "Uninitialized memory should be 0");
        assert_eq!(
            memory.read_word(0x1000),
            0,
            "Uninitialized memory should be 0"
        );
    }

    #[test]
    fn test_memory_read_write() {
        let mut memory = memory::Memory::new();

        // Test word operations
        memory.write_word(0x1000, 0x1234);
        assert_eq!(
            memory.read_word(0x1000),
            0x1234,
            "Should read back written word"
        );

        // Test different address
        memory.write_word(0x2000, 0xABCD);
        assert_eq!(
            memory.read_word(0x2000),
            0xABCD,
            "Should read back written word"
        );

        // Ensure first write is still intact
        assert_eq!(
            memory.read_word(0x1000),
            0x1234,
            "Previous write should be preserved"
        );
    }

    #[test]
    fn test_assembler_initialization() {
        let mut assembler = assembler::Assembler::new();

        // Test empty assembly
        let result = assembler.assemble(&[]);
        assert!(
            result.is_empty(),
            "Empty assembly should return empty result"
        );
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
        assert_eq!(
            cpu.get_data_register(0),
            42,
            "D0 should contain 42 after MOVEQ"
        );

        // Check that PC advanced
        assert_eq!(cpu.get_pc(), 2, "PC should advance to next instruction");
    }

    #[test]
    fn test_branch_instructions() {
        let mut assembler = assembler::Assembler::new();

        // Test verschiedene Branch-Instruktionen
        let lines = vec![
            "start:",
            "MOVEQ #0, D0",
            "BEQ zero",     // Branch if equal (should branch)
            "MOVEQ #1, D0", // Should be skipped
            "zero:",
            "MOVEQ #42, D1", // Should be executed
            "BRA end",       // Always branch
            "MOVEQ #99, D2", // Should be skipped
            "end:",
            "NOP",
        ];

        let result = assembler.assemble(&lines);
        assert!(
            !result.is_empty(),
            "Branch assembly should generate machine code"
        );
        assert!(result.len() >= 4, "Should generate multiple instructions");
    }

    #[test]
    fn test_jump_instruction() {
        let mut assembler = assembler::Assembler::new();

        // Test JMP instruction
        let lines = vec![
            "main:",
            "MOVEQ #1, D0",
            "JMP target",
            "MOVEQ #2, D0", // Should be skipped
            "target:",
            "MOVEQ #3, D0",
        ];

        let result = assembler.assemble(&lines);
        assert!(
            !result.is_empty(),
            "JMP assembly should generate machine code"
        );

        // Check that JMP instruction is generated
        let has_jump = result.iter().any(|(_, instr)| instr & 0xFFF8 == 0x4EF8);
        assert!(has_jump, "Should contain JMP instruction");
    }

    #[test]
    fn test_loop_pattern() {
        let mut assembler = assembler::Assembler::new();

        // Test Loop-ähnliches Muster
        let lines = vec![
            "MOVEQ #5, D0", // Counter
            "loop:",
            "MOVEQ #1, D1",  // Do some work
            "ADD D1, D2",    // Accumulate
            "MOVEQ #-1, D3", // Decrement
            "ADD D3, D0",    // D0 = D0 - 1
            "BNE loop",      // Branch if not zero
            "MOVEQ #0, D7",  // End marker
        ];

        let result = assembler.assemble(&lines);
        assert!(
            !result.is_empty(),
            "Loop pattern should generate machine code"
        );
        assert!(
            result.len() >= 6,
            "Should generate multiple instructions for loop"
        );

        // Check that we have branch back instruction
        let has_branch_back = result.iter().any(|(_, instr)| {
            let opcode = (instr >> 12) & 0xF;
            opcode == 0x6 // Branch instruction
        });
        assert!(
            has_branch_back,
            "Should contain branch instruction for loop"
        );
    }

    #[test]
    fn test_comprehensive_program() {
        let mut assembler = assembler::Assembler::new();

        // Komplexes Testprogramm - nur Assembly-Test
        let lines = vec![
            "MOVEQ #10, D0", // Load counter
            "MOVEQ #0, D1",  // Sum accumulator
            "loop:",
            "ADD D0, D1",    // Add counter to sum
            "MOVEQ #-1, D2", // Decrement value
            "ADD D2, D0",    // Decrement counter
            "BNE loop",      // Loop if not zero
            "MOVEQ #42, D7", // End marker
            "NOP",
        ];

        let machine_code = assembler.assemble(&lines);
        assert!(!machine_code.is_empty(), "Complex program should assemble");

        // Check that key instructions are generated
        let moveq_count = machine_code
            .iter()
            .filter(|(_, code)| (*code >> 12) == 0x7)
            .count();
        assert!(moveq_count >= 3, "Should have multiple MOVEQ instructions");

        let add_count = machine_code
            .iter()
            .filter(|(_, code)| (*code >> 12) == 0xD)
            .count();
        assert!(add_count >= 2, "Should have ADD instructions");

        let branch_count = machine_code
            .iter()
            .filter(|(_, code)| (*code >> 12) == 0x6)
            .count();
        assert!(branch_count >= 1, "Should have branch instruction");

        // Check NOP is present
        let nop_present = machine_code.iter().any(|(_, code)| *code == 0x4E71);
        assert!(nop_present, "Should have NOP instruction");
    }

    #[test]
    fn test_all_branch_conditions() {
        let mut assembler = assembler::Assembler::new();

        // Test alle Branch-Bedingungen
        let lines = vec![
            "BRA target", // Always
            "BEQ target", // Equal
            "BNE target", // Not Equal
            "BCC target", // Carry Clear
            "BCS target", // Carry Set
            "BPL target", // Plus
            "BMI target", // Minus
            "BGE target", // Greater or Equal
            "BLT target", // Less Than
            "BGT target", // Greater Than
            "BLE target", // Less or Equal
            "target:",
            "NOP",
        ];

        let result = assembler.assemble(&lines);
        assert_eq!(
            result.len(),
            12,
            "Should generate 11 branch instructions + 1 NOP"
        );

        // Check that all are branch instructions (opcode 0x6)
        let branch_count = result
            .iter()
            .filter(|(_, instr)| (instr >> 12) & 0xF == 0x6)
            .count();
        assert_eq!(branch_count, 11, "Should have 11 branch instructions");
    }

    #[test]
    fn test_memory_operations() {
        let mut memory = memory::Memory::new();

        // Test verschiedene Speicher-Operationen
        memory.write_word(0x1000, 0x1234);
        memory.write_word(0x2000, 0x5678);
        memory.write_word(0x3000, 0x9ABC);

        assert_eq!(memory.read_word(0x1000), 0x1234);
        assert_eq!(memory.read_word(0x2000), 0x5678);
        assert_eq!(memory.read_word(0x3000), 0x9ABC);

        // Test big-endian byte order
        memory.write_word(0x4000, 0xABCD);
        assert_eq!(memory.read_byte(0x4000), 0xAB, "High byte should be first");
        assert_eq!(memory.read_byte(0x4001), 0xCD, "Low byte should be second");
    }

    #[test]
    fn test_assembler_error_handling() {
        let mut assembler = assembler::Assembler::new();

        // Test fehlerhafte Assembly-Codes
        let empty_result = assembler.assemble(&[]);
        assert!(
            empty_result.is_empty(),
            "Empty input should return empty result"
        );

        let comment_only =
            assembler.assemble(&["; This is just a comment", "  ; Another comment  "]);
        assert!(
            comment_only.is_empty(),
            "Comment-only input should return empty result"
        );

        // Test unbekannte Instruktion
        let unknown_instr = assembler.assemble(&["UNKNOWN D0, D1"]);
        assert!(
            unknown_instr.is_empty(),
            "Unknown instruction should not generate code"
        );
    }
}
