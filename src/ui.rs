use crate::structs::{self, ChampionJson, MatchSummary, OverallRanking, RankScore};
use crate::Errors;
use chrono::{DateTime, NaiveDateTime, Utc};
use eframe::egui::{self, TextBuffer, TextureOptions, Vec2};
use eframe::epaint::ColorImage;
use std::collections::HashMap;
use std::sync::Arc;

pub enum Results {
    MatchSum(Result<structs::PlayerMatchSummeries, Errors>),
    PlayerUpdate(Result<structs::UpdatePlayer, Errors>),
    ProfileRanks(Result<structs::PlayerRank, Errors>),
    Ranking(Result<structs::PlayerRanking, Errors>),
    PlayerInfo(Result<structs::PlayerInfo, Errors>),
    PlayerIcon(Result<bytes::Bytes, Errors>),
    Versions(Result<Vec<String>, Errors>),
    ChampJson(Result<ChampionJson, Errors>),
    ChampImage(Result<(bytes::Bytes, i64), Errors>),
}

pub struct Message {
    pub ctx: egui::Context,
    pub payload: Payload,
}

pub enum Payload {
    MatchSummaries { name: Arc<String>, roles: Vec<i8> },
    PlayerRanks { name: Arc<String> },
    UpdatePlayer { name: Arc<String> },
    PlayerRanking { name: Arc<String> },
    PlayerInfo { name: Arc<String> },
    GetVersions,
    GetChampInfo { url: String },
    GetChampImage { url: String, id: i64 },
}

/// TODO: Store player data in a sub struct
pub struct MyEguiApp {
    messenger: crossbeam_channel::Sender<Message>,
    receiver: crossbeam_channel::Receiver<Results>,

    active_player: String,
    role: u8,
    roles_map: HashMap<String, i8>,
    summeries: Option<Vec<MatchSummary>>,
    rank: Option<Vec<RankScore>>,
    ranking: Option<OverallRanking>,
    player_icon: Option<egui::TextureHandle>,
    data_dragon: DataDragon,

    refresh_enabled: bool,
    update_enabled: bool,

    id_name_champ_map: Option<HashMap<i64, Champion>>,
}

struct DataDragon {
    versions: Option<Vec<String>>,
    ver_started: bool,
    champ_json: Option<ChampionJson>,
}

mod champ {
    use eframe::egui;

    /// # Safety:
    /// Because ths name and key strings are loaded once, and cannot be unloaded,
    /// we create this struct after loading, with a ptr and len for every champ in
    /// the json, meaning this ptrs cannot be null
    pub struct Champion {
        name_ptr: *const u8,
        name_len: u8,
        key_ptr: *const u8,
        key_len: u8,
        image: Option<egui::TextureHandle>,
        image_started: bool,
    }

    impl Champion {
        pub fn new(name: &str, key: &str) -> Champion {
            Champion {
                name_ptr: name.as_ptr(),
                name_len: name.len() as u8,
                key_ptr: key.as_ptr(),
                key_len: key.len() as u8,
                image: None,
                image_started: false,
            }
        }

        pub fn name(&self) -> &str {
            // SAFETY: A string ptr will not move even if the string is moved, and the length will never be modified in this code
            unsafe {
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    self.name_ptr,
                    self.name_len as usize,
                ))
            }
        }

        pub fn key(&self) -> &str {
            // SAFETY: A string ptr will not move even if the string is moved, and the length will never be modified in this code
            unsafe {
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    self.key_ptr,
                    self.key_len as usize,
                ))
            }
        }

        pub fn image(&self) -> Option<&egui::TextureHandle> {
            self.image.as_ref()
        }

        pub fn set_image(&mut self, texture: Option<egui::TextureHandle>) {
            self.image = texture;
        }

        pub fn image_started(&self) -> bool {
            self.image_started
        }

        pub fn set_image_started(&mut self, started: bool) {
            self.image_started = started;
        }
    }
}

use champ::Champion;

static ROLES: [&str; 6] = ["Top", "Jungle", "Mid", "ADC", "Support", "None"];

const UGG_ROLES_REVERSED: [&str; 8] =
    ["", "Jungle", "Support", "ADC", "Top", "Mid", "Aram", "None"];

