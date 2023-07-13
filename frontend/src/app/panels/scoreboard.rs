use eframe::egui;
use egui::{
    plot::{Corner, Legend, Line, MarkerShape, Plot, PlotBounds, Points},
    Vec2,
};

use itertools::Itertools;

use crate::app::ClientState;

#[derive(Default)]
pub struct ScoreboardPanel {}

impl ScoreboardPanel {
    fn name(&self) -> &'static str {
        "Scoreboard"
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, ctf_state: &ClientState) {
        if let Some(global_state) = &ctf_state.ctf_state.global_data {
            // Store the lowest time solve. The CTF will "Start 20 minutes
            // before that" for now. Later, we can add a "Start at" field to the
            // CTF in the database.
            // TODO: this ^
            let lowest_time = global_state
                .scoreboard
                .teams
                .values()
                .filter_map(|solves| solves.iter().map(|s| s.time_millis).min())
                .min()
                .unwrap_or(0);

            // The latest time a solve was submitted
            let highest_time = global_state
                .scoreboard
                .teams
                .values()
                .filter_map(|solves| solves.iter().map(|s| s.time_millis).max())
                .max()
                .unwrap_or(0);

            // The team with the max score
            let max_team_score = global_state
                .scoreboard
                .teams
                .values()
                .map(|solves| solves.iter().map(|s| s.points).sum::<u32>())
                .max()
                .unwrap_or(0);

            Plot::new("custom_axes")
                .legend(Legend::default().position(Corner::RightBottom))
                // .width(400.0)
                // .height(200.0)
                // .center_y_axis(true)
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false)
                .allow_boxed_zoom(false)
                .allow_double_click_reset(false)
                .show_x(true)
                .set_margin_fraction(Vec2 { x: 0.1, y: 0.1 })
                .auto_bounds_x()
                .auto_bounds_y()
                // .view_aspect(0.2)
                // .include_x(0.0)
                // .include_x(10.0)
                // .include_y(0.0)
                // .include_y(1000.0)
                // .clamp_grid(true)
                .label_formatter(|name, value| {
                    let mut format_string = String::new();
                    // If the name is has something, add it to the string first,
                    // followed by a newline
                    // TODO: Why isn't this working?
                    if !name.is_empty() {
                        format_string.push_str(&format!("{}\n", name));
                    }
                    // Next, add the time and point data
                    format_string.push_str(&format!(
                        "{:.0} points\n{:02}:{:02}:{:02}",
                        value.y,
                        (value.x / 60.0).floor(),
                        (value.x % 60.0).floor(),
                        (value.x % 1.0 * 60.0).floor()
                    ));

                    format_string
                })
                .show(ui, |plot_ui| {
                    for (team_name, solves) in &global_state.scoreboard.teams {
                        // Iterate over this team's scores. Make sure to sort them by
                        // time. The time is stored in milliseconds since the epoch, so
                        // translate it to minutes.

                        // A team's line of score
                        plot_ui.line(Line::new(
                            solves
                                .iter()
                                .sorted_by(|a, b| a.time_millis.cmp(&b.time_millis))
                                .fold((0, vec![[0.0, 0.0]]), |mut acc, s| {
                                    acc.0 += s.points;
                                    acc.1.push([
                                        (s.time_millis - lowest_time) as f64 / 1000.0 / 60.0,
                                        acc.0 as f64,
                                    ]);
                                    acc
                                })
                                .1,
                        ));

                        // A team's points for each score
                        plot_ui.points(
                            Points::new(
                                solves
                                    .iter()
                                    .sorted_by(|a, b| a.time_millis.cmp(&b.time_millis))
                                    .fold((0, vec![[0.0, 0.0]]), |mut acc, s| {
                                        acc.0 += s.points;
                                        acc.1.push([
                                            (s.time_millis - lowest_time) as f64 / 1000.0 / 60.0,
                                            acc.0 as f64,
                                        ]);
                                        acc
                                    })
                                    .1,
                            )
                            .name(team_name)
                            .filled(true)
                            .radius(3.0)
                            .shape(MarkerShape::Circle),
                        );
                    }

                    let time_diff = (highest_time - lowest_time) as f64 / 1000.0 / 60.0;

                    let border = 10.0;

                    let mut minimums = [
                        0.0 - time_diff / border,
                        0.0 - max_team_score as f64 / border,
                    ];

                    let mut maximums = [
                        time_diff + time_diff / border,
                        max_team_score as f64 + max_team_score as f64 / border,
                    ];

                    // If both the minimums and maximums are 0, then we should
                    // just use bounds of [0.0, 0.0], [100, 100]
                    if minimums[0] == 0.0
                        && minimums[1] == 0.0
                        && maximums[0] == 0.0
                        && maximums[1] == 0.0
                    {
                        minimums = [0.0, 0.0];
                        maximums = [100.0, 100.0];
                    }

                    plot_ui.set_plot_bounds(PlotBounds::from_min_max(minimums, maximums));
                });
        }
    }
}
