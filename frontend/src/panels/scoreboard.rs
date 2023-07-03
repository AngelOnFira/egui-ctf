use std::f64::consts::TAU;

use common::{
    ctf_message::{CTFMessage, TeamData},
    NetworkMessage,
};
use eframe::egui;
use egui::{
    plot::{Corner, Legend, Line, Plot},
    remap,
};
use egui_extras::{Column, TableBuilder};
use itertools::Itertools;

use crate::app::{ClientState, ConnectionState};

pub struct ScoreboardPanel {}

impl Default for ScoreboardPanel {
    fn default() -> Self {
        Self {}
    }
}

impl ScoreboardPanel {
    fn name(&self) -> &'static str {
        "Scoreboard"
    }

    pub fn show(&mut self, ctx: &egui::Context, ctf_state: &ClientState) {
        egui::Window::new(self.name())
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                self.ui(ui, ctf_state);
            });
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctf_state: &ClientState) {
        let n = 100;
        let mut team_scores = Vec::new();

        if let Some(global_state) = &ctf_state.ctf_state.global_data {
            // Store the lowest time solve. The CTF will "Start 20 minutes
            // before that" for now. Later, we can add a "Start at" field to the
            // CTF in the database.
            // TODO: this ^
            let lowest_time = global_state
                .scoreboard
                .teams
                .iter()
                .map(|(_, solves)| solves.iter().map(|s| s.time).min())
                .flatten()
                .min()
                .unwrap_or(0);

            for (team_name, solves) in &global_state.scoreboard.teams {
                // Iterate over this team's scores. Make sure to sort them by
                // time. The time is stored in milliseconds since the epoch, so
                // translate it to minutes.
                let mut score_values: Vec<[f64; 2]> = solves
                    .iter()
                    .sorted_by(|a, b| a.time.cmp(&b.time))
                    .map(|s| {
                        [
                            (s.time - lowest_time) as f64 / 1000.0 / 60.0,
                            s.points as f64,
                        ]
                    })
                    .collect();

                // Add a point to the start at (0, 0) so that the line starts
                // there.
                score_values.insert(0, [0.0, 0.0]);

                let line = Line::new(score_values);

                team_scores.push(line);
            }
        }

        Plot::new("custom_axes")
            .legend(Legend::default().position(Corner::RightBottom))
            .width(400.0)
            .height(200.0)
            .show(ui, |plot_ui| {
                // plot_ui.line(CustomAxisDemo::logistic_fn());
                for line in team_scores {
                    plot_ui.line(line.name("Line with fill"));
                }
            })
            .response;
    }
}
