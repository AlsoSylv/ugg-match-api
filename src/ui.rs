use crate::structs::{self, MatchSummary, OverallRanking, RankScore};
use crate::{match_summaries, player_info, player_ranking, player_ranks, update_player, Errors};
use chrono::{DateTime, NaiveDateTime, Utc};
use eframe::egui::{self, Label, TextureOptions, Vec2};
use eframe::epaint::ColorImage;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use tokio::runtime::{Builder, Runtime};

pub enum Results {
    MatchSum(Result<structs::PlayerMatchSummeries, Errors>),
    PlayerUpdate(Result<structs::UpdatePlayer, Errors>),
    ProfileRanks(Result<structs::PlayerRank, Errors>),
    Ranking(Result<structs::PlayerRanking, Errors>),
    PlayerInfo(Result<structs::PlayerInfo, Errors>),
    PlayerIcon(Result<bytes::Bytes, Errors>),
}

/// TODO: Replace the hashmaps with bimaps
/// TODO: Store player data in a sub struct
pub struct MyEguiApp {
    tx: Sender<Results>,
    rx: Receiver<Results>,

    active_player: String,
    role: u8,
    roles_map: HashMap<String, i8>,
    roles_reversed: HashMap<i8, String>,
    summeries: Option<Vec<MatchSummary>>,
    rank: Option<RankScore>,
    ranking: Option<OverallRanking>,
    player_icon: Option<egui::TextureHandle>,

    refresh_enabled: bool,
    update_enabled: bool,

    client: reqwest::Client,
    runtime: Runtime,
}

static ROLES: [&str; 6] = ["Top", "Jungle", "Mid", "ADC", "Support", "None"];

