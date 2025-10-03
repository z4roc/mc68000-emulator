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

        // Erster Pass: Labels sammeln und Instruktionen parsen
        for line in assembly_lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue; // Kommentare und leere Zeilen überspringen
            }

            if line.contains(':') {
                // Label gefunden
                let label_name = line.replace(':', "").trim().to_string();
                self.labels.insert(label_name, current_address);
            } else {
                // Instruktion parsen
                let instruction = self.parse_instruction(line, current_address);
                self.instructions.push(instruction);
                current_address += 2; // Jede Instruktion ist 2 Bytes (16-bit)
            }
        }

        // Zweiter Pass: Maschinenbefehle generieren
        let mut machine_code = Vec::new();
        for i in 0..self.instructions.len() {
            if let Some(code) = self.generate_machine_code(&self.instructions[i]) {
                self.instructions[i].machine_code = Some(code);
                machine_code.push((self.instructions[i].address, code));
            }
        }

        machine_code
    }

    fn parse_instruction(&self, line: &str, address: u32) -> AssemblyInstruction {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mnemonic = parts[0].to_uppercase();
        
        let operands = if parts.len() > 1 {
            // Alle Teile außer dem ersten (Mnemonic) zusammenfügen und dann nach Komma splitten
            let operand_string = parts[1..].join(" ");
            operand_string.split(',')
                         .map(|s| s.trim().to_string())
                         .filter(|s| !s.is_empty())
                         .collect()
        } else {
            Vec::new()
        };
        
        println!("Parse: '{}' -> Mnemonic: '{}', Operands: {:?}", line, mnemonic, operands);

        AssemblyInstruction {
            address,
            mnemonic,
            operands,
            machine_code: None,
        }
    }

    fn generate_machine_code(&self, instruction: &AssemblyInstruction) -> Option<u16> {
        println!("Generiere Maschinencode für: {} {:?}", instruction.mnemonic, instruction.operands);
        match instruction.mnemonic.as_str() {
            "MOVEQ" => self.encode_moveq(instruction),
            "MOVE" => self.encode_move(instruction),
            "BRA" => self.encode_branch(instruction, 0x0), // Always
            "BEQ" => self.encode_branch(instruction, 0x7), // Equal
            "BNE" => self.encode_branch(instruction, 0x6), // Not Equal
            "BCC" => self.encode_branch(instruction, 0x4), // Carry Clear
            "BCS" => self.encode_branch(instruction, 0x5), // Carry Set
            "BPL" => self.encode_branch(instruction, 0x8), // Plus
            "BMI" => self.encode_branch(instruction, 0x9), // Minus
            "BGE" => self.encode_branch(instruction, 0xC), // Greater or Equal
            "BLT" => self.encode_branch(instruction, 0xD), // Less Than
            "BGT" => self.encode_branch(instruction, 0xE), // Greater Than
            "BLE" => self.encode_branch(instruction, 0xF), // Less or Equal
            "NOP" => Some(0x4E71),
            "ADD" => self.encode_add(instruction),
            "SUB" => self.encode_sub(instruction),
            "CMP" => self.encode_cmp(instruction),
            _ => {
                println!("Warnung: Unbekannte Instruktion: {}", instruction.mnemonic);
                None
            }
        }
    }

    // MOVEQ #immediate, Dn
    fn encode_moveq(&self, instruction: &AssemblyInstruction) -> Option<u16> {
        if instruction.operands.len() != 2 {
            println!("MOVEQ: Erwarte 2 Operanden, gefunden: {}", instruction.operands.len());
            return None;
        }

        let immediate = self.parse_immediate(&instruction.operands[0])?;
        let register = self.parse_data_register(&instruction.operands[1])?;

        // MOVEQ: 0111 RRR0 DDDDDDDD
        let opcode = 0x7000 | ((register as u16) << 9) | (immediate as u16 & 0xFF);
        Some(opcode)
    }

    // MOVE Dx, Dy (vereinfacht)
    fn encode_move(&self, instruction: &AssemblyInstruction) -> Option<u16> {
        if instruction.operands.len() != 2 {
            return None;
        }

        let source_reg = self.parse_data_register(&instruction.operands[0])?;
        let dest_reg = self.parse_data_register(&instruction.operands[1])?;

        // MOVE.W Dx,Dy: 0011 DDD 000 000 SSS (Word Move, Data Register to Data Register)
        let opcode = 0x3000 | ((dest_reg as u16) << 9) | (source_reg as u16);
        Some(opcode)
    }

    // Branch Instructions: Bcc displacement
    fn encode_branch(&self, instruction: &AssemblyInstruction, condition: u16) -> Option<u16> {
        if instruction.operands.is_empty() {
            return None;
        }

        let displacement = self.parse_branch_displacement(&instruction.operands[0], instruction.address)?;
        
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
        if instruction.operands.len() != 2 {
            return None;
        }

        if instruction.operands[0].starts_with('#') {
            // CMP #immediate, Dy - verwende CMPI
            let immediate = self.parse_immediate(&instruction.operands[0])? as u8;
            let dest_reg = self.parse_data_register(&instruction.operands[1])?;
            
            // CMPI.B #immediate, Dn: 0000 1100 0000 0RRR
            // Vereinfacht für 8-bit immediate values
            println!("Warnung: CMPI noch nicht vollständig implementiert, verwende MOVEQ/CMP workaround");
            return None;
        } else {
            // CMP Dx, Dy: 1011 DDD 001 000 SSS
            let source_reg = self.parse_data_register(&instruction.operands[0])?;
            let dest_reg = self.parse_data_register(&instruction.operands[1])?;
            let opcode = 0xB040 | ((dest_reg as u16) << 9) | (source_reg as u16);
            Some(opcode)
        }
    }

    // Hilfsfunktionen zum Parsen

    fn parse_immediate(&self, operand: &str) -> Option<i8> {
        if !operand.starts_with('#') {
            return None;
        }

        let value_str = &operand[1..];
        if value_str.starts_with("0x") || value_str.starts_with("$") {
            // Hexadezimal
            let hex_str = if value_str.starts_with("0x") {
                &value_str[2..]
            } else {
                &value_str[1..]
            };
            i8::from_str_radix(hex_str, 16).ok()
        } else {
            // Dezimal
            value_str.parse::<i8>().ok()
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

    fn parse_branch_displacement(&self, operand: &str, current_address: u32) -> Option<i8> {
        // Label-Referenz
        if let Some(&target_address) = self.labels.get(operand) {
            let displacement = (target_address as i32) - (current_address as i32) - 2;
            if displacement >= -128 && displacement <= 127 {
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
        output.push_str("\n");
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