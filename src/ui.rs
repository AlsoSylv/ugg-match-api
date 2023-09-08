use crate::structs::{
    self, ChampData, ChampionJson, GetMatch, Match, MatchSummary, OverallRanking, RankScore,
};
use crate::Errors;
use chrono::{DateTime, NaiveDateTime, Utc};
use eframe::egui::{self, TextBuffer, TextureOptions, Ui, Vec2};
use eframe::epaint::ColorImage;
use std::cell::OnceCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Debug)]
pub enum Results {
    MatchSum(Result<structs::PlayerMatchSummeries, Errors>),
    PlayerUpdate(Result<structs::UpdatePlayer, Errors>),
    ProfileRanks(Result<structs::PlayerRank, Errors>),
    Ranking(Result<structs::PlayerRanking, Errors>),
    PlayerInfo(Result<structs::PlayerInfo, Errors>),
    PlayerIcon(Result<(i64, bytes::Bytes), Errors>),
    Versions(Result<Arc<[String]>, Errors>),
    ChampJson(Result<ChampionJson, Errors>),
    ChampImage(Result<(bytes::Bytes, i64), Errors>),
    MatchDetails(Box<Result<(GetMatch, i64), Errors>>),
}

pub struct Message {
    pub ctx: egui::Context,
    pub payload: Payload,
}

#[derive(Debug)]
pub enum Payload {
    MatchSummaries {
        name: Arc<String>,
        roles: Vec<u8>,
    },
    PlayerRanks {
        name: Arc<String>,
    },
    UpdatePlayer {
        name: Arc<String>,
    },
    PlayerRanking {
        name: Arc<String>,
    },
    PlayerInfo {
        name: Arc<String>,
        version: Arc<[String]>,
        version_index: usize,
    },
    GetVersions,
    GetChampInfo {
        url: String,
    },
    GetChampImage {
        url: String,
        id: i64,
    },
    GetMatchDetails {
        name: Arc<String>,
        version: Arc<String>,
        id: String,
    },
}

/// TODO: Store player data in a sub struct
pub struct MyEguiApp {
    messenger: async_channel::Sender<Message>,
    receiver: async_channel::Receiver<Results>,

    active_player: String,
    message_name: Arc<String>,
    role: u8,
    summeries: Option<Vec<MatchSummary>>,
    rank: Option<Vec<RankScore>>,
    ranking: Option<OverallRanking>,
    data_dragon: DataDragon,
    refresh_enabled: bool,
    update_enabled: bool,
    id_name_champ_map: OnceCell<HashMap<i64, Champ>>,
    match_summeries: Option<HashMap<i64, Option<Match>>>,
    player_icons: HashMap<i64, Option<egui::TextureHandle>>,
    icon_id: i64,
}

struct DataDragon {
    versions: OnceCell<Arc<[String]>>,
    ver_started: bool,
}

pub struct Champ {
    pub key: String,
    pub name: String,
    image: Option<egui::TextureHandle>,
    image_started: AtomicBool,
}

impl Debug for Champ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Champ")
            .field("key", &self.key)
            .field("name", &self.name)
            .field("image_started", &self.image_started)
            .finish()
    }
}

impl ChampData {
    fn into(self) -> Champ {
        Champ {
            key: self.key,
            name: self.name,
            image: None,
            image_started: AtomicBool::new(false),
        }
    }
}

static ROLES: [&str; 6] = ["Top", "Jungle", "Mid", "ADC", "Support", "None"];

const UGG_ROLES_REVERSED: [&str; 8] =
    ["", "Jungle", "Support", "ADC", "Top", "Mid", "Aram", "None"];

fn get_role_index(role: u8) -> Option<u8> {
    match role {
        0 => Some(4), // Top
        1 => Some(1), // Jungle
        2 => Some(5), // Mid
        3 => Some(3), // ADC
        4 => Some(2), // Support
        _ => None,    // No role, used to map to an empty vec
    }
}

