// The window.
// Left side: the machine, 1:1 with the real Baby. Only controls it actually had.
// Right side: modern helpers the real Baby never had.

mod controls;
mod tube;

use std::time::{Duration, Instant};

use eframe::egui;

use crate::cpu::Cpu;
use crate::display;
use crate::memory::{self, Memory, ROWS, Row};
use crate::programs;

// The amended 1948 program runs 2.1 million instructions. At about 670 instructions a second
// it takes around 52 minutes, the same length as the original 1948 run.
const BABY_INSTRUCTIONS_PER_SECOND: f64 = 670.0;

// At "your CPU" speed, how long to keep running instructions before redrawing the window.
// Without a limit the window would freeze until the whole program finished.
const FULL_SPEED_TIME_PER_FRAME: Duration = Duration::from_millis(12);

const TUBE_SIZE: f32 = 440.0;

#[derive(PartialEq)]
enum Speed {
    Baby,
    YourCpu,
}

// Which tube the display shows. The real Baby had 3 monitor buttons to pick one.
#[derive(PartialEq)]
enum Monitor {
    Store,
    Accumulator,
    Control,
}

pub struct BabyApp {
    memory: Memory,
    cpu: Cpu,

    // The control panel, as on the real Baby.
    // Line switches: pick a row, in binary. Switch 0 is worth 1, switch 4 is worth 16.
    line_switches: [bool; 5],
    // Function switches: pick an instruction by hand, used in manual mode. Worth 1, 2, 4.
    function_switches: [bool; 3],
    // Manual/auto switch. Up is manual: instructions come from the line and function switches.
    // Down is auto: instructions come from memory.
    manual: bool,
    // Write/erase switch. Up is erase: typewriter buttons clear bits. Down is write: they set bits.
    erase: bool,
    // CS switch. Up is stop, down is run.
    cs_stop: bool,
    monitor: Monitor,

    // Modern helpers.
    speed: Speed,
    show_selected_line: bool,
    // How many instructions have run since the program was loaded.
    instructions_run: u64,
    // At Baby speed: when the last batch of instructions ran.
    last_tick: Instant,
    // At Baby speed: instructions that are due but haven't run yet.
    // Fractions carry over, so the long run speed comes out exact.
    instructions_owed: f64,
}

impl BabyApp {
    // Starts the machine with the 1948 program loaded.
    pub fn new() -> BabyApp {
        let mut memory = memory::empty();
        programs::load_1948(&mut memory);
        BabyApp {
            memory,
            cpu: Cpu::new(),
            line_switches: [false; 5],
            function_switches: [false; 3],
            manual: false,
            erase: false,
            cs_stop: true,
            monitor: Monitor::Store,
            speed: Speed::Baby,
            show_selected_line: true,
            instructions_run: 0,
            last_tick: Instant::now(),
            instructions_owed: 0.0,
        }
    }

    // The machine runs while the CS switch is down and the stop lamp is off.
    fn running(&self) -> bool {
        !self.cs_stop && !self.cpu.stopped
    }

    // The row picked by the line switches.
    fn selected_line(&self) -> usize {
        switches_as_number(&self.line_switches) as usize
    }

    // The instruction set on the switches: function switches for which instruction,
    // line switches for which row. Used in manual mode.
    fn switch_instruction(&self) -> Row {
        switches_as_number(&self.line_switches)
            | (switches_as_number(&self.function_switches) << 13)
    }

    // Runs one instruction: from memory in auto mode, from the switches in manual mode.
    fn run_one(&mut self) {
        if self.manual {
            let instruction = self.switch_instruction();
            self.cpu.execute(instruction, &mut self.memory);
        } else {
            self.cpu.step(&mut self.memory);
        }
        self.instructions_run = self.instructions_run + 1;
    }

    // Runs however many instructions are due since the last frame, at the chosen speed.
    fn run_for_this_frame(&mut self) {
        match self.speed {
            Speed::Baby => {
                let now = Instant::now();
                let seconds_passed = now.duration_since(self.last_tick).as_secs_f64();
                self.last_tick = now;

                self.instructions_owed =
                    self.instructions_owed + seconds_passed * BABY_INSTRUCTIONS_PER_SECOND;
                while self.instructions_owed >= 1.0 && self.running() {
                    self.run_one();
                    self.instructions_owed = self.instructions_owed - 1.0;
                }
            }
            Speed::YourCpu => {
                let started = Instant::now();
                while self.running() && started.elapsed() < FULL_SPEED_TIME_PER_FRAME {
                    // Checking the clock is slow compared to an instruction, so run a batch between checks.
                    for _ in 0..10_000 {
                        if !self.running() {
                            break;
                        }
                        self.run_one();
                    }
                }
            }
        }
    }

