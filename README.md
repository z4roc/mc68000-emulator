# MC68000 Emulator 🖥️

Kleines Projet für das Modul Rechnertechnik aus dem 3. Semester
Ein lehrreicher MC68000-Prozessor-Emulator in Rust mit grafischer Benutzeroberfläche.

## Features ✨

### Core Emulator
- ✅ **Vollständige MC68000-CPU-Emulation** (Register, Flags, PC)
- ✅ **16MB Speicher-System** (Big-Endian wie Original)
- ✅ **Assembly-Parser** für lesbaren Code
- ✅ **Fetch-Decode-Execute Zyklus**

### Unterstützte Instruktionen
- ✅ **MOVEQ** - Quick Move (8-bit immediate)
- ✅ **MOVE** - Daten-Transfer zwischen Registern
- ✅ **ADD** - Addition
- ✅ **SUB/CMP** - Subtraktion/Vergleich
- ✅ **Branch-Instruktionen** (BEQ, BNE, BRA, etc.)
- ✅ **NOP** - No Operation
- ✅ **Label-Support** für Sprungziele

### GUI-Features
- 🎨 **Assembly-Editor** mit Syntax-Hervorhebung
- 🔍 **Register-Viewer** (D0-D7, A0-A7, PC, CCR, SR)
- 💾 **Machine-Code-Anzeige** mit aktueller Position
- ⏯️ **Step-by-Step Debugging**
- 📊 **Output-Log** für Ausführungsschritte
- 🔄 **Reset-Funktion**

## Installation & Ausführung

### Voraussetzungen
```bash
# Rust installieren
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Kompilieren & Starten
```bash
# Repository klonen
git clone <repository-url>
cd mc68000

# GUI-Version starten
cargo run --bin mc68000-gui

# CLI-Version starten  
cargo run --bin mc68000-cli
```

## Verwendung 📚

### Assembly-Programmierung
```assembly
MOVEQ #42, D0    ; Lade 42 in D0
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
BRA end          ; Endlos-Loop
```

### Bedienung
1. **Assembly-Code** in den Editor eingeben
2. **"Assemble"** klicken → Kompiliert zu Maschinencode
3. **"Step"** für schrittweise Ausführung 
4. **"Run"** für kontinuierliche Ausführung
5. **"Reset"** für Neustart

## Architektur 🏗️

```
├── src/
│   ├── cpu.rs          # MC68000 CPU-Emulation
│   ├── memory.rs       # 16MB Speicher-System
│   ├── assembler.rs    # Assembly → Machine Code Parser
│   ├── gui.rs          # egui GUI-Interface
│   ├── main.rs         # CLI-Version
│   └── main_gui.rs     # GUI-Version
```

### MC68000 Instruktionsformat
```
MOVEQ #42, D0 → 0x702A
┌─────┬─────┬─────────┐
│0111 │000  │00101010 │
│Opcd │Reg  │Immediate│
└─────┴─────┴─────────┘
```

## Lernziele 🎓

Dieses Projekt demonstriert:
- **Prozessor-Architektur** (Fetch-Decode-Execute)
- **Assembly-Programmierung** 
- **Maschinencode-Generierung**
- **Compiler-Techniken** (Parser, AST)
- **GUI-Entwicklung** in Rust
- **Emulation-Techniken**

## Entwicklung 🚧

### Nächste Features
- [ ] Mehr Instruktionen (OR, AND, Shifts)
- [ ] Address Register Indirect Modes
- [ ] Interrupt-Handling
- [ ] Exception-Behandlung
- [ ] Memory-Viewer
- [ ] Breakpoints
- [ ] VS Code Extension

### Bekannte Einschränkungen
- Vereinfachte Addressing Modes
- Keine Privileged Instructions
- Kein Timing-genaues Verhalten
- Begrenzte Exception-Behandlung

## Technologie-Stack 🛠️

- **Rust** - Systemsprache für Performance & Sicherheit
- **egui** - Immediate Mode GUI Framework  
- **eframe** - Application Framework
- **Cargo** - Build-System & Package Manager

## Lizenz 📄

Dieses Projekt ist für Bildungszwecke entwickelt.

---

*Entwickelt als Übungsprojekt basierend auf Vorlesungsmaterial zum MC68000-Prozessor* 📖 