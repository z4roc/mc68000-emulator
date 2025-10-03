// MC68000 Emulator GUI mit egui
use eframe::egui;
use crate::{assembler, cpu, memory};

pub struct EmulatorApp {
    // Assembly Code Editor
    assembly_code: String,
    
    // Emulator State
    cpu: cpu::CPU,
    memory: memory::Memory,
    assembler: assembler::Assembler,
    
    // GUI State
    is_running: bool,
    step_mode: bool,
    current_step: usize,
    machine_code: Vec<(u32, u16)>,
    
    // Output/Logs
    output_log: String,
    error_message: String,
    
    // Layout State
    show_compare_view: bool,
    bottom_panel_height: f32,
    side_panel_width: f32,
}

impl Default for EmulatorApp {
    fn default() -> Self {
        Self {
            assembly_code: String::from(
"MOVEQ #42, D0    ; Lade 42 in D0
MOVEQ #7, D1     ; Lade 7 in D1  
ADD D0, D1       ; D1 = D1 + D0 (7 + 42 = 49)
MOVEQ #49, D2    ; Lade erwartetes Ergebnis in D2
CMP D2, D1       ; Vergleiche D1 mit D2 (49)
BEQ success      ; Springe zu success wenn gleich
MOVEQ #-1, D0    ; Fehler: -1 in D0
BRA end          ; Springe zum Ende
success:         ; Label für Erfolg
MOVEQ #1, D0     ; Erfolg: 1 in D0
end:             ; Label für Ende
NOP              ; No Operation
BRA end          ; Endlos-Loop"
            ),
            cpu: cpu::CPU::new(),
            memory: memory::Memory::new(),
            assembler: assembler::Assembler::new(),
            is_running: false,
            step_mode: true,
            current_step: 0,
            machine_code: Vec::new(),
            output_log: String::new(),
            error_message: String::new(),
            
            // Layout State
            show_compare_view: false,
            bottom_panel_height: 150.0,
            side_panel_width: 300.0,
        }
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // VS Code Style Layout
        
        // Top Panel - Toolbar (smaller height, buttons right-aligned)
        egui::TopBottomPanel::top("toolbar")
            .exact_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Title links
                    ui.heading("🖥️ MC68000 Emulator");
                    
                    // Push buttons to the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.checkbox(&mut self.step_mode, "Step Mode");
                        
                        ui.separator();
                        
                        if ui.button("🔄 Reset").on_hover_text("Reset CPU (Ctrl+R)").clicked() {
                            self.reset_emulator();
                        }
                        
                        if ui.button("⏸️ Step").on_hover_text("Step one instruction (F10)").clicked() && !self.machine_code.is_empty() {
                            self.step_program();
                        }
                        
                        if ui.button("▶️ Run").on_hover_text("Run program (F5)").clicked() && !self.machine_code.is_empty() {
                            self.run_program();
                        }
                        
                        if ui.button("🔧 Assemble").on_hover_text("Assemble code (F9)").clicked() {
                            self.assemble_code();
                            self.show_compare_view = true; // Show compare view after assembly
                        }
                    });
                });
            });

        // Bottom Panel - Output/Console (VS Code style)
        egui::TopBottomPanel::bottom("console")
            .resizable(true)
            .default_height(self.bottom_panel_height)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("📋 Output");
                    ui.separator();
                    
                    // Console tabs (like VS Code)
                    if ui.selectable_label(true, "Terminal").clicked() {
                        // Future: multiple console tabs
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑️").on_hover_text("Clear").clicked() {
                            self.output_log.clear();
                        }
                    });
                });
                
                ui.separator();
                
                // Error Messages
                if !self.error_message.is_empty() {
                    ui.colored_label(egui::Color32::RED, &self.error_message);
                    ui.separator();
                }
                
                // Output Console
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.output_log)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                        );
                    });
            });

        // Right Panel - CPU Registers (collapsible)
        egui::SidePanel::right("cpu_panel")
            .resizable(true)
            .default_width(self.side_panel_width)
            .show(ctx, |ui| {
                ui.heading("🧠 CPU State");
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Data Registers
                    ui.collapsing("Data Registers", |ui| {
                        egui::Grid::new("data_regs").show(ui, |ui| {
                            for i in 0..8 {
                                ui.label(format!("D{}:", i));
                                ui.monospace(format!("0x{:08X}", self.cpu.get_data_register(i)));
                                ui.end_row();
                            }
                        });
                    });

                    // Address Registers  
                    ui.collapsing("Address Registers", |ui| {
                        egui::Grid::new("addr_regs").show(ui, |ui| {
                            for i in 0..8 {
                                ui.label(format!("A{}:", i));
                                ui.monospace(format!("0x{:08X}", self.cpu.get_address_register(i)));
                                ui.end_row();
                            }
                        });
                    });

                    // Special Registers
                    ui.collapsing("Special Registers", |ui| {
                        egui::Grid::new("special_regs").show(ui, |ui| {
                            ui.label("PC:");
                            ui.monospace(format!("0x{:08X}", self.cpu.get_pc()));
                            ui.end_row();
                            
                            ui.label("CCR:");
                            let ccr = self.cpu.get_ccr();
                            ui.monospace(format!("0x{:02X} (N:{} Z:{} V:{} C:{})", 
                                     ccr,
                                     (ccr >> 3) & 1,
                                     (ccr >> 2) & 1,
                                     (ccr >> 1) & 1,
                                     ccr & 1));
                            ui.end_row();
                            
                            ui.label("SR:");
                            ui.monospace(format!("0x{:04X}", self.cpu.get_sr()));
                            ui.end_row();
                        });
                    });
                });
            });

        // Central Panel - Main Editor Area
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.show_compare_view && !self.machine_code.is_empty() {
                // Compare View (Assembly vs Bytecode) - VS Code merge style
                self.show_compare_editor(ui);
            } else {
                // Main Assembly Editor (full width when not comparing)
                self.show_assembly_editor(ui);
            }
        });

        // Keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::F5) {
                // F5 - Assemble & Run
                self.assemble_code();
                self.show_compare_view = true;
                if !self.machine_code.is_empty() {
                    self.run_program();
                }
            }
            
            if i.key_pressed(egui::Key::F9) {
                // F9 - Assemble only
                self.assemble_code();
                self.show_compare_view = true;
            }
            
            if i.key_pressed(egui::Key::F10) {
                // F10 - Step
                if !self.machine_code.is_empty() {
                    self.step_program();
                }
            }
            
            if i.modifiers.ctrl && i.key_pressed(egui::Key::R) {
                // Ctrl+R - Reset
                self.reset_emulator();
            }
        });

        // Auto-refresh während Emulation
        if self.is_running {
            ctx.request_repaint();
        }
    }
}