impl MyEguiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let runtime = Builder::new_multi_thread()
            .worker_threads(2)
            .enable_io()
            .enable_time()
            .build()
            .unwrap();

        let (tx, rx) = std::sync::mpsc::channel();
        // Todo: Remove explicit None case?
        let roles_map = HashMap::from([
            ("Top".to_owned(), 4),
            ("Jungle".to_owned(), 1),
            ("Mid".to_owned(), 5),
            ("ADC".to_owned(), 3),
            ("Support".to_owned(), 2),
        ]);

        // Todo: replace with an array
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
            active_player: Default::default(),
            role: 5,
            roles_map,
            roles_reversed,
            summeries: None,
            rank: None,
            ranking: None,
            refresh_enabled: false,
            update_enabled: false,
            client,
            player_icon: None,
            runtime,
        }
    }

    /// This long line of function calls, well looking like bullshit
    /// actually drives the entire state of the GUI to change!
    fn update_matches(&self, ctx: &egui::Context) {
        let name = Arc::new(self.active_player.clone());

        match_summaries(
            name.clone(),
            self.tx.clone(),
            ctx.clone(),
            self.roles_map.get(ROLES[self.role as usize]).copied(),
            self.client.clone(),
            self.runtime.handle(),
        );
        player_ranks(
            name.clone(),
            self.tx.clone(),
            ctx.clone(),
            self.client.clone(),
            self.runtime.handle(),
        );
        player_ranking(
            name.clone(),
            self.tx.clone(),
            ctx.clone(),
            self.client.clone(),
            self.runtime.handle(),
        );
        player_info(
            name.clone(),
            self.tx.clone(),
            ctx.clone(),
            self.client.clone(),
            self.runtime.handle(),
        );
    }

    fn match_page(&self, summary: &MatchSummary, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(format!(
            "{} {}",
            summary.champion_id, self.roles_reversed[&summary.role]
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

    fn update_player(&self, ctx: &egui::Context) {
        let name = Arc::new(self.active_player.clone());

        update_player(
            name.clone(),
            self.tx.clone(),
            ctx.clone(),
            self.client.clone(),
            self.runtime.handle(),
        );
        self.update_matches(ctx);
    }

    fn update_data(&mut self, ctx: &egui::Context) {
        if !self.active_player.is_empty() {
            self.refresh_enabled = true;
            self.update_enabled = true;
        } else {
            self.refresh_enabled = false;
            self.update_enabled = false;
        };

        if let Ok(receiver) = self.rx.try_recv() {
            match receiver {
                Results::MatchSum(match_sums) => match match_sums {
                    Ok(matches) => {
                        self.summeries =
                            Some(matches.data.fetch_player_match_summaries.match_summaries)
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::PlayerUpdate(update) => match update {
                    Ok(updated) => {
                        let data = updated.data.update_player_profile;
                        if data.success {
                            self.update_matches(ctx);
                        } else {
                            dbg!("{:?}", data.error_reason);
                        }
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::ProfileRanks(rank) => match rank {
                    Ok(rank) => {
                        let mut data = rank.data.fetch_profile_ranks.rank_scores;
                        self.rank = Some(data.remove(0));
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::Ranking(ranking) => match ranking {
                    Ok(ranking) => {
                        self.ranking = ranking.data.overall_ranking;
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                // Todo: Display this info
                Results::PlayerInfo(info) => match info {
                    Ok(_) => {}
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::PlayerIcon(data) => match data {
                    Ok(icon) => {
                        let mut decoder = png::Decoder::new(&*icon);
                        let headers = decoder
                            .read_header_info()
                            .expect("This is always a PNG, so this shouldn't ever fail");

                        let x = headers.height as usize;
                        let y = headers.width as usize;

                        let mut reader = decoder.read_info().unwrap();

                        let mut buf = vec![0; reader.output_buffer_size()];

                        reader.next_frame(&mut buf).unwrap();

                        let texture = ctx.load_texture(
                            "icon",
                            ColorImage::from_rgb([x, y], &buf),
                            TextureOptions::LINEAR,
                        );
                        let _ = self.player_icon.insert(texture);
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
            }
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_data(ctx);

        egui::SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.label("Player: ");

                        let search_bar = ui.text_edit_singleline(&mut self.active_player);

                        if search_bar.clicked() {
                            if !self.active_player.is_empty() {
                                self.update_matches(ctx);
                            } else {
                                self.summeries = None;
                            }
                        }

                        if search_bar.changed() {
                            self.summeries = None;
                            self.player_icon = None;
                        }
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label("Roles: ");

                        egui::ComboBox::from_id_source("roles")
                            .selected_text(ROLES[self.role as usize])
                            .show_ui(ui, |ui| {
                                for (index, text) in ROLES.iter().enumerate() {
                                    ui.selectable_value(&mut self.role, index as u8, *text);
                                }
                            });
                    });

                    ui.add_space(5.0);

                    let button =
                        ui.add_enabled(self.refresh_enabled, egui::Button::new("Refresh Player"));

                    if button.clicked() {
                        self.update_matches(ctx);
                    };

                    ui.add_space(5.0);

                    let button =
                        ui.add_enabled(self.update_enabled, egui::Button::new("Update Player"));

                    if button.clicked() {
                        self.update_player(ctx);
                    };

                    egui::TopBottomPanel::bottom("bottom_panel").show_inside(ui, |ui| {
                        ui.add_space(5.0);
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            egui::widgets::global_dark_light_mode_switch(ui);
                        });
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    if self.summeries.is_some() {
                        if let Some(texture) = &self.player_icon {
                            ui.image(texture, Vec2::splat(50.0));
                        } else if !self.active_player.is_empty() {
                            ui.spinner();
                        }

                        if let Some(rank) = &self.rank {
                            let (lol_rank, lp, ranking) = if rank.tier.is_empty() {
                                (
                                    Label::new("Unranked"),
                                    Label::new("LP: None"),
                                    Label::new("Ranking: None"),
                                )
                            } else {
                                (
                                    Label::new(format!("Wins: {}", rank.wins)),
                                    Label::new(format!("Losses: {}", rank.losses)),
                                    if let Some(ranking) = &self.ranking {
                                        Label::new(format!(
                                            "Ranking: {} / {}",
                                            ranking.overall_ranking, ranking.total_player_count
                                        ))
                                    } else {
                                        Label::new("Ranking: None")
                                    },
                                )
                            };

                            ui.vertical(|ui| {
                                ui.add(lol_rank);
                                ui.add(lp);
                                ui.label(format!("Queue: {}", rank.queue_type));
                            });

                            ui.separator();

                            ui.vertical(|ui| {
                                ui.label(format!("Wins: {}", rank.wins));
                                ui.label(format!("Losses: {}", rank.losses));
                                ui.add(ranking)
                            });
                        };
                    }
                });

                ui.add_space(5.0);

                if self.summeries.is_some() {
                    ui.separator();
                }

                ui.add_space(5.0);

                egui::ScrollArea::vertical()
                    .max_height(ui.available_height())
                    .show(ui, |ui| {
                        if let Some(summeries) = &self.summeries {
                            if summeries.is_empty() {
                                ui.label("No Data For Summoner");
                            } else {
                                summeries.iter().enumerate().for_each(|(index, summary)| {
                                    self.match_page(summary, ui);
                                    if index != summeries.len() - 1 {
                                        ui.separator();
                                    }
                                });
                            }
                        } else if self.summeries.is_none() && !self.active_player.is_empty() {
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