    // KC key: clears the stop lamp and runs one instruction. Does nothing while running.
    fn press_kc(&mut self) {
        if self.running() {
            return;
        }
        self.cpu.stopped = false;
        self.run_one();
    }

    // A typewriter button: sets or clears that bit in the row picked by the line switches.
    fn press_typewriter(&mut self, bit: usize) {
        if self.running() {
            return;
        }
        let line = self.selected_line();
        let mask: Row = 1 << bit;
        if self.erase {
            self.memory[line] = self.memory[line] & !mask;
        } else {
            self.memory[line] = self.memory[line] | mask;
        }
    }

    // What the tube shows, depending on the monitor button pressed.
    fn tube_rows(&self) -> [Row; ROWS] {
        match self.monitor {
            Monitor::Store => self.memory,
            // The accumulator is one row, repeated on every line.
            Monitor::Accumulator => [self.cpu.accumulator; ROWS],
            // While running, CI and PI alternate down the screen. Stopped, only CI is shown.
            Monitor::Control => {
                let mut rows = [self.cpu.counter; ROWS];
                if self.running() {
                    for line in (1..ROWS).step_by(2) {
                        rows[line] = self.cpu.present_instruction;
                    }
                }
                rows
            }
        }
    }

    // The machine: tube, monitor buttons and the control panel. Only what the real Baby had.
    fn draw_machine(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let highlight = if self.show_selected_line && self.monitor == Monitor::Store {
                Some(self.selected_line())
            } else {
                None
            };
            tube::draw(ui, &self.tube_rows(), highlight, TUBE_SIZE);

            ui.add_space(14.0);
            ui.allocate_ui_with_layout(
                egui::vec2(60.0, TUBE_SIZE),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.add_space(8.0);
                    controls::section_label(ui, "MONITOR");
                    ui.add_space(4.0);
                    let pressed = |monitor: &Monitor, which: Monitor| *monitor == which;
                    let store_on = pressed(&self.monitor, Monitor::Store);
                    if controls::key(
                        ui,
                        if store_on { "S  ◀" } else { "S" },
                        controls::RED_KEY,
                        true,
                    ) {
                        self.monitor = Monitor::Store;
                    }
                    let acc_on = pressed(&self.monitor, Monitor::Accumulator);
                    if controls::key(
                        ui,
                        if acc_on { "A  ◀" } else { "A" },
                        controls::RED_KEY,
                        true,
                    ) {
                        self.monitor = Monitor::Accumulator;
                    }
                    let control_on = pressed(&self.monitor, Monitor::Control);
                    if controls::key(
                        ui,
                        if control_on { "C  ◀" } else { "C" },
                        controls::RED_KEY,
                        true,
                    ) {
                        self.monitor = Monitor::Control;
                    }
                    ui.add_space(20.0);
                    controls::lamp(ui, "STOP", self.cpu.stopped);
                },
            );
        });

        ui.add_space(10.0);

        // Typewriter: 40 buttons, one per bit. Only 0 to 31 were connected.
        controls::section_label(ui, "TYPEWRITER");
        for first in [0, 20] {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for bit in first..first + 20 {
                    if controls::typewriter_key(ui, bit, bit < 32) {
                        self.press_typewriter(bit);
                    }
                }
            });
        }

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                controls::section_label(ui, "LINE");
                ui.horizontal(|ui| {
                    for (number, switch) in self.line_switches.iter_mut().enumerate() {
                        controls::switch(ui, switch, "", &number.to_string());
                    }
                });
            });
            ui.add_space(16.0);

            ui.vertical(|ui| {
                controls::section_label(ui, "FUNCTION");
                ui.horizontal(|ui| {
                    for (number, switch) in self.function_switches.iter_mut().enumerate() {
                        controls::switch(ui, switch, "", &(13 + number).to_string());
                    }
                });
            });
            ui.add_space(16.0);

            controls::switch(ui, &mut self.erase, "ERASE", "WRITE");
            ui.add_space(8.0);
            controls::switch(ui, &mut self.manual, "MAN", "AUTO");
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            controls::section_label(ui, "CLEAR");
            if controls::key(ui, "KLC", controls::BLACK_KEY, !self.running()) {
                let line = self.selected_line();
                self.memory[line] = 0;
            }
            if controls::key(ui, "KSC", controls::BLACK_KEY, !self.running()) {
                self.memory = memory::empty();
            }
            if controls::key(ui, "KAC", controls::BLACK_KEY, !self.running()) {
                self.cpu.accumulator = 0;
            }
            if controls::key(ui, "KCC", controls::BLACK_KEY, !self.running()) {
                self.cpu.counter = 0;
                self.cpu.present_instruction = 0;
                self.cpu.accumulator = 0;
            }
            // These three were never wired up on the real Baby.
            controls::key(ui, "KBC", controls::BLACK_KEY, false);
            controls::key(ui, "KEC", controls::BLACK_KEY, false);
            controls::key(ui, "KMC", controls::BLACK_KEY, false);

            ui.add_space(24.0);
            if controls::key(ui, "KC", controls::RED_KEY, !self.running()) {
                self.press_kc();
            }
            ui.add_space(8.0);
            controls::switch(ui, &mut self.cs_stop, "STOP", "RUN");
        });
    }

    // Modern helpers: speed, loading the program, and readouts in plain numbers and words.
    fn draw_helpers(&mut self, ui: &mut egui::Ui) {
        ui.heading("Modern helpers");
        ui.label(egui::RichText::new("Things the real Baby didn't have.").small());
        ui.add_space(8.0);

        ui.label("Speed");
        ui.radio_value(&mut self.speed, Speed::Baby, "Baby (about 670 a second)");
        ui.radio_value(
            &mut self.speed,
            Speed::YourCpu,
            "Your CPU (as fast as possible)",
        );
        ui.add_space(8.0);

        if ui.button("Load the 1948 program").clicked() {
            programs::load_1948(&mut self.memory);
            self.cpu = Cpu::new();
            self.instructions_run = 0;
            self.cs_stop = true;
        }
        ui.checkbox(
            &mut self.show_selected_line,
            "Highlight the row picked by the line switches",
        );
        ui.add_space(8.0);
        ui.separator();

        let line = self.selected_line();
        ui.monospace(format!("Line switches pick row: {}", line));
        if self.manual {
            ui.monospace(format!(
                "Switches say: {}",
                display::as_text(self.switch_instruction())
            ));
        }
        if self.cpu.stopped {
            ui.monospace(format!("Stopped at row {}", self.cpu.counter & 0b11111));
        } else if !self.manual {
            let next_row = (self.cpu.counter.wrapping_add(1) & 0b11111) as usize;
            ui.monospace(format!(
                "Next row: {} ({})",
                next_row,
                display::as_text(self.memory[next_row])
            ));
        }
        ui.monospace(format!("Accumulator: {}", self.cpu.accumulator));
        ui.monospace(format!("Counter (CI): {}", self.cpu.counter));
        ui.monospace(format!("Instructions run: {}", self.instructions_run));
        ui.monospace(format!(
            "Time on a real Baby: {}",
            as_minutes_and_seconds(self.instructions_run as f64 / BABY_INSTRUCTIONS_PER_SECOND)
        ));
        ui.add_space(8.0);
        ui.separator();

        ui.label("Memory in plain words");
        ui.label(
            egui::RichText::new("Every row read both ways: as an instruction, and as a number.")
                .small(),
        );
        ui.monospace(format!(
            "  {:>3}  {:<20} {}",
            "row", "as instruction", "as number"
        ));
        let next_row = (self.cpu.counter.wrapping_add(1) & 0b11111) as usize;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (row_number, row) in self.memory.iter().enumerate() {
                let pointer = if row_number == next_row && !self.cpu.stopped {
                    ">"
                } else {
                    " "
                };
                ui.monospace(format!(
                    "{} {:>2}  {:<20} {}",
                    pointer,
                    row_number,
                    display::as_text(*row),
                    row
                ));
            }
        });
    }
}

impl eframe::App for BabyApp {
    // Runs every time the window redraws.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.running() {
            self.run_for_this_frame();
            // Keep redrawing while running, so the tube updates live.
            ui.ctx().request_repaint();
        } else {
            // Start the Baby speed clock fresh, so time spent stopped doesn't count.
            self.last_tick = Instant::now();
            self.instructions_owed = 0.0;
        }

        egui::Panel::right("helpers")
            .exact_size(330.0)
            .show(ui, |ui| {
                self.draw_helpers(ui);
            });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Manchester Baby");
            ui.add_space(6.0);
            self.draw_machine(ui);
        });
    }
}

// Reads a row of switches as a binary number. The first switch is worth 1, the next 2, then 4, and so on.
fn switches_as_number(switches: &[bool]) -> Row {
    let mut number = 0;
    for (position, &up) in switches.iter().enumerate() {
        if up {
            number = number | (1 << position);
        }
    }
    number
}

// Formats a number of seconds as "3 min 12 s".
fn as_minutes_and_seconds(seconds: f64) -> String {
    let whole = seconds as u64;
    format!("{} min {} s", whole / 60, whole % 60)
}