impl EmulatorApp {
    fn assemble_code(&mut self) {
        self.output_log.clear();
        self.error_message.clear();
        
        // Assembly-Code in Zeilen aufteilen und assemblieren
        let lines: Vec<&str> = self.assembly_code
            .lines()
            .map(|line| {
                // Kommentare entfernen (alles nach ';')
                line.split(';').next().unwrap_or("").trim()
            })
            .filter(|line| !line.is_empty())
            .collect();
        
        self.machine_code = self.assembler.assemble(&lines);
        
        if self.machine_code.is_empty() {
            self.error_message = "Assembly fehlgeschlagen! Keine Instruktionen generiert.".to_string();
            return;
        }

        // Maschinenbefehle in Speicher laden
        for (address, instruction) in &self.machine_code {
            self.memory.write_word(*address, *instruction);
        }
        
        self.output_log.push_str("✅ Assembly erfolgreich!\n");
        self.output_log.push_str(&format!("📊 {} Instruktionen generiert\n\n", self.machine_code.len()));
        
        // Assembly Listing anzeigen
        self.assembler.print_assembly_to_string(&mut self.output_log);
        
        self.reset_emulator();
    }
    
    fn run_program(&mut self) {
        if !self.step_mode {
            self.is_running = true;
            // Kontinuierliche Ausführung (würde in echtem Code begrenzt werden)
            for _ in 0..100 { // Maximal 100 Schritte zur Sicherheit
                if self.cpu.get_pc() >= (self.machine_code.len() as u32 * 2) {
                    break;
                }
                self.step_program();
            }
            self.is_running = false;
        } else {
            // Im Step Mode nur einen Schritt ausführen
            self.step_program();
        }
    }
    
    fn step_program(&mut self) {
        if self.cpu.get_pc() >= (self.machine_code.len() as u32 * 2) {
            self.output_log.push_str("🛑 Programm beendet (PC außerhalb des Codes)\n");
            return;
        }
        
        let old_pc = self.cpu.get_pc();
        self.cpu.execute_instruction(&mut self.memory);
        self.current_step += 1;
        
        self.output_log.push_str(&format!(
            "Step {}: PC 0x{:06X} → 0x{:06X}\n", 
            self.current_step, old_pc, self.cpu.get_pc()
        ));
    }
    
