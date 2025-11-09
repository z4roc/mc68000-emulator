// MC68000 Assembly Parser
// Wandelt Assembly-Strings in Maschinenbefehle um

use std::collections::HashMap;

pub struct Assembler {
    labels: HashMap<String, u32>,
    instructions: Vec<AssemblyInstruction>,
}

#[derive(Debug, Clone)]
struct AssemblyInstruction {
    address: u32,
    mnemonic: String,
    operands: Vec<String>,
    machine_code: Option<u16>,
    extension_word: Option<u16>,  // Für Adressen bei MOVE.L etc.
    size: u32,  // Größe der Instruktion in Bytes (2 oder 4)
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

impl Assembler {
    pub fn new() -> Self {
        Assembler {
            labels: HashMap::new(),
            instructions: Vec::new(),
        }
    }

    /// Parst Assembly-Code und gibt Maschinenbefehle zurück
    pub fn assemble(&mut self, assembly_lines: &[&str]) -> Vec<(u32, u16)> {
        self.instructions.clear();
        self.labels.clear();

        let mut current_address = 0u32;
        let mut data_values: Vec<(u32, u32)> = Vec::new();  // (address, value) für DC.L

        // Erster Pass: Labels sammeln und Instruktionen parsen
        for line in assembly_lines {
            let mut line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue; // Kommentare und leere Zeilen überspringen
            }

            // Handle END directive
            if line.to_uppercase().starts_with("END") {
                break;
            }

            // Handle ORG directive
            if line.to_uppercase().starts_with("ORG") {
                if let Some(addr) = self.parse_org_directive(line) {
                    current_address = addr;
                }
                continue;
            }

            // Handle labels (with or without colon)
            if line.contains(':') {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                let label_name = parts[0].trim().to_string();
                self.labels.insert(label_name, current_address);
                
                // Check if there's an instruction on the same line
                if parts.len() > 1 {
                    line = parts[1].trim();
                    if line.is_empty() {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Handle data directives
            if line.to_uppercase().contains("DC.") || line.to_uppercase().contains("DS.") {
                if let Some((label, size, value)) = self.parse_data_directive_with_value(line) {
                    if !label.is_empty() {
                        self.labels.insert(label, current_address);
                    }
                    // If DC.L with value, store it for memory initialization
                    if let Some(val) = value {
                        data_values.push((current_address, val));
                    }
                    current_address += size;
                }
                continue;
            }

            // Instruktion parsen
            let instruction = self.parse_instruction(line, current_address);
            current_address += instruction.size;  // Berücksichtige Extension Words
            self.instructions.push(instruction);
        }

        // Zweiter Pass: Maschinenbefehle generieren
        let mut machine_code = Vec::new();
        
        // Add data values first (DC.L directives)
        for (addr, value) in data_values {
            // Split 32-bit value into two 16-bit words (big-endian)
            machine_code.push((addr, (value >> 16) as u16));
            machine_code.push((addr + 2, (value & 0xFFFF) as u16));
        }
        
        for i in 0..self.instructions.len() {
            let inst = &self.instructions[i];
            if let Some((code, ext_word)) = self.encode_instruction_with_ext(inst) {
                machine_code.push((inst.address, code));
                
                // Extension Word hinzufügen, falls vorhanden
                if let Some(ext) = ext_word {
                    machine_code.push((inst.address + 2, ext));
                }
            }
        }

        machine_code
    }

    fn encode_instruction_with_ext(&self, instruction: &AssemblyInstruction) -> Option<(u16, Option<u16>)> {
        println!(
            "Generiere Maschinencode für: {} {:?}",
            instruction.mnemonic, instruction.operands
        );
        match instruction.mnemonic.as_str() {
            "MOVEQ" => self.encode_moveq(instruction).map(|c| (c, None)),
            "MOVE" => self.encode_move_with_ext(instruction),
            "MOVEA" => self.encode_movea_with_ext(instruction),
            "MULS" => self.encode_muls_with_ext(instruction),
            "TST" => self.encode_tst(instruction).map(|c| (c, None)),
            "SUBQ" => self.encode_subq(instruction).map(|c| (c, None)),
            "ASL" => self.encode_asl(instruction).map(|c| (c, None)),
            "DBRA" => self.encode_dbra(instruction).map(|c| (c, None)),
            "BRA" => self.encode_branch(instruction, 0x0).map(|c| (c, None)), // Always
            "BEQ" => self.encode_branch(instruction, 0x7).map(|c| (c, None)), // Equal
            "BNE" => self.encode_branch(instruction, 0x6).map(|c| (c, None)), // Not Equal
            "BCC" => self.encode_branch(instruction, 0x4).map(|c| (c, None)), // Carry Clear
            "BCS" => self.encode_branch(instruction, 0x5).map(|c| (c, None)), // Carry Set
            "BPL" => self.encode_branch(instruction, 0x8).map(|c| (c, None)), // Plus
            "BMI" => self.encode_branch(instruction, 0x9).map(|c| (c, None)), // Minus
            "BGE" => self.encode_branch(instruction, 0xC).map(|c| (c, None)), // Greater or Equal
            "BLT" => self.encode_branch(instruction, 0xD).map(|c| (c, None)), // Less Than
            "BGT" => self.encode_branch(instruction, 0xE).map(|c| (c, None)), // Greater Than
            "BLE" => self.encode_branch(instruction, 0xF).map(|c| (c, None)), // Less or Equal
            "NOP" => Some((0x4E71, None)),
            "SIMHALT" => Some((0x4E72, None)), // Custom halt instruction
            "ADD" => self.encode_add(instruction).map(|c| (c, None)),
            "SUB" => self.encode_sub(instruction).map(|c| (c, None)),
            "CMP" => self.encode_cmp_with_ext(instruction),
            "JMP" | "JUMP" => self.encode_jump(instruction).map(|c| (c, None)),
            _ => {
                println!("Warnung: Unbekannte Instruktion: {}", instruction.mnemonic);
                None
            }
        }
    }

    fn parse_instruction(&self, line: &str, address: u32) -> AssemblyInstruction {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return AssemblyInstruction {
                address,
                mnemonic: String::new(),
                operands: Vec::new(),
                machine_code: None,
                extension_word: None,
                size: 2,
            };
        }

        // Split mnemonic from size suffix (e.g., MOVE.L -> MOVE and .L)
        let mnemonic_parts: Vec<&str> = parts[0].split('.').collect();
        let mnemonic = mnemonic_parts[0].to_uppercase();

        let operands = if parts.len() > 1 {
            // Alle Teile außer dem ersten (Mnemonic) zusammenfügen und dann nach Komma splitten
            let operand_string = parts[1..].join(" ");
            operand_string
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        };

        // Bestimme die Größe der Instruktion (prüfe auf Extension Words)
        let size = if operands.len() >= 2 {
            let src = &operands[0];
            let dst = &operands[operands.len() - 1];
            
            // Instruktionen die Extension Words brauchen:
            // 1. MOVE.L/MOVEA.L mit #immediate oder Labels
            // 2. CMP.L mit #immediate
            // 3. MULS mit #immediate
            
            if (mnemonic == "MOVE" || mnemonic == "MOVEA") && mnemonic_parts.get(1) == Some(&"L") {
                // MOVE.L/MOVEA.L mit #immediate oder Label braucht Extension Word
                if src.starts_with('#') || (!src.starts_with('D') && !src.starts_with('A') && !src.starts_with('(')) {
                    4  // Instruktion + Extension Word
                } else if !dst.starts_with('D') && !dst.starts_with('A') && !dst.starts_with('(') {
                    4  // Destination ist Label
                } else {
                    2  // Register-zu-Register
                }
            } else if mnemonic == "CMP" && mnemonic_parts.get(1) == Some(&"L") && src.starts_with('#') {
                4  // CMP.L #imm, Dn
            } else if mnemonic == "MULS" && src.starts_with('#') {
                4  // MULS #imm, Dn
            } else {
                2  // Standardgröße
            }
        } else {
            2  // Keine oder nur ein Operand
        };

        println!(
            "Parse: '{}' -> Mnemonic: '{}', Operands: {:?}, Size: {}",
            line, mnemonic, operands, size
        );

        AssemblyInstruction {
            address,
            mnemonic,
            operands,
            machine_code: None,
            extension_word: None,
            size,
        }
    }

    // MOVEQ #immediate, Dn
    fn encode_moveq(&self, instruction: &AssemblyInstruction) -> Option<u16> {
        if instruction.operands.len() != 2 {
            println!(
                "MOVEQ: Erwarte 2 Operanden, gefunden: {}",
                instruction.operands.len()
            );
            return None;
        }

        let immediate = self.parse_immediate(&instruction.operands[0])?;
        let register = self.parse_data_register(&instruction.operands[1])?;

        // MOVEQ: 0111 RRR0 DDDDDDDD
        let opcode = 0x7000 | ((register as u16) << 9) | (immediate as u16 & 0xFF);
        Some(opcode)
    }

    // MOVE with extension word support
    fn encode_move_with_ext(&self, instruction: &AssemblyInstruction) -> Option<(u16, Option<u16>)> {
        if instruction.operands.len() != 2 {
            return None;
        }

        let source = &instruction.operands[0];
        let dest = &instruction.operands[1];

        // MOVE.L #immediate, Dn
        if source.starts_with('#') {
            if let Some(dest_reg) = self.parse_data_register(dest) {
                if let Some(imm_value) = self.parse_immediate_u16(source) {
                    // MOVE.L #imm, Dn: 0010 DDD 111 111 100 + extension word
                    // Binary: 0010 000 1 111 111 00 = 0x21FC for D0
                    let opcode = 0x21FC | ((dest_reg as u16) << 9);
                    return Some((opcode, Some(imm_value)));
                }
            }
        }

        // MOVE.L (An), Dn - Address Register Indirect to Data Register
        if let Some(src_areg) = self.parse_indirect_register(source) {
            if let Some(dest_reg) = self.parse_data_register(dest) {
                // MOVE.L (An), Dn: 0010 DDD 010 000 AAA
                let opcode = 0x2010 | ((dest_reg as u16) << 9) | (src_areg as u16);
                return Some((opcode, None));
            }
        }

        // MOVE.L Dn, (An) - Data Register to Address Register Indirect
        if let Some(src_reg) = self.parse_data_register(source) {
            if let Some(dest_areg) = self.parse_indirect_register(dest) {
                // MOVE.L Dn, (An): 0010 AAA 110 000 RRR
                let opcode = 0x2080 | ((dest_areg as u16) << 9) | (src_reg as u16);
                return Some((opcode, None));
            }
        }

        // Check if source is a data register
        if let Some(source_reg) = self.parse_data_register(source) {
            // MOVE Dx, Dy
            if let Some(dest_reg) = self.parse_data_register(dest) {
                // MOVE.W Dx,Dy: 0011 DDD 000 000 SSS (Word Move, Data Register to Data Register)
                let opcode = 0x3000 | ((dest_reg as u16) << 9) | (source_reg as u16);
                return Some((opcode, None));
            }
        }

        // Check if source is a label or absolute address (MOVE.L label, Dn)
        if let Some(dest_reg) = self.parse_data_register(dest) {
            // Lookup label address
            if let Some(&label_addr) = self.labels.get(source) {
                // MOVE.L (xxx).W, Dn
                // Format: 0010 DDD 111 111 000
                let opcode = 0x2078 | ((dest_reg as u16) << 9);
                return Some((opcode, Some(label_addr as u16)));
            }
        }

        // MOVE.L Dn, label - store to memory
        if let Some(source_reg) = self.parse_data_register(source) {
            // Lookup label address
            if let Some(&label_addr) = self.labels.get(dest) {
                // MOVE.L Dn, (xxx).W
                // Format: 0010 0011 110 000 RRR
                let opcode = 0x23C0 | (source_reg as u16);
                return Some((opcode, Some(label_addr as u16)));
            }
        }

        None
    }

    // MOVE Dx, Dy or MOVE.L label, Dn (old version, now deprecated)
    fn encode_move(&self, instruction: &mut AssemblyInstruction) -> Option<u16> {
        self.encode_move_with_ext(instruction).map(|(code, _)| code)
    }

    // MOVEA - Move Address (loads address into An register)
    fn encode_movea_with_ext(&self, instruction: &AssemblyInstruction) -> Option<(u16, Option<u16>)> {
        if instruction.operands.len() != 2 {
            return None;
        }

        let source = &instruction.operands[0];
        let dest = &instruction.operands[1];

        // MOVEA.L #label, An
        if source.starts_with('#') {
            if let Some(dest_areg) = self.parse_address_register(dest) {
                // Try to parse as immediate or label
                let label_name = &source[1..]; // Remove #
                if let Some(&label_addr) = self.labels.get(label_name) {
                    // MOVEA.L #imm, An: 0010 AAA 111 111 100 + extension word
                    let opcode = 0x207C | ((dest_areg as u16) << 9);
                    return Some((opcode, Some(label_addr as u16)));
                }
            }
        }

        None
    }

    // MULS - Signed Multiply
    fn encode_muls(&self, instruction: &AssemblyInstruction) -> Option<u16> {
        self.encode_muls_with_ext(instruction).map(|(code, _)| code)
    }

    fn encode_muls_with_ext(&self, instruction: &AssemblyInstruction) -> Option<(u16, Option<u16>)> {
        if instruction.operands.len() != 2 {
            return None;
        }

        let source = &instruction.operands[0];
        let dest = &instruction.operands[1];

        // MULS #imm, Dn or MULS Dn, Dm
        if let Some(dest_reg) = self.parse_data_register(dest) {
            if source.starts_with('#') {
                // MULS.W #imm, Dn: 1100 RRR 111 111 100 + extension word
                if let Some(imm_value) = self.parse_immediate_u16(source) {
                    let opcode = 0xC1FC | ((dest_reg as u16) << 9);
                    return Some((opcode, Some(imm_value)));
                }
            } else if let Some(src_reg) = self.parse_data_register(source) {
                // MULS Ds, Dd: 1100 RRR 111 000 SSS
                let opcode = 0xC1C0 | ((dest_reg as u16) << 9) | (src_reg as u16);
                return Some((opcode, None));
            }
        }

        None
    }

    // Branch Instructions: Bcc displacement
    fn encode_branch(&self, instruction: &AssemblyInstruction, condition: u16) -> Option<u16> {
        if instruction.operands.is_empty() {
            return None;
        }

        let displacement =
            self.parse_branch_displacement(&instruction.operands[0], instruction.address)?;

        // Bcc: 0110 CCCC DDDDDDDD
        let opcode = 0x6000 | (condition << 8) | (displacement as u16 & 0xFF);
        Some(opcode)
    }

    // ADD Dx, Dy (vereinfacht)
    fn encode_add(&self, instruction: &AssemblyInstruction) -> Option<u16> {
        if instruction.operands.len() != 2 {
            return None;
        }

        let source_reg = self.parse_data_register(&instruction.operands[0])?;
        let dest_reg = self.parse_data_register(&instruction.operands[1])?;

        // ADD.W Dx,Dy: 1101 DDD 001 000 SSS
        let opcode = 0xD040 | ((dest_reg as u16) << 9) | (source_reg as u16);
        Some(opcode)
    }

    // SUB Dx, Dy (vereinfacht)
    fn encode_sub(&self, instruction: &AssemblyInstruction) -> Option<u16> {
        if instruction.operands.len() != 2 {
            return None;
        }

        let source_reg = self.parse_data_register(&instruction.operands[0])?;
        let dest_reg = self.parse_data_register(&instruction.operands[1])?;

        // SUB.W Dx,Dy: 1001 DDD 001 000 SSS
        let opcode = 0x9040 | ((dest_reg as u16) << 9) | (source_reg as u16);
        Some(opcode)
    }

    // CMP #immediate, Dy oder CMP Dx, Dy
    fn encode_cmp(&self, instruction: &AssemblyInstruction) -> Option<u16> {
        self.encode_cmp_with_ext(instruction).map(|(code, _)| code)
    }

    fn encode_cmp_with_ext(&self, instruction: &AssemblyInstruction) -> Option<(u16, Option<u16>)> {
        if instruction.operands.len() != 2 {
            return None;
        }

        if instruction.operands[0].starts_with('#') {
            // CMP.L #immediate, Dn - use CMPI.L
            let immediate = self.parse_immediate_u16(&instruction.operands[0])?;
            let dest_reg = self.parse_data_register(&instruction.operands[1])?;

            // CMPI.L #imm, Dn: 0000 1100 1000 0RRR + extension word
            let opcode = 0x0C80 | (dest_reg as u16);
            return Some((opcode, Some(immediate)));
        } else {
            // CMP Dx, Dy: 1011 DDD 001 000 SSS
            let source_reg = self.parse_data_register(&instruction.operands[0])?;
            let dest_reg = self.parse_data_register(&instruction.operands[1])?;
            let opcode = 0xB040 | ((dest_reg as u16) << 9) | (source_reg as u16);
            return Some((opcode, None));
        }
    }

    // JMP absolute address
    fn encode_jump(&self, instruction: &AssemblyInstruction) -> Option<u16> {
        if instruction.operands.len() != 1 {
            return None;
        }

        // JMP $address oder JMP address (absolute)
        if self
            .parse_immediate_address(&instruction.operands[0])
            .is_some()
        {
            // JMP.W $xxxx.W: 0100 1110 1111 1000
            Some(0x4EF8)
        } else {
            println!(
                "JMP benötigt eine absolute Adresse: {}",
                instruction.operands[0]
            );
            None
        }
    }

    // TST.L Dn - Test operand
    fn encode_tst(&self, instruction: &AssemblyInstruction) -> Option<u16> {
        if instruction.operands.len() != 1 {
            return None;
        }

        let reg = self.parse_data_register(&instruction.operands[0])?;
        // TST.L Dn: 0100 1010 1000 0RRR
        let opcode = 0x4A80 | (reg as u16);
        Some(opcode)
    }

    // SUBQ.L #immediate, Dn - Subtract quick
    fn encode_subq(&self, instruction: &AssemblyInstruction) -> Option<u16> {
        if instruction.operands.len() != 2 {
            return None;
        }

        let immediate = self.parse_immediate(&instruction.operands[0])? as u16;
        let reg = self.parse_data_register(&instruction.operands[1])?;
        
        // Convert 8 to 0 for encoding (SUBQ uses 0 to represent 8)
        let data = if immediate == 8 { 0 } else { immediate & 0x7 };
        
        // SUBQ.L #imm, Dn: 0101 DDD 110 000 RRR
        let opcode = 0x5180 | (data << 9) | (reg as u16);
        Some(opcode)
    }

    // ASL.L #immediate, Dn - Arithmetic shift left
    fn encode_asl(&self, instruction: &AssemblyInstruction) -> Option<u16> {
        if instruction.operands.len() != 2 {
            return None;
        }

        let shift_count = self.parse_immediate(&instruction.operands[0])? as u16;
        let reg = self.parse_data_register(&instruction.operands[1])?;
        
        // Convert 8 to 0 for encoding
        let count = if shift_count == 8 { 0 } else { shift_count & 0x7 };
        
        // ASL.L #imm, Dn: 1110 CCC 110 100 RRR
        let opcode = 0xE180 | (count << 9) | (reg as u16);
        Some(opcode)
    }

    // DBRA Dn, label - Decrement and branch
    fn encode_dbra(&self, instruction: &AssemblyInstruction) -> Option<u16> {
        if instruction.operands.len() != 2 {
            return None;
        }

        let reg = self.parse_data_register(&instruction.operands[0])?;
        let displacement = self.parse_branch_displacement(&instruction.operands[1], instruction.address)?;
        
        // DBRA Dn, disp: 0101 0001 1100 1RRR
        // Note: DBRA displacement is 16-bit, but we'll use 8-bit for simplicity
        let opcode = 0x51C8 | (reg as u16);
        Some(opcode)
    }

    // Hilfsfunktionen zum Parsen

    fn parse_org_directive(&self, line: &str) -> Option<u32> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }
        
