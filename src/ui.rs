use crate::structs::{self, ChampData, GetMatch, Match, MatchSummary, OverallRanking, RankScore};
use crate::{spawn_gui_shit, Errors, SharedState};
use chrono::{DateTime, NaiveDateTime, Utc};
use eframe::egui::{self, TextBuffer, Ui, Vec2};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use tokio::runtime::Runtime;

#[derive(Debug)]
pub enum Results {
    MatchSum(Result<structs::PlayerMatchSummeries, Errors>),
    PlayerUpdate(Result<structs::UpdatePlayer, Errors>),
    ProfileRanks(Result<structs::PlayerRank, Errors>),
    Ranking(Result<structs::PlayerRanking, Errors>),
    PlayerInfo(Result<structs::PlayerInfo, Errors>),
    PlayerIcon(Errors),
    Versions(Errors),
    ChampJson(Errors),
    ChampImage(Errors),
    MatchDetails(Result<(Box<GetMatch>, i64), Errors>),
}

#[derive(Debug)]
pub enum Payload {
    MatchSummaries {
        name: Arc<String>,
        roles: Vec<u8>,
        region_id: &'static str,
    },
    PlayerRanks {
        name: Arc<String>,
        region_id: &'static str,
    },
    UpdatePlayer {
        name: Arc<String>,
        region_id: &'static str,
    },
    PlayerRanking {
        name: Arc<String>,
        region_id: &'static str,
    },
    PlayerInfo {
        name: Arc<String>,
        version: Arc<[String]>,
        version_index: usize,
        region_id: &'static str,
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
        region_id: &'static str,
    },
}

/// TODO: Store player data in a sub struct
pub struct MyEguiApp {
    messenger: async_channel::Sender<Payload>,
    receiver: async_channel::Receiver<Results>,

    // The state shared between all worker threads and the main GUI thread
    shared_state: Arc<SharedState>,

    // Actively tracked state of GUI components
    refresh_enabled: bool,
    update_enabled: bool,
    icon_id: i64,
    role: u8,

    // Values used for data lookup
    active_player: String,
    message_name: Arc<String>,
    data_dragon: DataDragon,
    match_summeries: HashMap<i64, MatchFuture>,

    // These three are loaded lazily, and may or may not exist!
    summeries: Option<Vec<MatchSummary>>,
    rank: Option<Vec<RankScore>>,
    ranking: Option<OverallRanking>,

    // Runtime so the threads don't close
    _rt: Runtime,
}

/// This is really only used to avoid spamming network requests
struct MatchFuture {
    _match: Option<Match>,
}

/// This stores all data dragon assets that are being used at any given time, that are not in the shared state
struct DataDragon {
    ver_started: bool,
    champ_info_started: bool,
    region_id_name: HashMap<&'static str, &'static str>,
    region: &'static str,
}

