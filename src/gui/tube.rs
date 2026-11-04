// The display tube: draws 32 rows of 32 bits the way the Baby's monitor did.
// A dash is a 1, a dot is a 0, and the smallest bit is on the left.

use eframe::egui::{self, Color32, Stroke};

use crate::memory::{ROWS, Row};

// The tube glowed green on a dark screen.
const BEZEL: Color32 = Color32::from_rgb(28, 30, 30);
const SCREEN: Color32 = Color32::from_rgb(6, 16, 10);
const GLOW: Color32 = Color32::from_rgb(140, 255, 170);
const HALO: Color32 = Color32::from_rgba_premultiplied(30, 70, 40, 60);
const DIM: Color32 = Color32::from_rgb(40, 90, 55);
// Modern helper only: marks the row picked by the line switches.
const MARKER: Color32 = Color32::from_rgba_premultiplied(60, 50, 10, 60);

// Draws the tube at the given size, showing the given 32 rows.
// `highlight` marks one row, used by the modern helper that shows the selected line.
pub fn draw(ui: &mut egui::Ui, rows: &[Row; ROWS], highlight: Option<usize>, size: f32) {
    let (area, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let painter = ui.painter_at(area);

    // The metal surround, then the rounded glass screen inside it.
    painter.rect_filled(area, 14.0, BEZEL);
    let screen = area.shrink(12.0);
    painter.rect_filled(screen, 40.0, SCREEN);

    // Split the screen into a 32 by 32 grid, leaving a gap around the edge.
    let grid = screen.shrink(26.0);
    let cell = grid.width() / 32.0;

    for (row_number, row) in rows.iter().enumerate() {
        let row_top = grid.top() + row_number as f32 * cell;

        if highlight == Some(row_number) {
            let band = egui::Rect::from_min_size(
                egui::pos2(grid.left() - 4.0, row_top),
                egui::vec2(grid.width() + 8.0, cell),
            );
            painter.rect_filled(band, 2.0, MARKER);
        }

        for bit in 0..32 {
            let centre = egui::pos2(
                grid.left() + bit as f32 * cell + cell / 2.0,
                row_top + cell / 2.0,
            );

            if (row >> bit) & 1 == 1 {
                // Dash: a short horizontal line, with a soft halo behind it like phosphor glow.
                let half = cell * 0.32;
                let ends = [
                    centre - egui::vec2(half, 0.0),
                    centre + egui::vec2(half, 0.0),
                ];
                painter.line_segment(ends, Stroke::new(6.0, HALO));
                painter.line_segment(ends, Stroke::new(2.5, GLOW));
            } else {
                // Dot: a small dim point.
                painter.circle_filled(centre, 1.4, DIM);
            }
        }
    }
}
