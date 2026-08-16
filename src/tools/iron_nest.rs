use super::GameTool;
use crate::input::KeyEvent;
use eframe::egui;

const FACTOR: f64 = 0.012;
const CHARGES: [u32; 6] = [1, 2, 3, 4, 5, 6];
const MIN_ELEVATION: f64 = 15.0;
const MAX_ELEVATION: f64 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Cell {
    Ok(f64),
    TooClose,
    OutOfRange,
}

impl Cell {
    fn from_elevation(deg: f64) -> Self {
        if deg > MAX_ELEVATION {
            Cell::OutOfRange
        } else if deg < MIN_ELEVATION {
            Cell::TooClose
        } else {
            Cell::Ok(deg)
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Elevations {
    km: f64,
    elevations: [f64; 6],
}

impl Elevations {
    fn from_km(km: f64) -> Self {
        let m = km * 1000.0;
        let elevations = CHARGES.map(|n| m * FACTOR / n as f64);
        Self { km, elevations }
    }
}

#[derive(Debug, Default)]
enum Mode {
    #[default]
    Idle,
    Listening {
        buffer: String,
    },
}

#[derive(Default)]
pub struct IronNest {
    mode: Mode,
    result: Option<Elevations>,
    last_error: Option<String>,
}

impl IronNest {
    fn begin_listening(&mut self) {
        self.mode = Mode::Listening {
            buffer: String::new(),
        };
        self.last_error = None;
    }

    fn finish_listening(&mut self) {
        let Mode::Listening { buffer } = std::mem::take(&mut self.mode) else {
            return;
        };
        match buffer.parse::<f64>() {
            Ok(km) if km.is_finite() => {
                self.result = Some(Elevations::from_km(km));
                self.last_error = None;
            }
            _ if buffer.is_empty() => {
                self.last_error = None;
            }
            _ => {
                self.last_error = Some(format!("Couldn't read \"{buffer}\" as a number"));
            }
        }
    }
}

impl GameTool for IronNest {
    fn name(&self) -> &'static str {
        "Iron Nest — Elevation"
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        // ---- status / input line -------------------------------------------
        match &self.mode {
            Mode::Idle => {
                ui.horizontal(|ui| {
                    ui.label("○");
                    ui.label("Press Num Enter to enter a distance (km).");
                });
            }
            Mode::Listening { buffer } => {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(230, 80, 80), "●");
                    ui.label("Listening — type distance in km, Num Enter to finish");
                });
                let shown = if buffer.is_empty() {
                    "_"
                } else {
                    buffer.as_str()
                };
                ui.label(
                    egui::RichText::new(format!("{shown} km"))
                        .monospace()
                        .size(30.0)
                        .strong(),
                );
            }
        }

        if let Some(err) = &self.last_error {
            ui.colored_label(egui::Color32::from_rgb(230, 160, 60), err);
        }

        ui.separator();

        let Some(res) = self.result else {
            ui.weak("No result yet.");
            return;
        };

        ui.horizontal(|ui| {
            ui.label(format!("Distance: {:.2} km", res.km));
        });
        ui.add_space(6.0);

        let spacing = 10.0;
        let col_w = ((ui.available_width() - 2.0 * spacing) / 3.0).max(80.0);

        egui::Grid::new("iron_nest_elevations")
            .num_columns(3)
            .spacing([spacing, spacing])
            .min_col_width(col_w)
            .max_col_width(col_w)
            .show(ui, |ui| {
                for (i, (n, elev)) in CHARGES.iter().zip(res.elevations).enumerate() {
                    egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin::symmetric(8, 6))
                        .show(ui, |ui| {
                            ui.set_width(col_w - 16.0);
                            ui.vertical_centered(|ui| {
                                ui.small(format!("{n} charge/s"));
                                let text = match Cell::from_elevation(elev) {
                                    Cell::Ok(deg) => egui::RichText::new(format!("{deg:.2}°"))
                                        .size(26.0)
                                        .strong(),
                                    Cell::TooClose => egui::RichText::new("Too Close")
                                        .size(20.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(230, 160, 60)),
                                    Cell::OutOfRange => egui::RichText::new("Out of Range")
                                        .size(20.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(230, 80, 80)),
                                };
                                ui.label(text);
                            });
                        });
                    if i % 3 == 2 {
                        ui.end_row();
                    }
                }
            });
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match (&mut self.mode, key) {
            (Mode::Idle, KeyEvent::NumEnter) => self.begin_listening(),
            (Mode::Listening { .. }, KeyEvent::NumEnter) => self.finish_listening(),

            (Mode::Listening { buffer }, KeyEvent::NumDigit(d)) => {
                buffer.push((b'0' + d) as char);
            }
            (Mode::Listening { buffer }, KeyEvent::NumDecimal) => {
                if !buffer.contains('.') {
                    buffer.push('.');
                }
            }
            (Mode::Listening { buffer }, KeyEvent::Backspace) => {
                buffer.pop();
            }

            _ => {}
        }
    }

    fn wants_capture(&self) -> bool {
        matches!(self.mode, Mode::Listening { .. })
    }
}
