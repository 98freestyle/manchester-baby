// Control panel parts: toggle switches, push keys, typewriter buttons and lamps.
// Each one is drawn by hand so it looks like a panel rather than a normal app.

use eframe::egui::{self, Align, Color32, FontId, Layout, Sense, Stroke, Ui};

const PLATE: Color32 = Color32::from_rgb(45, 47, 50);
const SLOT: Color32 = Color32::from_rgb(15, 15, 16);
const METAL: Color32 = Color32::from_rgb(205, 208, 212);
const TYPEWRITER: Color32 = Color32::from_rgb(125, 32, 36);
const DEAD_KEY: Color32 = Color32::from_rgb(55, 34, 36);
const LABEL: Color32 = Color32::from_rgb(190, 190, 185);
pub const RED_KEY: Color32 = Color32::from_rgb(185, 45, 40);
pub const BLACK_KEY: Color32 = Color32::from_rgb(88, 88, 94);
const LAMP_ON: Color32 = Color32::from_rgb(255, 70, 50);
const LAMP_OFF: Color32 = Color32::from_rgb(60, 25, 22);

// Width given to each switch or key, so they sit side by side like on a panel.
const CONTROL_WIDTH: f32 = 36.0;

// Lays something out with a small caption underneath, centred, in a fixed width column.
fn captioned<R>(ui: &mut Ui, caption: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.allocate_ui_with_layout(
        egui::vec2(CONTROL_WIDTH, 50.0),
        Layout::top_down(Align::Center),
        |ui| {
            let result = add(ui);
            ui.label(egui::RichText::new(caption).small().color(LABEL));
            result
        },
    )
    .inner
}

// A toggle switch. `up` is the lever position. Clicking flips it.
// `top` and `bottom` name what each position means (either can be empty).
// Returns true if it was flipped this frame.
pub fn switch(ui: &mut Ui, up: &mut bool, top: &str, bottom: &str) -> bool {
    let mut flipped = false;
    ui.allocate_ui_with_layout(
        egui::vec2(CONTROL_WIDTH, 76.0),
        Layout::top_down(Align::Center),
        |ui| {
            if !top.is_empty() {
                ui.label(egui::RichText::new(top).small().color(LABEL));
            }

            let (area, response) = ui.allocate_exact_size(egui::vec2(22.0, 38.0), Sense::click());
            if response.clicked() {
                *up = !*up;
                flipped = true;
            }

            let painter = ui.painter();
            painter.rect_filled(area, 4.0, PLATE);
            painter.rect_filled(
                egui::Rect::from_center_size(area.center(), egui::vec2(6.0, 24.0)),
                2.0,
                SLOT,
            );

            // The lever points from the pivot in the middle towards the knob.
            let knob_y = if *up {
                area.top() + 8.0
            } else {
                area.bottom() - 8.0
            };
            let knob = egui::pos2(area.center().x, knob_y);
            painter.line_segment([area.center(), knob], Stroke::new(4.0, METAL));
            painter.circle_filled(knob, 5.5, METAL);

            if !bottom.is_empty() {
                ui.label(egui::RichText::new(bottom).small().color(LABEL));
            }
        },
    );
    flipped
}

// A round push key with a caption. Keys that weren't wired up on the real Baby
// are drawn but can't be pressed. Returns true when clicked.
pub fn key(ui: &mut Ui, caption: &str, colour: Color32, connected: bool) -> bool {
    captioned(ui, caption, |ui| {
        let sense = if connected {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (area, response) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), sense);

        let held = response.is_pointer_button_down_on();
        let fill = if !connected {
            colour.gamma_multiply(0.45)
        } else if held {
            colour.gamma_multiply(0.7)
        } else {
            colour
        };
        let radius = if held { 10.0 } else { 11.0 };
        ui.painter()
            .circle(area.center(), radius, fill, Stroke::new(1.5, PLATE));

        if !connected {
            response.on_hover_text("Not connected on the real Baby");
            return false;
        }
        response.clicked()
    })
}

// One of the typewriter buttons, numbered with its bit position.
// Only buttons 0 to 31 were wired up on the real Baby. Returns true when clicked.
pub fn typewriter_key(ui: &mut Ui, number: usize, connected: bool) -> bool {
    let sense = if connected {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (area, response) = ui.allocate_exact_size(egui::vec2(21.0, 21.0), sense);

    let held = response.is_pointer_button_down_on();
    let colour = if !connected {
        DEAD_KEY
    } else if held {
        TYPEWRITER.gamma_multiply(0.7)
    } else {
        TYPEWRITER
    };
    let painter = ui.painter();
    painter.circle_filled(area.center(), 9.5, colour);
    painter.text(
        area.center(),
        egui::Align2::CENTER_CENTER,
        number.to_string(),
        FontId::proportional(9.0),
        if connected {
            Color32::WHITE
        } else {
            Color32::GRAY
        },
    );

    if !connected {
        response.on_hover_text("Not connected on the real Baby");
        return false;
    }
    response.clicked()
}

// A lamp with a caption, lit or not.
pub fn lamp(ui: &mut Ui, caption: &str, lit: bool) {
    captioned(ui, caption, |ui| {
        let (area, _) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), Sense::hover());
        let painter = ui.painter();
        if lit {
            painter.circle_filled(area.center(), 13.0, LAMP_ON.gamma_multiply(0.25));
        }
        painter.circle(
            area.center(),
            9.0,
            if lit { LAMP_ON } else { LAMP_OFF },
            Stroke::new(1.5, PLATE),
        );
    });
}

// A small heading for a group of controls.
pub fn section_label(ui: &mut Ui, text: &str) {
    ui.label(egui::RichText::new(text).small().strong().color(LABEL));
}
