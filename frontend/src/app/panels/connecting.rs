use common::{ctf_message::CTFMessage, NetworkMessage};
use egui::{emath, epaint, pos2, vec2, Align2, Color32, Frame, Pos2, Rect, Stroke};
use wasm_timer::SystemTime;

use crate::app::ConnectionState;

/// Shows off one example of each major type of widget.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ConnectingPanel {}

impl ConnectingPanel {
    fn name(&self) -> String {
        // Get the current time in milliseconds using an instant
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let mut connecting_string = "Connecting".to_string();

        // Add zero to three dots after the connecting string depending on the
        // time
        let time_period = 1000;

        for _ in 0..((time / time_period) % 4) {
            connecting_string.push('.');
        }

        connecting_string
    }

    pub fn window(&mut self, ctx: &egui::Context) {
        egui::Window::new(self.name())
            .resizable(false)
            .movable(false)
            .collapsible(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }
}

impl ConnectingPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let color = if ui.visuals().dark_mode {
            Color32::from_additive_luminance(196)
        } else {
            Color32::from_black_alpha(240)
        };

        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            let time = ui.input(|i| i.time);

            let desired_size = ui.available_width() * vec2(1.0, 0.35);
            let (_id, rect) = ui.allocate_space(desired_size);

            let to_screen =
                emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);

            let mut shapes = vec![];

            for &mode in &[2, 3, 5] {
                let mode = mode as f64;
                let n = 120;
                let speed = 1.5;

                let points: Vec<Pos2> = (0..=n)
                    .map(|i| {
                        let t = i as f64 / (n as f64);
                        let amp = (time * speed * mode).sin() / mode;
                        let y = amp * (t * std::f64::consts::TAU / 2.0 * mode).sin();
                        to_screen * pos2(t as f32, y as f32)
                    })
                    .collect();

                let thickness = 10.0 / mode as f32;
                shapes.push(epaint::Shape::line(points, Stroke::new(thickness, color)));
            }

            ui.painter().extend(shapes);
        });
    }
}
