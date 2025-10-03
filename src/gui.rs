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
        }
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top Panel - Controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🖥️ MC68000 Emulator");
                
                ui.separator();
                
                if ui.button("🔧 Assemble").clicked() {
                    self.assemble_code();
                }
                
                ui.separator();
                
                if ui.button("▶️ Run").clicked() && !self.machine_code.is_empty() {
                    self.run_program();
                }
                
                if ui.button("⏸️ Step").clicked() && !self.machine_code.is_empty() {
                    self.step_program();
                }
                
                if ui.button("🔄 Reset").clicked() {
                    self.reset_emulator();
                }
                
                ui.separator();
                
                ui.checkbox(&mut self.step_mode, "Step Mode");
            });
        });

        // Left Panel - Assembly Editor
        egui::SidePanel::left("assembly_panel")
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("📝 Assembly Code");
                
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.assembly_code)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                        );
                    });

                ui.separator();

                // Machine Code Anzeige
                ui.heading("🔢 Machine Code");
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (address, instruction) in &self.machine_code {
                            let current_marker = if *address == self.cpu.get_pc() { "→ " } else { "   " };
                            ui.label(format!("{}0x{:06X}: 0x{:04X}", current_marker, address, instruction));
                        }
                    });
            });

        // Right Panel - CPU State & Memory
        egui::SidePanel::right("cpu_panel")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("🧠 CPU Registers");
                
                // Data Registers
                ui.group(|ui| {
                    ui.label("Data Registers:");
                    for i in 0..8 {
                        ui.horizontal(|ui| {
                            ui.label(format!("D{}:", i));
                            ui.label(format!("0x{:08X}", self.cpu.get_data_register(i)));
                        });
                    }
                });

                ui.separator();

                // Address Registers  
                ui.group(|ui| {
                    ui.label("Address Registers:");
                    for i in 0..8 {
                        ui.horizontal(|ui| {
                            ui.label(format!("A{}:", i));
                            ui.label(format!("0x{:08X}", self.cpu.get_address_register(i)));
                        });
                    }
                });

                ui.separator();

                // Special Registers
                ui.group(|ui| {
                    ui.label("Special Registers:");
                    ui.horizontal(|ui| {
                        ui.label("PC:");
                        ui.label(format!("0x{:08X}", self.cpu.get_pc()));
                    });
                    ui.horizontal(|ui| {
                        ui.label("CCR:");
                        let ccr = self.cpu.get_ccr();
                        ui.label(format!("0x{:02X} (N:{} Z:{} V:{} C:{})", 
                                 ccr,
                                 (ccr >> 3) & 1,
                                 (ccr >> 2) & 1,
                                 (ccr >> 1) & 1,
                                 ccr & 1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("SR:");
                        ui.label(format!("0x{:04X}", self.cpu.get_sr()));
                    });
                });
            });

        // Central Panel - Output & Logs
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📋 Output & Logs");
            
            // Error Messages
            if !self.error_message.is_empty() {
                ui.colored_label(egui::Color32::RED, &self.error_message);
                ui.separator();
            }
            
            // Output Log
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.output_log)
                            .font(egui::TextStyle::Monospace)
                    );
                });
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
}