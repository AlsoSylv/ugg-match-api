use crate::structs::{self, PlayerSuggestions};
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
    time: Option<String>,
    role: String,
    roles_map: HashMap<String, Option<i64>>,
    players: Option<PlayerSuggestions>,

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

        let client = reqwest::Client::new();

        Self {
            tx,
            rx,
            name: Default::default(),
            time: Default::default(),
            role: Default::default(),
            roles_map,
            players: None,
            client,
        }
    }
}

impl MyEguiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Ok(pain) = self.rx.try_recv() {
                if let Results::MatchSum(Ok(pain)) = &pain {
                    let a = &pain.data.fetch_player_match_summaries.match_summaries[0];
                    self.time = Some(format!(
                        "KDA: {}    KP: {}    TIME: {}",
                        format!("{}/{}/{}", a.kills, a.deaths, a.assists), format!("{}%", a.kill_participation), format_time(a.match_duration)
                    ));
                } else {
                    self.time = None;
                };
                if let Results::PlayerSuggestions(Ok(pain)) = &pain {
                    self.players = Some(pain.clone());
                } else {
                    self.players = None;
                }
            }

            ui.heading(&self.name);
            match &self.time {
                Some(a) => {
                    ui.heading(a);
                }
                None => {
                    ui.spinner();
                }
            }

            ui.horizontal(|ui| {
                egui::ComboBox::from_label(format!("Role: {}", self.role))
                    .selected_text(self.role.clone())
                    .show_ui(ui, |ui| {
                        for option in ROLES {
                            ui.selectable_value(&mut self.role, option.to_string(), option);
                        }
                    });

                if ui.button("WAAAA").clicked() {
                    let role = &self.role;
                    if let Some(role) = self.roles_map.get(role as &str) {
                        match_summaries(
                            self.name.clone(),
                            self.tx.clone(),
                            ctx.clone(),
                            *role,
                            self.client.clone(),
                        );
                    };
                }
            });
            ui.horizontal(|ui| {
                if ui.text_edit_singleline(&mut self.name).changed() {
                    player_suggestions(
                        self.name.clone(),
                        self.tx.clone(),
                        ctx.clone(),
                        self.client.clone(),
                    )
                };
                egui::ComboBox::from_label("Selected Player: ")
                    .selected_text(self.name.clone())
                    .show_ui(ui, |ui| {
                        if let Some(pain) = self.players.clone() {
                            for option in pain.data.player_info_suggestions {
                                ui.selectable_value(
                                    &mut self.name,
                                    option.summoner_name.clone().to_string(),
                                    option.summoner_name.clone(),
                                );
                            }
                        }
                    })
            })
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

