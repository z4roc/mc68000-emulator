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
        
        println!("PC: 0x{:06X}, Instruction: 0x{:04X}, Opcode: 0x{:01X}", 
                 self.program_counter, instruction, opcode);

        // EXECUTE: Je nach Opcode entsprechende Funktion aufrufen
        match opcode {
            0x1 | 0x2 | 0x3 => self.move_instruction(instruction, memory),
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
        println!("MOVE instruction detected");
        
        // Vereinfachtes MOVE D0,D1 (0x3200)
        if instruction == 0x3200 {
            self.data_registers[1] = self.data_registers[0];
            self.update_flags_for_result(self.data_registers[1] as i32);
        }
        
        self.program_counter += 2;
    }

    fn addq_subq_instruction(&mut self, instruction: u16, memory: &mut Memory) {
        println!("ADDQ/SUBQ instruction detected");
        // Hier würden Sie ADDQ/SUBQ implementieren
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
        
        println!("Branch instruction, condition: 0x{:01X}, displacement: {}", condition, displacement);
        
        if self.check_condition(condition) {
            self.program_counter = ((self.program_counter as i32) + (displacement as i32) + 2) as u32;
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
            0x0 => true,  // BRA - Always branch
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
        println!("Miscellaneous instruction: 0x{:04X}", instruction);
        self.program_counter += 2;
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
        println!("AND instruction: 0x{:04X}", instruction);
        self.program_counter += 2;
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
            println!("D{}: 0x{:08X}  A{}: 0x{:08X}", 
                     i, self.data_registers[i], i, self.address_registers[i]);
        }
        println!("PC: 0x{:08X}", self.program_counter);
        println!("CCR: 0x{:02X} (N:{} Z:{} V:{} C:{})", 
                 self.condition_code_register,
                 (self.condition_code_register >> 3) & 1,
                 (self.condition_code_register >> 2) & 1,
                 (self.condition_code_register >> 1) & 1,
                 self.condition_code_register & 1);
        println!("SR: 0x{:04X}", self.status_register);
    }
}