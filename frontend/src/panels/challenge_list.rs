use common::ctf_message::{CTFChallenge, CTFState, GameData};
use eframe::egui;
use egui::{epaint::ahash::HashMap, ScrollArea};
use egui_extras::{Column, TableBuilder};
use itertools::Itertools;

use crate::app::ClientState;

pub struct ChallengePanel {
    enabled: bool,
    visible: bool,
}

impl Default for ChallengePanel {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
        }
    }
}

impl ChallengePanel {
    fn name(&self) -> &'static str {
        "Challenges"
    }

    pub fn show(&mut self, ctx: &egui::Context, ctf_state: &ClientState) {
        egui::Window::new(self.name())
            // .open(open)
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                self.ui(ui, ctf_state);
            });
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctf_state: &ClientState) {
        ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
            // If we're logged in, show all the challenges
            if let GameData::LoggedIn { challenges } = &ctf_state.ctf_state.game_data {
                // Sort the challenges into categories
                let sorted_challenges: HashMap<String, Vec<CTFChallenge>> =
                    challenges
                        .iter()
                        .fold(HashMap::default(), |mut acc, challenge| {
                            acc.entry(challenge.category.clone())
                                .or_insert_with(Vec::new)
                                .push(challenge.clone());
                            acc
                        });

                // Show the challenges. Show the categories sorted by name. Use
                // a header for each category name, then show the challenges in
                // that category using a label. Sort the challenges by point
                // value.
                for (category, challenges) in
                    sorted_challenges.iter().sorted_by(|a, b| a.0.cmp(&b.0))
                {
                    ui.heading(category);
                    ui.separator();
                    let mut challenges = challenges.clone();
                    challenges.sort_by(|a, b| a.points.cmp(&b.points));
                    for challenge in challenges {
                        if ui
                            .button(format!("{} ({} points)", challenge.title, challenge.points))
                            .clicked()
                        {}
                    }
                }
            }
        });
    }
}
