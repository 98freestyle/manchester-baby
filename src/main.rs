// Manchester Baby emulator.
// Opens a window showing the machine.

mod cpu;
mod display;
mod gui;
mod memory;
mod programs;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Manchester Baby")
            .with_inner_size([1060.0, 820.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Manchester Baby",
        options,
        Box::new(|_| Ok(Box::new(gui::BabyApp::new()))),
    )
}