impl MyEguiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        sender: crossbeam_channel::Sender<Message>,
        receiver: crossbeam_channel::Receiver<Results>,
    ) -> Self {
        let roles_map = HashMap::from([
            ("Top".to_owned(), 4),
            ("Jungle".to_owned(), 1),
            ("Mid".to_owned(), 5),
            ("ADC".to_owned(), 3),
            ("Support".to_owned(), 2),
        ]);

        Self {
            active_player: Default::default(),
            role: 5,
            roles_map,
            summeries: None,
            rank: None,
            ranking: None,
            refresh_enabled: false,
            update_enabled: false,
            player_icon: None,
            data_dragon: DataDragon {
                versions: None,
                ver_started: false,
                champ_json: None,
            },
            id_name_champ_map: None,
            messenger: sender,
            receiver,
        }
    }

    fn send_message(&self, message: Message) {
        let _ = self.messenger.send(message);
    }

    /// This long line of function calls, well looking like bullshit
    /// actually drives the entire state of the GUI to change!
    fn update_matches(&self, ctx: &egui::Context, name: Arc<String>) {
        self.send_message(Message {
            ctx: ctx.clone(),
            payload: Payload::MatchSummaries {
                name: name.clone(),
                roles: self
                    .roles_map
                    .get(ROLES[self.role as usize])
                    .map_or_else(Vec::new, |role| vec![*role]),
            },
        });
        self.send_message(Message {
            ctx: ctx.clone(),
            payload: Payload::PlayerRanks { name: name.clone() },
        });
        self.send_message(Message {
            ctx: ctx.clone(),
            payload: Payload::PlayerRanking { name: name.clone() },
        });
        self.send_message(Message {
            ctx: ctx.clone(),
            payload: Payload::PlayerInfo { name: name.clone() },
        });
    }

    fn match_page(
        &mut self,
        summary: &MatchSummary,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        versions: &Vec<String>,
        map: &mut HashMap<i64, Champion>,
    ) {
        let champ = map.get_mut(&summary.champion_id).unwrap();
        let id = ui.make_persistent_id(summary.match_id);

        egui::collapsing_header::CollapsingState::load_with_default_open(ctx, id, false)
            .show_header(ui, |ui| {
                if let Some(image) = champ.image() {
                    ui.image(image, Vec2::splat(40.0));
                    ui.label(format!(
                        "{} {}",
                        champ.name(),
                        UGG_ROLES_REVERSED[summary.role as usize]
                    ));
                } else {
                    ui.spinner();

                    if !champ.image_started() {
                        self.send_message(Message {
                            ctx: ctx.clone(),
                            payload: Payload::GetChampImage {
                                url: format!(
                                    "http://ddragon.leagueoflegends.com/cdn/{}/img/champion/{}.png",
                                    versions[0],
                                    champ.key()
                                ),
                                id: summary.champion_id,
                            },
                        });

                        champ.set_image_started(true);
                    }
                }
            })
            .body(|ui| {
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
        self.send_message(Message {
            ctx: ctx.clone(),
            payload: Payload::UpdatePlayer { name: name.clone() },
        });
    }

    fn update_data(&mut self, ctx: &egui::Context) {
        if !self.active_player.is_empty() {
            self.refresh_enabled = true;
            self.update_enabled = true;
        } else {
            self.refresh_enabled = false;
            self.update_enabled = false;
        };

        if !self.data_dragon.ver_started {
            self.send_message(Message {
                ctx: ctx.clone(),
                payload: Payload::GetVersions,
            });

            self.data_dragon.ver_started = true;
        }

        if let Ok(receiver) = self.receiver.try_recv() {
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
                            self.update_matches(ctx, Arc::new(self.active_player.clone()));
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
                        let data = rank
                            .data
                            .fetch_profile_ranks
                            .rank_scores
                            .into_iter()
                            .filter_map(|rank| {
                                if rank.queue_type.is_empty() {
                                    None
                                } else {
                                    Some(rank)
                                }
                            })
                            .collect();
                        self.rank = Some(data);
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
                    Ok(info) => {
                        println!("{:?}", info);
                    }
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
                Results::Versions(vers) => match vers {
                    Ok(versions) => {
                        self.send_message(Message { ctx: ctx.clone(), payload: Payload::GetChampInfo { url: format!("http://ddragon.leagueoflegends.com/cdn/{}/data/en_US/champion.json", versions[0]) } });
                        self.data_dragon.versions = Some(versions);
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::ChampJson(json) => match json {
                    Ok(json) => {
                        let mut id_name_champ_map = HashMap::with_capacity(json.data.len());
                        for data in json.data.values() {
                            let id: i64 = data.key.parse().unwrap();
                            id_name_champ_map.insert(id, Champion::new(&data.name, &data.id));
                        }
                        self.data_dragon.champ_json = Some(json);
                        self.id_name_champ_map = Some(id_name_champ_map);
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::ChampImage(image) => match image {
                    Ok((image, id)) => {
                        let mut decoder = png::Decoder::new(&*image);
                        let headers = decoder
                            .read_header_info()
                            .map_err(|err| println!("{:?}", err))
                            .expect("This is always a PNG, so this shouldn't ever fail");

                        let x = headers.height as usize;
                        let y = headers.width as usize;

                        let mut reader = decoder.read_info().unwrap();
                        let mut buf = vec![0; reader.output_buffer_size()];

                        reader.next_frame(&mut buf).expect(
                            "If the champ does not exist in the map, something is deeply wrong",
                        );

                        let texture = ctx.load_texture(
                            "icon",
                            ColorImage::from_rgb([x, y], &buf),
                            TextureOptions::LINEAR,
                        );

                        let handle = self
                            .id_name_champ_map
                            .as_mut()
                            .unwrap()
                            .get_mut(&id)
                            .unwrap();

                        handle.set_image(Some(texture));
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
            }
        };
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_data(ctx);
        if let Some(versions) = self.data_dragon.versions.take() {
            if let Some(mut id_name_champ_map) = self.id_name_champ_map.take() {
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
                                        self.update_matches(
                                            ctx,
                                            Arc::new(self.active_player.clone()),
                                        );
                                    } else {
                                        self.summeries = None;
                                        self.player_icon = None;
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

                            let button = ui.add_enabled(
                                self.refresh_enabled,
                                egui::Button::new("Refresh Player"),
                            );

                            if button.clicked() {
                                self.update_matches(ctx, Arc::new(self.active_player.clone()));
                            };

                            ui.add_space(5.0);

                            let button = ui.add_enabled(
                                self.update_enabled,
                                egui::Button::new("Update Player"),
                            );

                            if button.clicked() {
                                self.update_player(ctx);
                            };

                            egui::TopBottomPanel::bottom("bottom_panel").show_inside(ui, |ui| {
                                ui.add_space(5.0);
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::LEFT),
                                    |ui| {
                                        egui::widgets::global_dark_light_mode_switch(ui);
                                    },
                                );
                            });
                        });
                    });

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.horizontal(|ui| {
                            if let Some(texture) = &self.player_icon {
                                ui.image(texture, Vec2::splat(50.0));
                            } else if !self.active_player.is_empty() && self.summeries.is_some() {
                                ui.spinner();
                            }

                            if let Some(scores) = &self.rank {
                                if scores.is_empty() {
                                    ui.vertical(|ui| {
                                        ui.label("Unranked");
                                        ui.label("LP: None");
                                        ui.label("Ranking: None");
                                    });
                                } else {
                                    for rank in scores {
                                        ui.vertical(|ui| {
                                            ui.label(format!("Wins: {}", rank.wins));
                                            ui.label(format!("Losses: {}", rank.losses));
                                            ui.label(format!("Queue: {}", rank.queue_type));
                                        });

                                        ui.separator();

                                        ui.vertical(|ui| {
                                            ui.label(format!("Wins: {}", rank.wins));
                                            ui.label(format!("Losses: {}", rank.losses));
                                            if let Some(ranking) = &self.ranking {
                                                ui.label(format!(
                                                    "Ranking: {} / {}",
                                                    ranking.overall_ranking,
                                                    ranking.total_player_count
                                                ));
                                            } else {
                                                ui.label("Ranking: None");
                                            }
                                        });

                                        ui.separator();
                                    }
                                }
                            };
                        });

                        ui.add_space(5.0);

                        let summeries = self.summeries.take();

                        if let Some(sums) = &summeries {
                            ui.separator();

                            ui.add_space(5.0);

                            egui::ScrollArea::vertical()
                                .max_height(ui.available_height())
                                .show(ui, |ui| {
                                    if sums.is_empty() {
                                        ui.label("No Data");
                                    } else {
                                        for summary in sums {
                                            self.match_page(
                                                summary,
                                                ui,
                                                ctx,
                                                &versions,
                                                &mut id_name_champ_map,
                                            );
                                            ui.separator();
                                        }
                                    }
                                });
                        }

                        self.summeries = summeries;
                    });
                });

                self.id_name_champ_map = Some(id_name_champ_map);
            } else {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.spinner();
                });
            }
            self.data_dragon.versions = Some(versions)
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.spinner();
            });
        }
    }
}

fn format_time(match_time: i64) -> String {
    let native_time = NaiveDateTime::from_timestamp_opt(match_time, 0).unwrap();
    let time: DateTime<Utc> = DateTime::from_local(native_time, Utc);
    let mut human_time = time.format("%H:%M:%S");
    if human_time.to_string().char_range(0..2) == "00" {
        human_time = time.format("%M:%S");
    }
    human_time.to_string()
}