    fn reset_emulator(&mut self) {
        self.cpu.reset();
        self.current_step = 0;
        self.is_running = false;
        self.output_log.push_str("🔄 Emulator zurückgesetzt\n");
    }
    
    fn show_assembly_editor(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("📝 Assembly Editor");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.show_compare_view {
                    if ui.button("📝 Editor View").clicked() {
                        self.show_compare_view = false;
                    }
                }
            });
        });
        
        ui.separator();
        
        // Large Assembly Editor (VS Code style)
        let available_height = ui.available_height() - 50.0;
        
        egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_sized(
                    [ui.available_width(), available_height],
                    egui::TextEdit::multiline(&mut self.assembly_code)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(30)
                        .lock_focus(true)
                );
            });
    }
    
    fn show_compare_editor(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("🔍 Compare View");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌ Close Compare").clicked() {
                    self.show_compare_view = false;
                }
                
                if ui.button("📝 Editor View").clicked() {
                    self.show_compare_view = false;
                }
            });
        });
        
        ui.separator();
        
        // Split view like VS Code merge conflicts
        ui.horizontal(|ui| {
            // Left side - Assembly Code
            ui.vertical(|ui| {
                ui.heading("📄 Assembly Source");
                ui.separator();
                
                let available_height = ui.available_height() - 20.0;
                
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        // Show assembly with line numbers and syntax highlighting
                        self.show_assembly_with_highlighting(ui);
                    });
            });
            
            ui.separator();
            
            // Right side - Machine Code
            ui.vertical(|ui| {
                ui.heading("🔢 Machine Code");
                ui.separator();
                
                let available_height = ui.available_height() - 20.0;
                
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .show(ui, |ui| {
                        self.show_machine_code_detailed(ui);
                    });
            });
        });
    }
    
    fn show_assembly_with_highlighting(&mut self, ui: &mut egui::Ui) {
        let lines: Vec<&str> = self.assembly_code.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            ui.horizontal(|ui| {
                // Line number (VS Code style)
                ui.label(
                    egui::RichText::new(format!("{:3}", line_num + 1))
                        .color(egui::Color32::GRAY)
                        .monospace()
                );
                
                // Assembly line with basic syntax highlighting
                if line.trim().is_empty() {
                    ui.label(" ");
                } else if line.trim_start().starts_with(';') {
                    // Comment - green
                    ui.label(
                        egui::RichText::new(*line)
                            .color(egui::Color32::from_rgb(106, 153, 85))
                            .monospace()
                    );
                } else if line.contains(':') {
                    // Label - bright yellow (VS Code style)
                    ui.label(
                        egui::RichText::new(*line)
                            .color(egui::Color32::from_rgb(255, 215, 0))
                            .monospace()
                    );
                } else {
                    // Check for instruction highlighting
                    self.highlight_instruction(ui, line);
                }
            });
        }
    }
    
    fn highlight_instruction(&self, ui: &mut egui::Ui, line: &str) {
        // Split line into instruction and operands, preserving comments
        let comment_pos = line.find(';');
        let (code_part, comment_part) = if let Some(pos) = comment_pos {
            (&line[..pos], Some(&line[pos..]))
        } else {
            (line, None)
        };
        
        let parts: Vec<&str> = code_part.split_whitespace().collect();
        
        if !parts.is_empty() {
            let instruction = parts[0].to_uppercase();
            
            // Instruction mnemonic - light blue (VS Code keyword color)
            let instr_color = match instruction.as_str() {
                "MOVEQ" | "MOVE" => egui::Color32::from_rgb(86, 156, 214),  // Blue
                "ADD" | "SUB" | "CMP" => egui::Color32::from_rgb(78, 201, 176), // Cyan
                "BRA" | "BEQ" | "BNE" | "BCC" | "BCS" => egui::Color32::from_rgb(197, 134, 192), // Purple
                "NOP" => egui::Color32::from_rgb(156, 220, 254), // Light blue
                _ => egui::Color32::from_rgb(220, 220, 220), // Default light gray
            };
            
            ui.label(
                egui::RichText::new(&instruction)
                    .color(instr_color)
                    .monospace()
                    .strong()
            );
            
            // Operands with syntax highlighting
            if parts.len() > 1 {
                let operands = parts[1..].join(" ");
                self.highlight_operands(ui, &operands);
            }
        }
        
        // Comment - green (VS Code comment color)
        if let Some(comment) = comment_part {
            ui.label(
                egui::RichText::new(comment)
                    .color(egui::Color32::from_rgb(106, 153, 85))
                    .monospace()
            );
        }
    }
    
    fn highlight_operands(&self, ui: &mut egui::Ui, operands: &str) {
        ui.label(" "); // Space between instruction and operands
        
        // Simple operand highlighting
        for part in operands.split(',') {
            let part = part.trim();
            
            let color = if part.starts_with('#') {
                // Immediate values - orange
                egui::Color32::from_rgb(181, 206, 168)
            } else if part.starts_with('D') && part.len() == 2 {
                // Data registers - light green
                egui::Color32::from_rgb(156, 220, 254)
            } else if part.starts_with('A') && part.len() == 2 {
                // Address registers - light blue
                egui::Color32::from_rgb(156, 220, 254)
            } else {
                // Labels or other - yellow
                egui::Color32::from_rgb(255, 215, 0)
            };
            
            ui.label(
                egui::RichText::new(part)
                    .color(color)
                    .monospace()
            );
            
            // Add comma if not the last part
            if operands.rfind(',').map_or(false, |pos| pos > operands.find(part).unwrap_or(0)) {
                ui.label(
                    egui::RichText::new(", ")
                        .color(egui::Color32::WHITE)
                        .monospace()
                );
            }
        }
    }
    
    fn show_machine_code_detailed(&self, ui: &mut egui::Ui) {
        egui::Grid::new("machine_code_grid")
            .striped(true)
            .show(ui, |ui| {
                // Header
                ui.strong("Address");
                ui.strong("Machine Code");
                ui.strong("Binary");
                ui.strong("Instruction");
                ui.end_row();
                
                for (address, instruction) in &self.machine_code {
                    let current_marker = if *address == self.cpu.get_pc() { "►" } else { " " };
                    
                    // Address with current PC marker
                    ui.label(
                        egui::RichText::new(format!("{} 0x{:06X}", current_marker, address))
                            .monospace()
                            .color(if *address == self.cpu.get_pc() { 
                                egui::Color32::YELLOW 
                            } else { 
                                egui::Color32::WHITE 
                            })
                    );
                    
                    // Machine code
                    ui.label(
                        egui::RichText::new(format!("0x{:04X}", instruction))
                            .monospace()
                            .color(egui::Color32::from_rgb(181, 206, 168))
                    );
                    
                    // Binary representation
                    ui.label(
                        egui::RichText::new(format!("{:016b}", instruction))
                            .monospace()
                            .color(egui::Color32::GRAY)
                    );
                    
                    // Decoded instruction (if available)
                    ui.label(
                        egui::RichText::new(self.decode_instruction(*instruction))
                            .monospace()
                            .color(egui::Color32::from_rgb(206, 145, 120))
                    );
                    
                    ui.end_row();
                }
            });
    }
    
    fn decode_instruction(&self, instruction: u16) -> String {
        let opcode = (instruction >> 12) & 0xF;
        
        match opcode {
            0x7 => {
                let reg = (instruction >> 9) & 0x7;
                let immediate = (instruction & 0xFF) as i8;
                format!("MOVEQ #{}, D{}", immediate, reg)
            }
            0x3 => {
                let dest_reg = (instruction >> 9) & 0x7;
                let src_reg = instruction & 0x7;
                format!("MOVE D{}, D{}", src_reg, dest_reg)
            }
            0xD => {
                let dest_reg = (instruction >> 9) & 0x7;
                let src_reg = instruction & 0x7;
                format!("ADD D{}, D{}", src_reg, dest_reg)
            }
            0xB => {
                let dest_reg = (instruction >> 9) & 0x7;
                let src_reg = instruction & 0x7;
                format!("CMP D{}, D{}", src_reg, dest_reg)
            }
            0x6 => {
                let condition = (instruction >> 8) & 0xF;
                let displacement = (instruction & 0xFF) as i8;
                let condition_name = match condition {
                    0x0 => "BRA",
                    0x7 => "BEQ",
                    0x6 => "BNE",
                    _ => "Bcc",
                };
                format!("{} {:+}", condition_name, displacement)
            }
            0x4 => {
                if instruction == 0x4E71 {
                    "NOP".to_string()
                } else {
                    format!("MISC 0x{:04X}", instruction)
                }
            }
            _ => format!("UNK 0x{:04X}", instruction),
        }
    }
}