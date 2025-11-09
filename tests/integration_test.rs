// Integration tests for MC68000 emulator
use mc68000::{Assembler, Memory, CPU};

#[test]
fn test_power_of_two_calculation() {
    // Test: Calculate 2^8 = 256
    let assembly = r#"
            ORG     $0800
N_VALUE:    DC.L    8         
RESULT:     DS.L    1           

            ORG     $1000

START:      MOVE.L  #1, D0         
            MOVEA.L #N_VALUE, A0  
            MOVE.L  (A0), D1       
            CMP.L   #0, D1
            BEQ     DONE          

LOOP:       MULS    #2, D0        
            SUBQ.L  #1, D1        
            BNE     LOOP         

DONE:       MOVEA.L #RESULT, A1   
            MOVE.L  D0, (A1)       
            SIMHALT
    "#;

    let (mut cpu, mut memory) = assemble_and_load(assembly);

    // PC should start at $1000 (first instruction), not $0800 (data)
    assert_eq!(cpu.get_pc(), 0x1000, "PC should start at first instruction");

    // Memory at $0800 should contain 8
    assert_eq!(memory.read_long(0x0800), 8, "N_VALUE should be 8");

    // Run program
    run_until_halt(&mut cpu, &mut memory, 1000);

    // Check results
    assert_eq!(cpu.get_data_register(0), 256, "D0 should contain 256 (2^8)");
    assert_eq!(cpu.get_data_register(1), 0, "D1 should be 0 after loop");

    // Memory at $0804 (RESULT) should contain 256
    assert_eq!(memory.read_long(0x0804), 256, "RESULT should be 256");
}

#[test]
fn test_move_immediate_to_register() {
    let assembly = r#"
            ORG     $1000
            MOVE.L  #42, D0
            SIMHALT
    "#;

    let (mut cpu, mut memory) = assemble_and_load(assembly);
    run_until_halt(&mut cpu, &mut memory, 10);

    assert_eq!(cpu.get_data_register(0), 42, "D0 should be 42");
}

#[test]
fn test_movea_indirect_addressing() {
    let assembly = r#"
            ORG     $0800
DATA:       DC.L    123

            ORG     $1000
            MOVEA.L #DATA, A0
            MOVE.L  (A0), D0
            SIMHALT
    "#;

    let (mut cpu, mut memory) = assemble_and_load(assembly);
    run_until_halt(&mut cpu, &mut memory, 10);

    assert_eq!(
        cpu.get_address_register(0),
        0x0800,
        "A0 should point to DATA"
    );
    assert_eq!(cpu.get_data_register(0), 123, "D0 should be 123");
}

#[test]
fn test_muls_multiplication() {
    let assembly = r#"
            ORG     $1000
            MOVE.L  #5, D0
            MULS    #3, D0
            SIMHALT
    "#;

    let (mut cpu, mut memory) = assemble_and_load(assembly);
    run_until_halt(&mut cpu, &mut memory, 10);

    assert_eq!(cpu.get_data_register(0), 15, "5 * 3 should be 15");
}

#[test]
fn test_subq_operation() {
    let assembly = r#"
            ORG     $1000
            MOVE.L  #10, D1
            SUBQ.L  #1, D1
            SUBQ.L  #2, D1
            SIMHALT
    "#;

    let (mut cpu, mut memory) = assemble_and_load(assembly);
    run_until_halt(&mut cpu, &mut memory, 10);

    assert_eq!(cpu.get_data_register(1), 7, "10 - 1 - 2 = 7");
}

#[test]
fn test_cmpi_beq_branch() {
    let assembly = r#"
            ORG     $1000
            MOVE.L  #5, D0
            CMP.L   #5, D0
            BEQ     EQUAL
            MOVE.L  #99, D1
            SIMHALT
EQUAL:      MOVE.L  #42, D1
            SIMHALT
    "#;

    let (mut cpu, mut memory) = assemble_and_load(assembly);
    run_until_halt(&mut cpu, &mut memory, 10);

    assert_eq!(cpu.get_data_register(1), 42, "Should branch to EQUAL");
}

#[test]
fn test_bne_loop() {
    let assembly = r#"
            ORG     $1000
            MOVE.L  #0, D0
            MOVE.L  #3, D1
LOOP:       SUBQ.L  #1, D1
            BNE     LOOP
            SIMHALT
    "#;

    let (mut cpu, mut memory) = assemble_and_load(assembly);
    run_until_halt(&mut cpu, &mut memory, 20);

    assert_eq!(cpu.get_data_register(1), 0, "D1 should be 0 after loop");
}

#[test]
fn test_indirect_write() {
    let assembly = r#"
            ORG     $0800
BUFFER:     DS.L    1

            ORG     $1000
            MOVEA.L #BUFFER, A0
            MOVE.L  #777, D0
            MOVE.L  D0, (A0)
            SIMHALT
    "#;

    let (mut cpu, mut memory) = assemble_and_load(assembly);
    run_until_halt(&mut cpu, &mut memory, 10);

    assert_eq!(memory.read_long(0x0800), 777, "BUFFER should contain 777");
}

// Helper functions

fn assemble_and_load(assembly_code: &str) -> (CPU, Memory) {
    let mut assembler = Assembler::new();
    let lines: Vec<&str> = assembly_code.lines().collect();
    let machine_code = assembler.assemble(&lines);

    let mut memory = Memory::new();
    let mut cpu = CPU::new();

    // Write all machine code (including data) to memory
    for (address, word) in &machine_code {
        memory.write_word(*address, *word);
    }

    // Find first instruction address (skip data section)
    // Instructions are at addresses >= $1000 in our test programs
    let first_instruction_addr = machine_code
        .iter()
        .find(|(addr, _)| *addr >= 0x1000)
        .map(|(addr, _)| *addr)
        .unwrap_or(0x1000);

    cpu.set_pc(first_instruction_addr);

    (cpu, memory)
}

fn run_until_halt(cpu: &mut CPU, memory: &mut Memory, max_steps: usize) {
    let mut steps = 0;
    let initial_pc = cpu.get_pc();

    while steps < max_steps {
        let pc_before = cpu.get_pc();
        cpu.execute_instruction(memory);
        let pc_after = cpu.get_pc();

        steps += 1;

        // SIMHALT detected: PC doesn't change
        if pc_before == pc_after {
            break;
        }

        // Prevent infinite loops
        if steps >= max_steps {
            panic!("Program did not halt within {} steps", max_steps);
        }
    }
}