        let addr_str = parts[1];
        if addr_str.starts_with('$') {
            u32::from_str_radix(&addr_str[1..], 16).ok()
        } else if addr_str.starts_with("0x") {
            u32::from_str_radix(&addr_str[2..], 16).ok()
        } else {
            addr_str.parse::<u32>().ok()
        }
    }

    fn parse_data_directive(&self, line: &str) -> Option<(String, u32)> {
        self.parse_data_directive_with_value(line).map(|(label, size, _)| (label, size))
    }

    fn parse_data_directive_with_value(&self, line: &str) -> Option<(String, u32, Option<u32>)> {
        // Parse DC.L and DS.L directives
        let line_upper = line.to_uppercase();
        
        // Extract label and directive part
        let label: String;
        let directive_str: String;
        
        if line.contains(':') {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            label = parts[0].trim().to_string();
            directive_str = parts.get(1).unwrap_or(&"").trim().to_string();
        } else {
            // Label might be the first word before DC/DS
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let first_word = parts[0].to_uppercase();
                // If first word is not a directive, it's a label
                if !first_word.starts_with("DC") && !first_word.starts_with("DS") {
                    label = parts[0].to_string();
                    directive_str = parts[1..].join(" ");
                } else {
                    label = String::new();
                    directive_str = line.to_string();
                }
            } else {
                label = String::new();
                directive_str = line.to_string();
            }
        }
        
