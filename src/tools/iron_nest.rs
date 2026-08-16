use eframe::egui;
use super::GameTool;
use crate::input::KeyEvent;

#[derive(Default)]
pub struct IronNest {
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
        match self.last_key {
            Some(k) => ui.label(format!("Last global key: {k:?}")),
            None => ui.weak("No global key events yet."),
        };
    }

    fn handle_key(&mut self, key: KeyEvent) {
        self.last_key = Some(key);
    }
}