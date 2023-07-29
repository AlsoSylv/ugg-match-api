use crate::structs::{self, MatchSummary, OverallRanking, RankScore};
use crate::{
    get_icon, match_summaries, player_info, player_ranking, player_ranks, player_suggestions,
    update_player, Errors,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use eframe::egui::{self, RichText, TextureOptions, Vec2};
use eframe::epaint::ColorImage;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use tokio::runtime::{Builder, Runtime};

pub enum Results {
    MatchSum(Result<structs::PlayerMatchSummeries, Errors>),
    // PlayerSuggestions(Result<structs::PlayerSuggestions, Errors>),
    PlayerUpdate(Result<structs::UpdatePlayer, Errors>),
    ProfileRanks(Result<structs::PlayerRank, Errors>),
    Ranking(Result<structs::PlayerRanking, Errors>),
    PlayerInfo(Result<structs::PlayerInfo, Errors>),
    PlayerIcon(Result<bytes::Bytes, Errors>),
}

pub struct MyEguiApp {
    tx: Sender<Results>,
    rx: Receiver<Results>,

    name: String,
    active_player: String,
    role: String,
    roles_map: HashMap<String, Option<i64>>,
    roles_reversed: HashMap<i64, String>,
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
            active_player: Default::default(),
            role: "None".to_owned(),
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

    fn update_matches(&self, ctx: &egui::Context) {
        let role = &self.role;
        if let Some(role) = self.roles_map.get(role as &str) {
            match_summaries(
                self.active_player.clone(),
                self.tx.clone(),
                ctx.clone(),
                *role,
                self.client.clone(),
                self.runtime.handle(),
            );
        } else {
            match_summaries(
                self.active_player.clone(),
                self.tx.clone(),
                ctx.clone(),
                None,
                self.client.clone(),
                self.runtime.handle(),
            );
        };
        player_ranks(
            self.active_player.clone(),
            self.tx.clone(),
            ctx.clone(),
            self.client.clone(),
            self.runtime.handle(),
        );
        player_ranking(
            self.active_player.clone(),
            self.tx.clone(),
            ctx.clone(),
            self.client.clone(),
            self.runtime.handle(),
        );
        player_info(
            self.active_player.clone(),
            self.tx.clone(),
            ctx.clone(),
            self.client.clone(),
            self.runtime.handle(),
        )
    }

    /// Note: This is unsused because the searchbar is broken, but I'm hoping it gets fixed one day
    #[allow(unused)]
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

    fn update_player(&self, ctx: &egui::Context) {
        update_player(
            self.active_player.clone(),
            self.tx.clone(),
            ctx.clone(),
            self.client.clone(),
            self.runtime.handle(),
        );
        self.update_matches(ctx);
    }

    fn player_search_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let search_bar = ui.text_edit_singleline(&mut self.name);
        search_bar.request_focus();
        if search_bar.changed() {
            // self.update_player_suggestion(ctx);
            if self.name.is_empty() {
                self.active_player = String::new();
            }
        };

        if search_bar.clicked() {
            if !self.name.is_empty() {
                self.active_player = self.name.clone();
                self.update_matches(ctx);
            } else {
                self.summeries = None;
                self.active_player = String::new();
            }
        }
    }

    fn update_data(&mut self, ctx: &egui::Context) {
        if let Ok(receiver) = self.rx.try_recv() {
            match receiver {
                Results::MatchSum(match_sums) => match match_sums {
                    Ok(matches) => {
                        let a = &matches.data.fetch_player_match_summaries.match_summaries;
                        self.summeries = Some(a.clone());
                    }
                    Err(err) => {
                        println!("{:?}", err);
                        // self.summeries = None;
                    }
                },
                // Results::PlayerSuggestions(players) => match players {
                //     Ok(players) => {
                //         self.players = Some(players);
                //     }
                //     Err(err) => {
                //         println!("{:?}", err);
                //         self.players = None;
                //     }
                // },
                Results::PlayerUpdate(update) => match update {
                    Ok(updated) => {
                        let data = updated.data.update_player_profile;
                        if data.success {
                            self.update_matches(ctx);
                        } else {
                            println!("{:?}", data.error_reason);
                        }
                    }
                    Err(err) => {
                        println!("{:?}", err)
                    }
                },
                Results::ProfileRanks(rank) => match rank {
                    Ok(rank) => {
                        let data = rank.data.fetch_profile_ranks.rank_scores;
                        self.rank = Some(data[0].clone());
                    }
                    Err(err) => {
                        println!("{:?}", err)
                    }
                },
                Results::Ranking(ranking) => match ranking {
                    Ok(ranking) => {
                        self.ranking = ranking.data.overall_ranking;
                    }
                    Err(err) => {
                        println!("{:?}", err)
                    }
                },
                Results::PlayerInfo(info) => match info {
                    Ok(info) => {
                        if let Some(data) = info.data.profile_player_info {
                            let id = data.icon_id;

                            get_icon(
                                id,
                                self.tx.clone(),
                                self.client.clone(),
                                self.runtime.handle(),
                            );
                        }
                    }
                    Err(err) => {
                        println!("{:?}", err)
                    }
                },
                Results::PlayerIcon(data) => match data {
                    Ok(icon) => {
                        let mut decoder = png::Decoder::new(&*icon);

                        let x = decoder.read_header_info().unwrap().height;
                        let y = decoder.read_header_info().unwrap().width;

                        let mut reader = decoder.read_info().unwrap();

                        let mut buf = vec![0; reader.output_buffer_size()];

                        reader.next_frame(&mut buf).unwrap();

                        let texture = ctx.load_texture(
                            "icon",
                            ColorImage::from_rgb([x as usize, y as usize], &buf),
                            TextureOptions::LINEAR,
                        );
                        let _ = self.player_icon.replace(texture);
                    }
                    Err(err) => {
                        println!("{:?}", err)
                    }
                },
            }
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_data(ctx);

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);

                ui.add_space(5.0);

                let reset_button = ui.button("Reset GUI");

                if reset_button.clicked() {
                    self.name = Default::default();
                    self.role = "None".to_owned();
                    self.summeries = None;
                    self.ranking = None;
                    self.rank = None;
                    self.active_player = Default::default();
                };

                reset_button.on_hover_ui(|ui| {
                    ui.label("Reset GUI To Defaults");
                });
            });
        });

        egui::SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    let texture: &eframe::epaint::TextureHandle =
                        self.player_icon.get_or_insert_with(|| {
                            let bytes: &[u8] = include_bytes!("../0.png");
                            let mut png = png::Decoder::new(bytes);

                            let headers = png.read_header_info().unwrap();
                            let x = headers.height as usize;
                            let y= headers.width as usize;

                            let mut reader = png.read_info().unwrap();
                            let mut buf = vec![0; reader.output_buffer_size()];
                            reader.next_frame(&mut buf).unwrap();

                            ctx.load_texture(
                                "none",
                                ColorImage::from_rgb(
                                    [x, y],
                                    &buf,
                                ),
                                TextureOptions::LINEAR,
                            )
                        });

                    ui.image(texture, Vec2::new(100.0, 100.0));

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label("Roles: ");
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
                        ui.label(
                            RichText::new("Player: ").color(egui::Color32::from_rgb(255, 0, 0)),
                        );
                        egui::ComboBox::from_id_source("player_suggestions")
                            .selected_text(self.name.clone())
                            .show_ui(ui, |ui| {
                                self.player_search_bar(ui, ctx);
                            });
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        let button = ui
                            .add_enabled(self.refresh_enabled, egui::Button::new("Refresh Player"));

                        if button.clicked() {
                            self.update_matches(ctx);
                        };

                        ui.add_space(5.0);

                        let button =
                            ui.add_enabled(self.update_enabled, egui::Button::new("Update Player"));

                        if button.clicked() {
                            self.update_player(ctx);
                        };
                    });

                    if !self.active_player.is_empty() {
                        self.refresh_enabled = true;
                        self.update_enabled = true;
                    } else {
                        self.refresh_enabled = false;
                        self.update_enabled = false;
                    };

                    ui.separator();

                    ui.label(format!("Summoner: {}", self.active_player));
                    if let Some(rank) = &self.rank {
                        if rank.tier.is_empty() {
                            ui.label("Unranked");
                            ui.label("LP: None");
                            ui.label(format!("Wins: {}", rank.wins));
                            ui.label(format!("Losses: {}", rank.losses));
                            ui.label(format!("Queue: {}", rank.queue_type));
                        } else {
                            ui.label(format!("Rank: {} {}", rank.tier, rank.rank));
                            ui.label(format!("LP: {}", rank.lp));
                            ui.label(format!("Wins: {}", rank.wins));
                            ui.label(format!("Losses: {}", rank.losses));
                            ui.label(format!("Queue: {}", rank.queue_type));
                            if let Some(ranking) = &self.ranking {
                                ui.label(format!("Ranking: {}", ranking.overall_ranking));
                                ui.label(format!(
                                    "Top: {:.1$}%",
                                    (ranking.overall_ranking * 100) as f64
                                        / (ranking.total_player_count) as f64,
                                    1
                                ));
                            }
                        }
                    };
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.add_space(5.0);

                egui::ScrollArea::vertical()
                    .max_height(ui.available_height())
                    .show(ui, |ui| {
                        if let Some(summeries) = &self.summeries {
                            if summeries.is_empty() {
                                ui.label("No Data For Summoner");
                            } else {
                                for (num, summary) in summeries.iter().enumerate() {
                                    self.match_page(summary, ui);
                                    if num != summeries.len() - 1 {
                                        ui.separator();
                                    }
                                }
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
