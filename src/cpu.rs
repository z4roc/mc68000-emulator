// CCR Flags S.31 Foliensatz 2
/*
    User Mode:
    Sign-Flag N=1 wenn Ergebnis negativ (Stelle 3)
    Zero-Flag Z=1 wenn Ergebnis 0 (Stelle 2)
    Overflow-Flag V=1 wenn Überlauf (Stelle 1)
    Carry-Flag C=1 wenn Übertrag (Stelle 0)

    Supervisor Mode:
    Trace-Mode T1/T2 (Stellen 15/14)
    Supervisor-Flag S=1 wenn Supervisor Mode (Stelle 13)
    Interrupt-Enable-Flag I1/I2/I3 (Stellen 8, 9, 10)
    Extended Carry (Stelle 4)

*/

use crate::memory::Memory;

pub struct CPU {
    // Section User Mode S.28 Foliensatz 2
    data_registers: [u32; 8],
    address_registers: [u32; 8],
    program_counter: u32,
    condition_code_register: u8,

    // Supervisor Mode S.28 Foliensatz 2
    supervisor_stack_pointer: u32,
    vector_base_register: u32,
    status_register: u16,
}

// Kernel ROM Mach ich mal nicht
impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            data_registers: [0; 8],
            address_registers: [0; 8],
            program_counter: 0,
            condition_code_register: 0,
            supervisor_stack_pointer: 0,
            vector_base_register: 0,
            status_register: 0,
        }
    }

    pub fn reset(&mut self) {
        self.program_counter = 0;
        self.condition_code_register = 0;
        self.status_register = 0x2700; // Supervisor Mode, Interrupts enabled
    }

    // Getter methods for testing
    pub fn get_pc(&self) -> u32 {
        self.program_counter
    }

    pub fn get_data_register(&self, reg: usize) -> u32 {
        if reg < 8 {
            self.data_registers[reg]
        } else {
            0
        }
    }

    pub fn get_address_register(&self, reg: usize) -> u32 {
        if reg < 8 {
            self.address_registers[reg]
        } else {
            0
        }
    }

    // Hauptausführungsschleife
    pub fn run(&mut self, memory: &mut Memory) {
        loop {
            self.execute_instruction(memory);
        }
    }

    // Fetch-Decode-Execute Zyklus
    pub fn execute_instruction(&mut self, memory: &mut Memory) {
        // FETCH: Instruktion aus Speicher lesen (16-bit Wort)
        let instruction = memory.read_word(self.program_counter);

        // DECODE: Instruktion analysieren
        let opcode = (instruction >> 12) & 0xF; // Obere 4 Bits

        println!(
            "PC: 0x{:06X}, Instruction: 0x{:04X}, Opcode: 0x{:01X}",
            self.program_counter, instruction, opcode
        );

        // EXECUTE: Je nach Opcode entsprechende Funktion aufrufen
        match opcode {
            0x0 => self.miscellaneous_instruction(instruction, memory), // CMPI and other immediate operations
            0x1..=0x3 => self.move_instruction(instruction, memory),
            0x4 => self.miscellaneous_instruction(instruction, memory),
            0x5 => self.addq_subq_instruction(instruction, memory),
            0x6 => self.branch_instruction(instruction, memory),
            0x7 => self.moveq_instruction(instruction, memory),
            0x8 => self.or_instruction(instruction, memory),
            0x9 | 0xB => self.sub_cmp_instruction(instruction, memory),
            0xA => self.unimplemented_instruction(instruction),
            0xC => self.and_instruction(instruction, memory),
            0xD => self.add_instruction(instruction, memory),
            0xE => self.shift_instruction(instruction, memory),
            0xF => self.unimplemented_instruction(instruction),
            _ => self.unimplemented_instruction(instruction),
        }
    }

    // Beispiel-Implementierungen für verschiedene Instruktionsgruppen
    fn move_instruction(&mut self, instruction: u16, memory: &mut Memory) {
        let size = (instruction >> 12) & 0x3; // 1=byte, 3=word, 2=long
        let dest_reg = ((instruction >> 9) & 0x7) as usize;
        let dest_mode = (instruction >> 6) & 0x7;
        let src_mode = (instruction >> 3) & 0x7;
        let src_reg = (instruction & 0x7) as usize;

        println!(
            "MOVE instruction: size={}, dest_reg={}, dest_mode={}, src_mode={}, src_reg={}",
            size, dest_reg, dest_mode, src_mode, src_reg
        );

        // MOVE.L #immediate, Dn: 0010 DDD 111 111 100
        // size=2 (long), dest_mode=7, src_mode=7, src_reg=4
        if size == 2 && dest_mode == 7 && src_mode == 7 && src_reg == 4 {
            self.program_counter += 2;
            let immediate = memory.read_word(self.program_counter) as u32;
            self.program_counter += 2;
            self.data_registers[dest_reg] = immediate;
            println!("  MOVE.L #0x{:08X}, D{}", immediate, dest_reg);
            return;
        }

        // MOVEA.L #immediate, An: 0010 AAA 001 111 100
        // size=2, dest_mode=1 (for address register), src_mode=7, src_reg=4
        if size == 2 && dest_mode == 1 && src_mode == 7 && src_reg == 4 {
            self.program_counter += 2;
            let immediate = memory.read_word(self.program_counter) as u32;
            self.program_counter += 2;
            self.address_registers[dest_reg] = immediate;
            println!("  MOVEA.L #0x{:08X}, A{}", immediate, dest_reg);
            return;
        }

        // MOVE.L (An), Dn: 0010 DDD 010 000 AAA
        if size == 2 && dest_mode == 0 && src_mode == 2 {
            let address = self.address_registers[src_reg];
            let value = memory.read_long(address);
            self.data_registers[dest_reg] = value;
            println!(
                "  MOVE.L (A{}=0x{:04X}), D{} -> 0x{:08X}",
                src_reg, address, dest_reg, value
            );
            self.program_counter += 2;
            return;
        }

        // MOVE.L Dn, (An): 0010 AAA 010 000 RRR
        if size == 2 && dest_mode == 2 && src_mode == 0 {
            let address = self.address_registers[dest_reg];
            let value = self.data_registers[src_reg];
            memory.write_long(address, value);
            println!(
                "  MOVE.L D{}, (A{}=0x{:04X}) -> 0x{:08X}",
                src_reg, dest_reg, address, value
            );
            self.program_counter += 2;
            return;
        }

        // Vereinfachtes MOVE D0,D1 (0x3200)
        if instruction == 0x3200 {
            self.data_registers[1] = self.data_registers[0];
            self.update_flags_for_result(self.data_registers[1] as i32);
        }

        self.program_counter += 2;
    }

    fn addq_subq_instruction(&mut self, instruction: u16, memory: &mut Memory) {
        // SUBQ.L #imm, Dn: 0101 DDD 1 SS MMM RRR
        // ADDQ.L #imm, Dn: 0101 DDD 0 SS MMM RRR
        // DDD = data (bits 9-11)
        // Bit 8 = 1 for SUBQ, 0 for ADDQ
        // SS = size (bits 6-7)
        // MMM = mode (bits 3-5)
        // RRR = register (bits 0-2)

        let data = (instruction >> 9) & 0x7; // Extract bits 9-11
        let is_subq = (instruction & 0x0100) != 0; // Check bit 8
        let size = (instruction >> 6) & 0x3; // Extract bits 6-7
        let mode = (instruction >> 3) & 0x7; // Extract bits 3-5
        let reg = (instruction & 0x7) as usize; // Extract bits 0-2

        // Convert 0 to 8 (SUBQ/ADDQ use 0 to represent 8)
        let immediate = if data == 0 { 8 } else { data as i32 };

        if is_subq {
            // SUBQ
            let old_value = self.data_registers[reg] as i32;
            let new_value = old_value - immediate;
            self.data_registers[reg] = new_value as u32;

            println!(
                "SUBQ.L #{}, D{} -> {} - {} = {}",
                immediate, reg, old_value, immediate, new_value
            );

            self.update_flags_for_result(new_value);
        } else {
            // ADDQ
            let old_value = self.data_registers[reg] as i32;
            let new_value = old_value + immediate;
            self.data_registers[reg] = new_value as u32;

            println!(
                "ADDQ.L #{}, D{} -> {} + {} = {}",
                immediate, reg, old_value, immediate, new_value
            );

            self.update_flags_for_result(new_value);
        }

        self.program_counter += 2;
    }

    fn moveq_instruction(&mut self, instruction: u16, memory: &mut Memory) {
        let register = (instruction >> 9) & 0x7; // Zielregister (D0-D7)
        let immediate = (instruction & 0xFF) as i8 as i32; // 8-bit signed immediate

        println!("MOVEQ #0x{:02X}, D{}", immediate & 0xFF, register);

        self.data_registers[register as usize] = immediate as u32;
        self.update_flags_for_result(immediate);
        self.program_counter += 2;
    }

    fn branch_instruction(&mut self, instruction: u16, memory: &mut Memory) {
        let condition = (instruction >> 8) & 0xF;
        let displacement = (instruction & 0xFF) as i8;

        println!(
            "Branch instruction, condition: 0x{:01X}, displacement: {}",
            condition, displacement
        );

        if self.check_condition(condition) {
            self.program_counter =
                ((self.program_counter as i32) + (displacement as i32) + 2) as u32;
        } else {
            self.program_counter += 2;
        }
    }

    fn unimplemented_instruction(&mut self, instruction: u16) {
        println!("Unimplemented instruction: 0x{:04X}", instruction);
        self.program_counter += 2;
    }

    // Hilfsfunktionen
    fn update_flags_for_result(&mut self, result: i32) {
        // Zero Flag
        if result == 0 {
            self.condition_code_register |= 0x04; // Z-Flag setzen
        } else {
            self.condition_code_register &= !0x04; // Z-Flag löschen
        }

        // Negative Flag
        if result < 0 {
            self.condition_code_register |= 0x08; // N-Flag setzen
        } else {
            self.condition_code_register &= !0x08; // N-Flag löschen
        }
    }

    fn check_condition(&self, condition: u16) -> bool {
        match condition {
            0x0 => true,                                       // BRA - Always branch
            0x1 => false, // BSR - Branch to subroutine (vereinfacht)
            0x2 => (self.condition_code_register & 0x01) != 0, // BHI - Branch if higher
            0x3 => (self.condition_code_register & 0x01) == 0, // BLS - Branch if lower or same
            0x4 => (self.condition_code_register & 0x01) == 0, // BCC - Branch if carry clear
            0x5 => (self.condition_code_register & 0x01) != 0, // BCS - Branch if carry set
            0x6 => (self.condition_code_register & 0x04) == 0, // BNE - Branch if not equal
            0x7 => (self.condition_code_register & 0x04) != 0, // BEQ - Branch if equal
            _ => false,
        }
    }

    // Platzhalter für weitere Instruktionsgruppen
    fn miscellaneous_instruction(&mut self, instruction: u16, memory: &mut Memory) {
        // Check for CMPI.L #imm, Dn: 0000 1100 1000 0RRR
        if (instruction & 0xFFF8) == 0x0C80 {
            let dest_reg = (instruction & 0x7) as usize;
            self.program_counter += 2;
            let immediate = memory.read_word(self.program_counter) as i32;
            self.program_counter += 2;

            let dest_value = self.data_registers[dest_reg] as i32;
            let result = dest_value - immediate;

            println!(
                "CMPI.L #0x{:04X}, D{} -> {} - {} = {}",
                immediate, dest_reg, dest_value, immediate, result
            );

            self.update_flags_for_result(result);
            return;
        }

        // Check for JMP instruction (0x4EF8 = JMP (xxx).W)
        if instruction == 0x4EF8 {
            // JMP (xxx).W - Jump to absolute word address
            // The target address follows as the next word
            let target_address = memory.read_word(self.program_counter + 2) as u32;
            println!("JMP to address: 0x{:06X}", target_address);
            self.program_counter = target_address;
        } else if instruction == 0x4E71 {
            // NOP
            println!("NOP");
            self.program_counter += 2;
        } else if instruction == 0x4E72 {
            // SIMHALT - Custom halt instruction
            println!("SIMHALT - Program stopped");
            // Don't increment PC - this signals the end
            // The GUI should detect this by checking if PC hasn't changed
        } else {
            println!("Miscellaneous instruction: 0x{:04X}", instruction);
            self.program_counter += 2;
        }
    }

    fn or_instruction(&mut self, instruction: u16, memory: &mut Memory) {
        println!("OR instruction: 0x{:04X}", instruction);
        self.program_counter += 2;
    }

    fn sub_cmp_instruction(&mut self, instruction: u16, memory: &mut Memory) {
        let opcode_high = (instruction >> 12) & 0xF;

        if opcode_high == 0xB {
            // CMP instruction: 1011 DDD SSS MMM RRR
            let dest_reg = ((instruction >> 9) & 0x7) as usize;
            let source_reg = (instruction & 0x7) as usize;

            println!("CMP.W D{}, D{}", source_reg, dest_reg);

            let source_value = self.data_registers[source_reg] as i32;
            let dest_value = self.data_registers[dest_reg] as i32;
            let result = dest_value - source_value; // CMP subtrahiert aber speichert nicht

            self.update_flags_for_result(result);
        } else {
            // SUB instruction
            let dest_reg = ((instruction >> 9) & 0x7) as usize;
            let source_reg = (instruction & 0x7) as usize;

            println!("SUB.W D{}, D{}", source_reg, dest_reg);

            let source_value = self.data_registers[source_reg] as i32;
            let dest_value = self.data_registers[dest_reg] as i32;
            let result = dest_value - source_value;

            self.data_registers[dest_reg] = result as u32;
            self.update_flags_for_result(result);
        }

        self.program_counter += 2;
    }

    fn and_instruction(&mut self, instruction: u16, memory: &mut Memory) {
        // Check if this is actually MULS instruction
        // MULS.W #imm, Dn: 1100 RRR 111 111 100
        // MULS.W Ds, Dd:   1100 RRR 111 000 SSS
        let dest_mode = (instruction >> 6) & 0x7;
        let src_mode = (instruction >> 3) & 0x7;
        let src_reg = (instruction & 0x7) as usize;

        if dest_mode == 7 && src_mode == 7 && src_reg == 4 {
            // MULS.W #imm, Dn - has extension word
            let dest_reg = ((instruction >> 9) & 0x7) as usize;
            self.program_counter += 2; // Skip opcode
            let immediate = memory.read_word(self.program_counter) as i16;
            self.program_counter += 2; // Skip extension word

            let dest_value = self.data_registers[dest_reg] as i16;
            let result = (dest_value as i32) * (immediate as i32);

            println!(
                "MULS.W #{}, D{} -> {} * {} = {}",
                immediate, dest_reg, dest_value, immediate, result
            );

            self.data_registers[dest_reg] = result as u32;
            self.update_flags_for_result(result);
        } else if dest_mode == 7 && src_mode == 0 {
            // MULS.W Ds, Dd
            let dest_reg = ((instruction >> 9) & 0x7) as usize;

            let source_value = self.data_registers[src_reg] as i16;
            let dest_value = self.data_registers[dest_reg] as i16;
            let result = (source_value as i32) * (dest_value as i32);

            println!(
                "MULS.W D{}, D{} -> {} * {} = {}",
                src_reg, dest_reg, source_value, dest_value, result
            );

            self.data_registers[dest_reg] = result as u32;
            self.update_flags_for_result(result);
            self.program_counter += 2;
        } else {
            println!("AND instruction: 0x{:04X}", instruction);
            self.program_counter += 2;
        }
    }

    fn add_instruction(&mut self, instruction: u16, memory: &mut Memory) {
        // ADD.W Dx,Dy: 1101 DDD 001 000 SSS
        let dest_reg = ((instruction >> 9) & 0x7) as usize;
        let source_reg = (instruction & 0x7) as usize;

        println!("ADD.W D{}, D{}", source_reg, dest_reg);

        let source_value = self.data_registers[source_reg] as i32;
        let dest_value = self.data_registers[dest_reg] as i32;
        let result = dest_value + source_value;

        self.data_registers[dest_reg] = result as u32;
        self.update_flags_for_result(result);
        self.program_counter += 2;
    }

    fn shift_instruction(&mut self, instruction: u16, memory: &mut Memory) {
        println!("Shift instruction: 0x{:04X}", instruction);
        self.program_counter += 2;
    }

    // Debug-Funktionen
    pub fn print_registers(&self) {
        println!("=== CPU State ===");
        for i in 0..8 {
            println!(
                "D{}: 0x{:08X}  A{}: 0x{:08X}",
                i, self.data_registers[i], i, self.address_registers[i]
            );
        }
        println!("PC: 0x{:08X}", self.program_counter);
        println!(
            "CCR: 0x{:02X} (N:{} Z:{} V:{} C:{})",
            self.condition_code_register,
            (self.condition_code_register >> 3) & 1,
            (self.condition_code_register >> 2) & 1,
            (self.condition_code_register >> 1) & 1,
            self.condition_code_register & 1
        );
        println!("SR: 0x{:04X}", self.status_register);
    }

    pub fn set_pc(&mut self, address: u32) {
        self.program_counter = address;
    }

    pub fn get_ccr(&self) -> u8 {
        self.condition_code_register
    }

    pub fn get_sr(&self) -> u16 {
        self.status_register
    }
}
