pub mod iron_nest;

use crate::input::KeyEvent;

pub trait GameTool {
    fn name(&self) -> &'static str;
    fn ui(&mut self, ui: &mut eframe::egui::Ui);
    fn handle_key(&mut self, _key: KeyEvent) {}
    fn wants_capture(&self) -> bool {
        false
    }
}

pub struct ToolEntry {
    pub name: &'static str,
    pub build: fn() -> Box<dyn GameTool>,
}

pub const REGISTRY: &[ToolEntry] = &[ToolEntry {
    name: "Iron Nest — Elevation",
    build: || Box::new(iron_nest::IronNest::default()),
}];