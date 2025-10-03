mod cpu;
mod memory;
mod assembler;

fn main() {
    println!("Starting MC68000 Emulator...");
    let mut cpu = cpu::CPU::new();
    let mut memory = memory::Memory::new();

    cpu.reset();
    println!("CPU and Memory initialized.");

    // Assembly-Code definieren
    let assembly_program = [
        "MOVEQ #42, D0",    // Lade 42 in D0
        "MOVEQ #7, D1",     // Lade 7 in D1  
        "ADD D0, D1",       // D1 = D1 + D0 (7 + 42 = 49)
        "MOVEQ #49, D2",    // Lade erwartetes Ergebnis in D2
        "CMP D2, D1",       // Vergleiche D1 mit D2 (49)
        "BEQ success",      // Springe zu success wenn gleich
        "MOVEQ #-1, D0",    // Fehler: -1 in D0
        "BRA end",          // Springe zum Ende
        "success:",         // Label für Erfolg
        "MOVEQ #1, D0",     // Erfolg: 1 in D0
        "end:",             // Label für Ende
        "NOP",              // No Operation
        "BRA end",          // Endlos-Loop
    ];

    println!("\n=== Assembly-Code ===");
    for (i, line) in assembly_program.iter().enumerate() {
        println!("{:2}: {}", i + 1, line);
    }

    // Assembly-Code assemblieren  
    let mut assembler = assembler::Assembler::new();
    let machine_code = assembler.assemble(&assembly_program);
    
    // Assembly-Listing anzeigen
    println!();
    assembler.print_assembly();

    // Maschinenbefehle in Speicher laden
    for (address, instruction) in machine_code {
        memory.write_word(address, instruction);
    }

    println!("\nTestprogramm geladen. Starte Ausführung:");
    println!("Programm berechnet: 42 + 7 = 49 und prüft das Ergebnis\n");

    // Register vor Ausführung anzeigen
    cpu.print_registers();
    println!();

    // Schrittweise Ausführung (5 Instruktionen)
    for step in 1..=6 {
        println!("--- Schritt {} ---", step);
        cpu.execute_instruction(&mut memory);
        cpu.print_registers();
        println!();
        
        // Nach 6 Schritten stoppen (vermeidet infinite loop)
        if step == 6 {
            println!("Demo beendet (infinite loop erreicht)");
            break;
        }
    }
}