        // Determine size based on directive
        let size = if line_upper.contains("DC.L") || line_upper.contains("DS.L") {
            4 // Long word = 4 bytes
        } else if line_upper.contains("DC.W") || line_upper.contains("DS.W") {
            2 // Word = 2 bytes
        } else if line_upper.contains("DC.B") || line_upper.contains("DS.B") {
            1 // Byte = 1 byte
        } else {
            2 // Default to word
        };
        
        // Extract value for DC directives (DS just reserves space)
        let value = if line_upper.contains("DC.") {
            // Find the value after DC.L/DC.W/DC.B
            let parts: Vec<&str> = directive_str.split_whitespace().collect();
            if parts.len() >= 2 {
                // parts[0] should be DC.L, parts[1] should be the value
                let value_str = parts[1].trim();
                if value_str.starts_with('$') {
                    u32::from_str_radix(&value_str[1..], 16).ok()
                } else if value_str.starts_with("0x") {
                    u32::from_str_radix(&value_str[2..], 16).ok()
                } else {
                    value_str.parse::<u32>().ok()
                }
            } else {
                None
            }
        } else {
            None // DS directive - no initial value
        };
        
        Some((label, size, value))
    }

    fn parse_immediate(&self, operand: &str) -> Option<i8> {
        if !operand.starts_with('#') {
            return None;
        }

        let value_str = &operand[1..];
        if value_str.starts_with("0x") || value_str.starts_with("$") {
            // Hexadezimal
            let hex_str = if let Some(stripped) = value_str.strip_prefix("0x") {
                stripped
            } else {
                &value_str[1..]
            };
            i8::from_str_radix(hex_str, 16).ok()
        } else {
            // Dezimal
            value_str.parse::<i8>().ok()
        }
    }

    fn parse_immediate_u16(&self, operand: &str) -> Option<u16> {
        if !operand.starts_with('#') {
            return None;
        }

        let value_str = &operand[1..];
        if value_str.starts_with("0x") || value_str.starts_with("$") {
            // Hexadezimal
            let hex_str = if let Some(stripped) = value_str.strip_prefix("0x") {
                stripped
            } else {
                &value_str[1..]
            };
            u16::from_str_radix(hex_str, 16).ok()
        } else {
            // Dezimal
            value_str.parse::<u16>().ok()
        }
    }

    fn parse_data_register(&self, operand: &str) -> Option<u8> {
        if operand.len() == 2 && operand.starts_with('D') {
            let reg_num = operand.chars().nth(1)?;
            if reg_num.is_ascii_digit() {
                let num = reg_num.to_digit(10)? as u8;
                if num <= 7 {
                    return Some(num);
                }
            }
        }
        None
    }

    fn parse_address_register(&self, operand: &str) -> Option<u8> {
        if operand.len() == 2 && operand.starts_with('A') {
            let reg_num = operand.chars().nth(1)?;
            if reg_num.is_ascii_digit() {
                let num = reg_num.to_digit(10)? as u8;
                if num <= 7 {
                    return Some(num);
                }
            }
        }
        None
    }

    fn parse_indirect_register(&self, operand: &str) -> Option<u8> {
        // Parse (An) - Address Register Indirect
        if operand.starts_with('(') && operand.ends_with(')') {
            let inner = &operand[1..operand.len()-1];
            return self.parse_address_register(inner);
        }
        None
    }

    fn parse_immediate_address(&self, operand: &str) -> Option<u16> {
        // $xxxx oder 0xxxxx Format
        if operand.starts_with('$') {
            u16::from_str_radix(&operand[1..], 16).ok()
        } else if operand.starts_with("0x") {
            u16::from_str_radix(&operand[2..], 16).ok()
        } else if operand.chars().all(|c| c.is_ascii_digit()) {
            operand.parse::<u16>().ok()
        } else {
            // Label lookup
            if let Some(&address) = self.labels.get(operand) {
                Some(address as u16)
            } else {
                None
            }
        }
    }

    fn parse_branch_displacement(&self, operand: &str, current_address: u32) -> Option<i8> {
        // Label-Referenz
        if let Some(&target_address) = self.labels.get(operand) {
            let displacement = (target_address as i32) - (current_address as i32) - 2;
            if (-128..=127).contains(&displacement) {
                return Some(displacement as i8);
            }
        }

        // Direkte Displacement-Angabe
        if operand.starts_with('+') || operand.starts_with('-') {
            return operand.parse::<i8>().ok();
        }

        None
    }

    /// Debug: Zeigt alle geparsten Instruktionen an
    pub fn print_assembly(&self) {
        println!("=== Assembly Listing ===");
        for instruction in &self.instructions {
            if let Some(machine_code) = instruction.machine_code {
                println!(
                    "{:06X}: {:04X}  {} {}",
                    instruction.address,
                    machine_code,
                    instruction.mnemonic,
                    instruction.operands.join(", ")
                );
            }
        }

        if !self.labels.is_empty() {
            println!("\n=== Labels ===");
            for (label, address) in &self.labels {
                println!("{}: {:06X}", label, address);
            }
        }
    }

    /// Debug: Schreibt Assembly-Listing in einen String
    pub fn print_assembly_to_string(&self, output: &mut String) {
        output.push_str("=== Assembly Listing ===\n");
        for instruction in &self.instructions {
            if let Some(machine_code) = instruction.machine_code {
                output.push_str(&format!(
                    "{:06X}: {:04X}  {} {}\n",
                    instruction.address,
                    machine_code,
                    instruction.mnemonic,
                    instruction.operands.join(", ")
                ));
            }
        }

        if !self.labels.is_empty() {
            output.push_str("\n=== Labels ===\n");
            for (label, address) in &self.labels {
                output.push_str(&format!("{}: {:06X}\n", label, address));
            }
        }
        output.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moveq_parsing() {
        let mut assembler = Assembler::new();
        let code = assembler.assemble(&["MOVEQ #42, D0"]);
        assert_eq!(code[0].1, 0x702A);
    }

    #[test]
    fn test_move_parsing() {
        let mut assembler = Assembler::new();
        let code = assembler.assemble(&["MOVE D0, D1"]);
        assert_eq!(code[0].1, 0x3200);
    }

    #[test]
    fn test_branch_parsing() {
        let mut assembler = Assembler::new();
        let code = assembler.assemble(&["BRA +2"]);
        assert_eq!(code[0].1, 0x6002);
    }
}
