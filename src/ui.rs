use crate::structs::{self, MatchSummary, PlayerSuggestions};
use crate::{match_summaries, player_suggestions, Errors};
use chrono::{DateTime, NaiveDateTime, Utc};
use eframe::egui;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

pub enum Results {
    MatchSum(Result<structs::PlayerMatchSummeries, Errors>),
    PlayerSuggestions(Result<structs::PlayerSuggestions, Errors>),
}

pub struct MyEguiApp {
    tx: Sender<Results>,
    rx: Receiver<Results>,

    name: String,
    role: String,
    roles_map: HashMap<String, Option<i64>>,
    roles_reversed: HashMap<i64, String>,
    players: Option<PlayerSuggestions>,
    summeries: Option<Vec<MatchSummary>>,

    client: reqwest::Client,
}

static ROLES: [&str; 6] = ["Top", "Jungle", "Mid", "ADC", "Support", "None"];

impl Default for MyEguiApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let roles_map = HashMap::from([
            ("Top".to_owned(), Some(4)),
            ("Jungle".to_owned(), Some(1)),
            ("Mid".to_owned(), Some(5)),
            ("ADC".to_owned(), Some(3)),
            ("Support".to_owned(), Some(2)),
            ("None".to_owned(), None),
        ]);

        let roles_reversed = HashMap::from([
            (4, "Top".to_owned()),
            (1, "Jungle".to_owned()),
            (5, "Mid".to_owned()),
            (3, "ADC".to_owned()),
            (2, "Support".to_owned()),
            (6, "ARAM".to_owned()),
        ]);

        let client = reqwest::Client::new();

        Self {
            tx,
            rx,
            name: Default::default(),
            role: Default::default(),
            roles_map,
            roles_reversed,
            players: None,
            summeries: None,
            client,
        }
    }
}

impl MyEguiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn update_matches(&self, ctx: &egui::Context) {
        let role = &self.role;
        if let Some(role) = self.roles_map.get(role as &str) {
            match_summaries(
                self.name.clone(),
                self.tx.clone(),
                ctx.clone(),
                *role,
                self.client.clone(),
            );
        } else {
            match_summaries(
                self.name.clone(),
                self.tx.clone(),
                ctx.clone(),
                None,
                self.client.clone(),
            );
        };
    }

    fn update_player_suggestion(&self, ctx: &egui::Context) {
        player_suggestions(
            self.name.clone(),
            self.tx.clone(),
            ctx.clone(),
            self.client.clone(),
        );
    }

    fn match_page(&self, summary: &MatchSummary, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(format!(
            "{} {}",
            summary.champion_id,
            self.roles_reversed
                .get(&summary.role)
                .unwrap_or(&"None".to_owned())
        ))
        .id_source(summary.match_id)
        .show(ui, |ui| {
            ui.label(summary.champion_id.to_string());
            ui.label(format!(
                "Match duration: {}",
                format_time(summary.match_duration)
            ));
            ui.label(format!(
                "KDA: {}/{}/{}",
                summary.kills, summary.deaths, summary.assists
            ));
        });
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Ok(pain) = self.rx.try_recv() {
                match pain {
                    Results::MatchSum(match_sums) => match match_sums {
                        Ok(matches) => {
                            let a = &matches.data.fetch_player_match_summaries.match_summaries;
                            self.summeries = Some(a.clone());
                        }
                        Err(_err) => {
                            self.summeries = None;
                        }
                    },
                    Results::PlayerSuggestions(players) => match players {
                        Ok(players) => {
                            self.players = Some(players);
                        }
                        Err(_err) => {
                            self.players = None;
                        }
                    },
                }
            }

            ui.vertical(|ui| {
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Roles:");
                    egui::ComboBox::from_id_source("roles")
                        .selected_text(self.role.clone())
                        .show_ui(ui, |ui| {
                            for option in ROLES {
                                ui.selectable_value(&mut self.role, option.to_string(), option);
                            }
                        });
                });

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Player Suggestions:");
                    egui::ComboBox::from_id_source("players")
                        .selected_text(self.name.clone())
                        .show_ui(ui, |ui| {
                            let search_bar = ui.text_edit_singleline(&mut self.name);
                            search_bar.request_focus();
                            if search_bar.changed() {
                                self.update_player_suggestion(ctx);
                            };

                            if let Some(pain) = self.players.clone() {
                                let players = pain.data.player_info_suggestions;
                                for option in players {
                                    if ui
                                        .selectable_value(
                                            &mut self.name,
                                            option.summoner_name.clone().to_string(),
                                            option.summoner_name.clone(),
                                        )
                                        .clicked()
                                    {
                                        self.update_matches(ctx);
                                    };
                                }
                            }
                        });
                });

                ui.add_space(5.0);

                if ui.button("Update Player").clicked() {
                    self.update_matches(ctx);
                };

                ui.add_space(5.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(summeries) = &self.summeries {
                        if summeries.is_empty() {
                            ui.label("No Data For Summoner");
                        }

                        for summary in summeries {
                            self.match_page(summary, ui);
                            ui.separator();
                        }
                    } else {
                        ui.spinner();
                    }
                });
            });
        });
    }
}

fn format_time(match_time: i64) -> String {
    let native_time = NaiveDateTime::from_timestamp_opt(match_time, 0).unwrap();
    let time: DateTime<Utc> = DateTime::from_local(native_time, Utc);
    let human_time = time.format("%H:%M:%S");
    if human_time.to_string().split(':').collect::<Vec<&str>>()[0] == "00" {
        let human_time = time.format("%M:%S");
        human_time.to_string()
    } else {
        human_time.to_string()
    }
}
