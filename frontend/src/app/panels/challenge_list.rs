use common::ctf_message::{CTFChallenge, GameData};
use eframe::egui;
use egui::{epaint::ahash::HashMap, ScrollArea};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::app::ClientState;

#[derive(Deserialize, Serialize, Default)]
pub struct ChallengeList {
    // Challenge that should be displayed
    pub visible_challenge: Option<String>,
}

impl ChallengeList {
    fn name(&self) -> &'static str {
        "Challenge"
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
                    sorted_challenges.iter().sorted_by(|a, b| a.0.cmp(b.0))
                {
                    ui.heading(category);
                    ui.separator();
                    let mut challenges = challenges.clone();
                    challenges.sort_by(|a, b| a.points.cmp(&b.points));
                    for challenge in challenges {
                        if ui
                            .button(format!("{} ({} points)", challenge.title, challenge.points))
                            .clicked()
                        {
                            self.visible_challenge = Some(challenge.title.clone());
                        }
                    }
                }
            }
        });
    }
}