impl MyEguiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        sender: async_channel::Sender<Message>,
        receiver: async_channel::Receiver<Results>,
    ) -> Self {
        Self {
            active_player: Default::default(),
            message_name: Default::default(),
            role: 5,
            summeries: None,
            rank: None,
            ranking: None,
            refresh_enabled: false,
            update_enabled: false,
            // player_icon: None,
            data_dragon: DataDragon {
                versions: Default::default(),
                ver_started: false,
                // champ_json: None,
            },
            id_name_champ_map: Default::default(),
            messenger: sender,
            receiver,
            match_summeries: None,
            player_icons: Default::default(),
            icon_id: -1,
        }
    }

    fn send_message(&self, ctx: &egui::Context, payload: Payload) {
        self.messenger
            .try_send(Message {
                ctx: ctx.clone(),
                payload,
            })
            .unwrap();
    }

    /// This long line of function calls, well looking like bullshit
    /// actually drives the entire state of the GUI to change!
    fn update_matches(&self, ctx: &egui::Context, name: Arc<String>, versions: Arc<[String]>) {
        self.send_message(
            ctx,
            Payload::MatchSummaries {
                name: name.clone(),
                roles: get_role_index(self.role).map_or_else(Vec::new, |role| vec![role]),
            },
        );
        self.send_message(ctx, Payload::PlayerRanks { name: name.clone() });
        self.send_message(ctx, Payload::PlayerRanking { name: name.clone() });
        self.send_message(
            ctx,
            Payload::PlayerInfo {
                name: name.clone(),
                version: versions,
                version_index: 0,
            },
        );
    }

    fn match_page(
        &self,
        summary: &MatchSummary,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        champ: &Champ,
    ) {
        let id = ui.make_persistent_id(summary.match_id);

        egui::collapsing_header::CollapsingState::load_with_default_open(ctx, id, false)
            .show_header(ui, |ui| {
                if let Some(image) = &champ.image {
                    ui.image(image, Vec2::splat(40.0));
                } else {
                    ui.spinner();
                }

                ui.vertical(|ui| {
                    ui.label(&champ.name);
                    ui.label(UGG_ROLES_REVERSED[summary.role as usize]);
                });
                ui.vertical(|ui| {
                    let win;

                    if summary.win {
                        win = "Win";
                    } else {
                        win = "Loss";
                    }

                    ui.label(win);

                    let kda = format!("{}/{}/{}", summary.kills, summary.deaths, summary.assists);

                    ui.label(kda);
                });
            })
            .body(|ui| {
                if let Some(map) = &self.match_summeries {
                    if let Some(Some(md)) = map.get(&summary.match_id) {
                        let player_data = |ui: &mut Ui, role_index: u8, name: &str| {
                            ui.horizontal(|ui| {
                                ui.label(UGG_ROLES_REVERSED[role_index as usize]);
                                ui.label(name);
                            });
                        };

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                for player in md.match_summary.team_a.iter() {
                                    player_data(ui, player.role, &player.summoner_name);
                                }
                            });

                            ui.separator();

                            ui.vertical(|ui| {
                                for player in md.match_summary.team_b.iter() {
                                    player_data(ui, player.role, &player.summoner_name);
                                }
                            });
                        });
                    }
                }
            });
    }

    fn update_player(&self, ctx: &egui::Context) {
        self.send_message(
            ctx,
            Payload::UpdatePlayer {
                name: self.message_name.clone(),
            },
        );
    }

    fn load_version(&mut self, ctx: &egui::Context) {
        if !self.data_dragon.ver_started {
            println!("Request sent!");
            self.send_message(ctx, Payload::GetVersions);
            self.data_dragon.ver_started = true;
        }

        if let Ok(Results::Versions(versions)) = self.receiver.try_recv() {
            match versions {
                Ok(versions) => {
                    self.send_message(ctx, Payload::GetChampInfo { url: format!("http://ddragon.leagueoflegends.com/cdn/{}/data/en_US/champion.json", versions[0]) } );
                    self.data_dragon.versions.set(versions).unwrap();
                }
                Err(err) => {
                    egui::Window::new("Version Error").show(ctx, |ui| ui.label(err.to_string()));
                }
            }
        }
    }

    fn load_champ_json(&mut self, ctx: &egui::Context) {
        if let Ok(Results::ChampJson(json)) = self.receiver.try_recv() {
            match json {
                Ok(json) => {
                    self.id_name_champ_map.get_or_init(|| {
                        let mut id_name_champ_map = HashMap::with_capacity(json.data.len());
                        for (_, data) in json.data {
                            let id: i64 = data.id.parse().unwrap();
                            id_name_champ_map.insert(id, data.into());
                        }

                        id_name_champ_map
                    });
                }
                Err(err) => {
                    egui::Window::new("Champ Json Error").show(ctx, |ui| ui.label(err.to_string()));
                }
            }
        }
    }

    fn update_data(
        &mut self,
        ctx: &egui::Context,
        versions: Arc<[String]>,
        id_name_champ_map: &mut HashMap<i64, Champ>,
    ) {
        if let Ok(receiver) = self.receiver.try_recv() {
            match receiver {
                Results::MatchSum(match_sums) => match match_sums {
                    Ok(matches) => {
                        self.match_summeries = Some(HashMap::with_capacity(20));
                        let summaries = matches.data.fetch_player_match_summaries.match_summaries;
                        summaries.iter().for_each(|summary| {
                            if let Some(map) = &mut self.match_summeries {
                                if let Entry::Vacant(e) = map.entry(summary.match_id) {
                                    e.insert(None);
                                    self.send_message(ctx, Payload::GetMatchDetails { name: self.message_name.clone(), version: summary.version.clone(), id: summary.match_id.to_string() });
                                }
                            }
                            let champ = &id_name_champ_map[&summary.champion_id];

                            if !champ.image_started.load(std::sync::atomic::Ordering::Relaxed) {
                                let key = &champ.key;
                                self.send_message(
                                    ctx,
                                    Payload::GetChampImage {
                                        url: format!(
                                            "http://ddragon.leagueoflegends.com/cdn/{}/img/champion/{}.png",
                                            versions[0], key
                                        ),
                                        id: summary.champion_id,
                                    },
                                );
                                champ.image_started.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                        });
                        self.summeries = Some(summaries)
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::PlayerUpdate(update) => match update {
                    Ok(updated) => {
                        let data = updated.data.update_player_profile;
                        if data.success {
                            self.update_matches(ctx, self.message_name.clone(), versions.clone());
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
                        self.icon_id = info.data.profile_player_info.unwrap().icon_id;
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::PlayerIcon(data) => match data {
                    Ok((id, icon)) => {
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
                        let _ = self.player_icons.insert(id, Some(texture));
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

                        let handle = id_name_champ_map.get_mut(&id).unwrap();

                        handle.image = Some(texture);
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::MatchDetails(deets) => match *deets {
                    Ok((match_details, id)) => {
                        self.match_summeries
                            .as_mut()
                            .expect("The app has reached an impossible state")
                            .insert(id, Some(match_details.data.data_match));
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },

                payload => unreachable!(
                    "App has reached an impossible state, this should already be covered {:?}",
                    payload
                ),
            }
        };
    }

    fn zero_player(&mut self) {
        self.summeries = None;
        // self.player_icon = None;
        self.rank = None;
        self.ranking = None;
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.active_player.is_empty() {
            self.refresh_enabled = true;
            self.update_enabled = true;
        } else {
            self.refresh_enabled = false;
            self.update_enabled = false;
        };

        let Some(versions) = self.data_dragon.versions.get() else {
            self.load_version(ctx);
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.spinner();
            });

            return;
        };

        let versions = versions.clone();

        let Some(mut id_name_champ_map) = self.id_name_champ_map.take() else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.spinner();
            });

            self.load_champ_json(ctx);
            return;
        };

        self.update_data(ctx, versions.clone(), &mut id_name_champ_map);

        let side_panel_ui = |ui: &mut Ui| {
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Player: ");

                let search_bar = ui.text_edit_singleline(&mut self.active_player);

                if search_bar.clicked() && !self.active_player.is_empty() {
                    if !self.active_player.is_empty() {
                        self.message_name = Arc::new(self.active_player.clone());
                        self.update_matches(ctx, self.message_name.clone(), versions.clone());
                    } else {
                        self.zero_player();
                    }
                }

                if search_bar.changed() {
                    self.zero_player();
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Roles: ");

                egui::ComboBox::from_id_source("roles")
                    .selected_text(ROLES[self.role as usize])
                    .show_ui(ui, |ui| {
                        ROLES.iter().enumerate().for_each(|(index, text)| {
                            ui.selectable_value(&mut self.role, index as u8, *text);
                        })
                    });
            });

            ui.add_space(5.0);

            let button = egui::Button::new("Refresh Player");
            if ui.add_enabled(self.refresh_enabled, button).clicked() {
                self.update_matches(ctx, self.message_name.clone(), versions.clone());
            }

            ui.add_space(5.0);

            let button = egui::Button::new("Update Player");
            if ui.add_enabled(self.update_enabled, button).clicked() {
                self.update_player(ctx);
            }

            egui::TopBottomPanel::bottom("bottom_panel").show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    egui::widgets::global_dark_light_mode_switch(ui);
                });
            });
        };

        egui::SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, side_panel_ui);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    if let Some(Some(texture)) = self.player_icons.get(&self.icon_id) {
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
                                    ui.label(format!("Rank: {}", rank.rank));
                                    ui.label(format!("LP: {}", rank.lp));
                                    ui.label(format!("Queue: {}", rank.queue_type));
                                });

                                ui.separator();

                                ui.vertical(|ui| {
                                    ui.label(format!("Wins: {}", rank.wins));
                                    ui.label(format!("Losses: {}", rank.losses));
                                    if let Some(ranking) = &self.ranking {
                                        ui.label(format!(
                                            "Ranking: {} / {}",
                                            ranking.overall_ranking, ranking.total_player_count
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

                if let Some(sums) = &self.summeries {
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height())
                        .show(ui, |ui| {
                            if sums.is_empty() {
                                ui.label("No Data");
                            } else {
                                for summary in sums {
                                    let champ = &id_name_champ_map[&summary.champion_id];
                                    self.match_page(summary, ui, ctx, champ);
                                    ui.separator();
                                }
                            }
                        });
                }
            });
        });

        self.id_name_champ_map.set(id_name_champ_map).unwrap();
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