/// Struct representing all the data of a champ we display
pub struct Champ {
    pub key: String,
    pub name: String,
    // This is updated from the threadpool, and as such, can be locked
    pub image: RwLock<Option<egui::TextureHandle>>,
    // The champ struct is passed around a lot, but this allos me to only use
    // .get() instead of .take() and .set() at the beginning and end of the loop
    pub image_started: AtomicBool,
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

impl From<ChampData> for Champ {
    fn from(val: ChampData) -> Champ {
        Champ {
            key: val.key,
            name: val.name,
            image: RwLock::new(None),
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
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (_rt, sender, receiver, shared_state) = spawn_gui_shit(&_cc.egui_ctx);

        Self {
            active_player: Default::default(),
            message_name: Default::default(),
            shared_state,
            role: 5,
            summeries: None,
            rank: None,
            ranking: None,
            refresh_enabled: false,
            update_enabled: false,
            data_dragon: DataDragon {
                ver_started: false,
                champ_info_started: false,
                region_id_name: HashMap::from([("na1", "North America"), ("euw1", "EU West")]),
                region: "na1",
            },
            messenger: sender,
            receiver,
            match_summeries: Default::default(),
            icon_id: -1,
            _rt,
        }
    }

    fn send_message(&self, payload: Payload) {
        self.messenger.try_send(payload).unwrap();
    }

    fn update_matches(&self, name: Arc<String>, versions: Arc<[String]>) {
        self.send_message(Payload::MatchSummaries {
            name: name.clone(),
            roles: get_role_index(self.role).map_or_else(Vec::new, |role| vec![role]),
            region_id: self.data_dragon.region,
        });
        self.send_message(Payload::PlayerRanks {
            name: name.clone(),
            region_id: self.data_dragon.region,
        });
        self.send_message(Payload::PlayerRanking {
            name: name.clone(),
            region_id: self.data_dragon.region,
        });
        self.send_message(Payload::PlayerInfo {
            name: name.clone(),
            version: versions,
            version_index: 0,
            region_id: self.data_dragon.region,
        });
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
                if let Ok(image) = &champ.image.try_read() {
                    if let Some(texture) = &**image {
                        ui.image(texture, Vec2::splat(40.0));
                    } else {
                        ui.spinner();
                    }
                } else {
                    ui.spinner();
                }

                ui.vertical(|ui| {
                    ui.label(&champ.name);
                    ui.label(UGG_ROLES_REVERSED[summary.role as usize]);
                });
                ui.vertical(|ui| {
                    let win = if summary.win { "Win" } else { "Loss" };

                    ui.label(win);

                    let kda = format!("{}/{}/{}", summary.kills, summary.deaths, summary.assists);

                    ui.label(kda);
                });
            })
            .body(|ui| {
                let map = &self.match_summeries;
                {
                    if let Some(md) = map.get(&summary.match_id) {
                        if let Some(md) = &md._match {
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
                }
            });
    }

    fn update_player(&self) {
        self.send_message(Payload::UpdatePlayer {
            name: self.message_name.clone(),
            region_id: self.data_dragon.region,
        });
    }

    fn load_version(&mut self, ctx: &egui::Context) {
        if !self.data_dragon.ver_started {
            self.send_message(Payload::GetVersions);
            self.data_dragon.ver_started = true;
        }

        if let Ok(Results::Versions(err)) = self.receiver.try_recv() {
            egui::Window::new("Version Error").show(ctx, |ui| ui.label(err.to_string()));
        }
    }

    fn update_data(&mut self, versions: Arc<[String]>, champs: Arc<HashMap<i64, Champ>>) {
        if let Ok(receiver) = self.receiver.try_recv() {
            match receiver {
                Results::MatchSum(match_sums) => match match_sums {
                    Ok(matches) => {
                        let summaries = matches.data.fetch_player_match_summaries.match_summaries;
                        summaries.iter().for_each(|summary| {
                            if self.match_summeries.get(&summary.match_id).is_none() {
                                self.match_summeries.insert(summary.match_id, MatchFuture { _match: None });
                                self.send_message(Payload::GetMatchDetails { name: self.message_name.clone(), version: summary.version.clone(), id: summary.match_id.to_string(), region_id: self.data_dragon.region });
                            }

                            let champ = &champs[&summary.champion_id];
                            if !champ.image_started.load(std::sync::atomic::Ordering::Relaxed) {
                                let key = &champ.key;
                                self.send_message(
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
                            self.update_matches(self.message_name.clone(), versions.clone());
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
                Results::PlayerIcon(data) => {
                    todo!("{:?}", data)
                }
                Results::ChampImage(image_errors) => {
                    todo!("{:?}", image_errors)
                }
                Results::MatchDetails(deets) => match deets {
                    Ok((match_details, id)) => {
                        self.match_summeries.insert(
                            id,
                            MatchFuture {
                                _match: Some(match_details.data.data_match),
                            },
                        );
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },

                Results::ChampJson(err) => {
                    todo!("{:?}", err)
                }

                payload => unreachable!(
                    "App has reached an impossible state, this should already be covered {:?}",
                    payload
                ),
            }
        };
    }

    fn zero_player(&mut self) {
        self.icon_id = -1;
        self.summeries = None;
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

        let Some(versions) = self.shared_state.versions.get() else {
            self.load_version(ctx);
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.spinner();
            });

            return;
        };

        let versions = versions.clone();

        let Some(champs) = self.shared_state.champs.get() else {
            if !self.data_dragon.champ_info_started {
                self.send_message(Payload::GetChampInfo { url: format!("http://ddragon.leagueoflegends.com/cdn/{}/data/en_US/champion.json", versions[0]) });
                self.data_dragon.champ_info_started = true;
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.spinner();
            });

            return;
        };

        let champs = champs.clone();

        self.update_data(versions.clone(), champs.clone());

        let side_panel_ui = |ui: &mut Ui| {
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Player: ");

                let search_bar = ui.text_edit_singleline(&mut self.active_player);

                if search_bar.clicked() && !self.active_player.is_empty() {
                    if !self.active_player.is_empty() {
                        self.message_name = Arc::new(self.active_player.clone());
                        self.update_matches(self.message_name.clone(), versions.clone());
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

            ui.horizontal(|ui| {
                ui.label("Regions: ");

                egui::ComboBox::from_id_source("regions")
                    .selected_text(self.data_dragon.region_id_name[self.data_dragon.region])
                    .show_ui(ui, |ui| {
                        self.data_dragon
                            .region_id_name
                            .iter()
                            .for_each(|(index, name)| {
                                if ui.button(*name).clicked() {
                                    self.data_dragon.region = *index;
                                };
                            });
                    });
            });

            ui.add_space(5.0);

            let button = egui::Button::new("Refresh Player");
            if ui.add_enabled(self.refresh_enabled, button).clicked() {
                self.update_matches(self.message_name.clone(), versions.clone());
            }

            ui.add_space(5.0);

            let button = egui::Button::new("Update Player");
            if ui.add_enabled(self.update_enabled, button).clicked() {
                self.update_player();
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
                    if self.icon_id != -1 {
                        if let Ok(map) = self.shared_state.player_icons.try_read() {
                            if let Some(texture) = map.get(&self.icon_id) {
                                ui.image(texture, Vec2::splat(50.0));
                            } else {
                                ui.spinner();
                            }
                        } else {
                            ui.spinner();
                        }
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
                                    let champ = &champs[&summary.champion_id];
                                    self.match_page(summary, ui, ctx, champ);
                                    ui.separator();
                                }
                            }
                        });
                }
            });
        });
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
