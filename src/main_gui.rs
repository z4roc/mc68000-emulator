// MC68000 Emulator - GUI Version
mod cpu;
mod memory;
mod assembler;
mod gui;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you want to see it, run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("MC68000 Emulator")
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "MC68000 Emulator",
        options,
        Box::new(|_cc| Ok(Box::new(gui::EmulatorApp::default()))),
    )
}