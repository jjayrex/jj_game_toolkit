#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod input;
mod tools;

use eframe::egui;
use tools::{GameTool, REGISTRY};
use std::sync::atomic::Ordering;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("JJ's Game Tools")
            .with_inner_size([440.0, 360.0])
            .with_min_inner_size([320.0, 240.0])
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "JJ's Game Tools",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

enum Screen {
    Launcher,
    Tool(Box<dyn GameTool>),
}

struct App {
    screen: Screen,
    always_on_top: bool,
    input: input::InputHandle
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_zoom_factor(1.15);
        Self {
            screen: Screen::Launcher,
            always_on_top: true,
            input: input::start(cc.egui_ctx.clone())
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) -> bool {
        let mut go_back = false;
        let (in_tool, title): (bool, &'static str) = match &self.screen {
            Screen::Tool(t) => (true, t.name()),
            Screen::Launcher => (false, "JJ's Game Tools"),
        };

        egui::Panel::top("top_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if in_tool {
                    if ui.button("⬅ Tools").clicked() {
                        go_back = true;
                    }
                    ui.separator();
                }
                ui.strong(title);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .checkbox(&mut self.always_on_top, "Always On Top")
                        .changed()
                    {
                        let level = if self.always_on_top {
                            egui::WindowLevel::AlwaysOnTop
                        } else {
                            egui::WindowLevel::Normal
                        };
                        ui.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                    }
                });
            });
        });

        go_back
    }

    fn launcher_ui(ui: &mut egui::Ui) -> Option<Box<dyn GameTool>> {
        let mut chosen = None;

        ui.add_space(8.0);
        ui.label("Pick a tool:");
        ui.add_space(8.0);

        for entry in REGISTRY.iter() {
            let label = egui::RichText::new(format!("{}",entry.name)).size(18.0);
            let button = egui::Button::new(label).min_size(egui::vec2(ui.available_width(), 40.0));

            if ui.add(button).clicked() {
                chosen = Some((entry.build)());
            }
            ui.add_space(10.0);
        }

        chosen
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
        if self.top_bar(ui) || escape {
            self.screen = Screen::Launcher;
        }

        let capture = match &mut self.screen {
            Screen::Tool(tool) => {
                while let Ok(ev) = self.input.rx.try_recv() {
                    tool.handle_key(ev);
                }
                tool.wants_capture()
            }
            Screen::Launcher => {
                while self.input.rx.try_recv().is_ok() {}
                false
            }
        };
        self.input.capture.store(capture, Ordering::Relaxed);

        let mut picked = None;
        egui::CentralPanel::default().show(ui, |ui| match &mut self.screen {
            Screen::Launcher => picked = Self::launcher_ui(ui),
            Screen::Tool(tool) => tool.ui(ui),
        });
        if let Some(tool) = picked {
            self.screen = Screen::Tool(tool);
        }
    }
}
