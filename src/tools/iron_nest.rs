use eframe::egui;
use super::GameTool;
use crate::input::KeyEvent;

#[derive(Default)]
pub struct IronNest {
    capturing: bool,
    buffer: String,
    last_key: Option<KeyEvent>,
}

impl GameTool for IronNest {
    fn name(&self) -> &'static str {
        "Iron Nest — Elevation"
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Iron Nest — Elevation");
        ui.label("Press Num Enter to start entering a distance in km.");
        ui.separator();
        ui.label(if self.capturing { "● CAPTURING" } else { "○ idle" });
        ui.monospace(if self.buffer.is_empty() { "—" } else { self.buffer.as_str() });
        match self.last_key {
            Some(k) => ui.weak(format!("last key: {k:?}")),
            None => ui.weak("no global key events yet"),
        };
    }

    fn handle_key(&mut self, key: KeyEvent) {
        self.last_key = Some(key);
        match key {
            KeyEvent::NumEnter => {
                self.capturing = !self.capturing;
                if self.capturing {
                    self.buffer.clear();
                }
            }
            KeyEvent::NumDigit(d) if self.capturing => self.buffer.push((b'0' + d) as char),
            KeyEvent::NumDecimal if self.capturing => self.buffer.push('.'),
            KeyEvent::Backspace if self.capturing => {
                self.buffer.pop();
            }
            _ => {}
        }
    }

    fn wants_capture(&self) -> bool {
        self.capturing
    }
}