use crate::structs::{self, ChampData, GetMatch, Match, MatchSummary, OverallRanking, RankScore};
use crate::{spawn_gui_shit, Errors, SharedState};
use chrono::{DateTime, NaiveDateTime, Utc};
use eframe::egui::{
    self, Button, CentralPanel, ComboBox, Label, RichText, TextBuffer, TextEdit, Ui, Vec2,
};
use eframe::epaint::Color32;
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
        roles: Option<u8>,
        page: u8,
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
        version: String,
        id: i64,
        region_id: &'static str,
    },
}

pub struct MyEguiApp {
    pub messenger: async_channel::Sender<Payload>,
    pub receiver: async_channel::Receiver<Results>,

    // The state shared between all worker threads and the main GUI thread
    pub shared_state: Arc<SharedState>,

    // Actively tracked state of GUI components
    pub refresh_enabled: bool,
    pub update_enabled: bool,
    pub finished_match_summeries: bool,
    pub page: u8,
    pub role: u8,

    // Values used for data lookup
    pub active_player: String,
    pub message_name: Arc<String>,
    pub data_dragon: DataDragon,

    // These three are loaded lazily, and may or may not exist!
    pub player_data: PlayerData,

    // Runtime so the threads don't close
    _rt: Runtime,
}

pub struct PlayerData {
    pub match_data_map: HashMap<i64, Option<Match>>,
    pub match_summaries: Option<Box<[MatchSummary]>>,
    pub rank_scores: Option<Box<[RankScore]>>,
    pub ranking: Option<OverallRanking>,
    pub icon_id: i16,
}

/// This stores all data dragon assets that are being used at any given time, that are not in the shared state
pub struct DataDragon {
    pub ver_started: bool,
    pub champ_info_started: bool,
    pub region_id_name: HashMap<&'static str, &'static str>,
    pub region: &'static str,
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

impl MyEguiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (_rt, sender, receiver, shared_state) = spawn_gui_shit(&_cc.egui_ctx);

        Self {
            active_player: Default::default(),
            message_name: Default::default(),
            shared_state,
            role: 5,
            refresh_enabled: false,
            update_enabled: false,
            data_dragon: DataDragon {
                ver_started: false,
                champ_info_started: false,
                region_id_name: HashMap::from([("na1", "NA"), ("euw1", "EUW")]),
                region: "na1",
            },
            player_data: PlayerData {
                match_data_map: Default::default(),
                match_summaries: None,
                rank_scores: None,
                ranking: None,
                icon_id: -1,
            },
            messenger: sender,
            receiver,
            page: 1,
            finished_match_summeries: true,
            _rt,
        }
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

    fn zero_player(&mut self) {
        self.player_data.icon_id = -1;
        self.player_data.match_summaries = None;
        self.player_data.rank_scores = None;
        self.player_data.ranking = None;
        self.page = 1;
        self.finished_match_summeries = true;
    }
}

fn custom_window_frame(
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    let window_frame = egui::Frame {
        fill: ctx.style().visuals.window_fill(),
        rounding: 15.0.into(),
        stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
        outer_margin: 0.5.into(),
        ..Default::default()
    };

    CentralPanel::default().frame(window_frame).show(ctx, |ui| {
        let app_rect = ui.max_rect();

        const TITLE_BAR_HEIGHT: f32 = 32.0;
        let title_bar_rect = {
            let mut rect = app_rect;
            rect.max.y = rect.min.y + TITLE_BAR_HEIGHT;
            rect
        };
        title_bar_ui(ui, frame, title_bar_rect, title);

        let content_rect = {
            let mut rect = app_rect;
            rect.min.y = title_bar_rect.max.y;
            rect
        }
        .shrink(4.0);

        let mut content_ui = ui.child_ui(content_rect, *ui.layout());
        add_contents(&mut content_ui);
    });
}

fn title_bar_ui(
    ui: &mut egui::Ui,
    frame: &mut eframe::Frame,
    title_bar_rect: eframe::epaint::Rect,
    title: &str,
) {
    use egui::*;

    let painter = ui.painter();

    let title_bar_response = ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click());

    // Paint the title:
    painter.text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(20.0),
        ui.style().visuals.text_color(),
    );

    // Paint the line under the title:
    painter.line_segment(
        [
            title_bar_rect.left_bottom() + vec2(1.0, 0.0),
            title_bar_rect.right_bottom() + vec2(-1.0, 0.0),
        ],
        ui.visuals().widgets.noninteractive.bg_stroke,
    );

    // Interact with the title bar (drag to move window):
    if title_bar_response.double_clicked() {
        frame.set_maximized(!frame.info().window_info.maximized);
    } else if title_bar_response.is_pointer_button_down_on() {
        frame.drag_window();
    }

    ui.allocate_ui_at_rect(title_bar_rect, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);
            close_maximize_minimize(ui, frame);
        });
    });
}

fn close_maximize_minimize(ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    let button_height = 12.0;

    let close_response = ui
        .add(Button::new(RichText::new("❌").size(button_height)))
        .on_hover_text("Close the window");
    if close_response.clicked() {
        frame.close();
    }

    let maximized = frame.info().window_info.maximized;

    let button = ui
        .add(Button::new(RichText::new("🗗").size(button_height)))
        .on_hover_text(if maximized {
            "Restore window"
        } else {
            "Maximize window"
        });

    if button.clicked() {
        frame.set_maximized(!maximized);
    }

    let minimized_response = ui
        .add(Button::new(RichText::new("🗕").size(button_height)))
        .on_hover_text("Minimize the window");
    if minimized_response.clicked() {
        frame.set_minimized(true);
    }
}

impl eframe::App for MyEguiApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        frame.set_decorations(false);

        custom_window_frame(ctx, frame, "Hell", |ui| {
            if !self.active_player.is_empty() {
                self.refresh_enabled = true;
                self.update_enabled = true;
            } else {
                self.refresh_enabled = false;
                self.update_enabled = false;
            };

            let Some(versions) = self.shared_state.versions.get() else {
                self.load_version(ctx);
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.spinner();
                });

                return;
            };

            let versions = versions.clone();

            let Some(champs) = self.shared_state.champs.get() else {
                if !self.data_dragon.champ_info_started {
                    self.send_message(Payload::GetChampInfo {
                        url: format!(
                            "http://ddragon.leagueoflegends.com/cdn/{}/data/en_US/champion.json",
                            versions[0]
                        ),
                    });
                    self.data_dragon.champ_info_started = true;
                }

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.spinner();
                });

                return;
            };

            let champs = champs.clone();

            self.update_data(versions.clone(), champs.clone());

            egui::SidePanel::left("Left Panel")
                // 15% of available width
                .exact_width(0.15 * ui.available_width())
                .resizable(false)
                .show_inside(ui, |ui| {
                    let full_height = ui.available_height();

                    ui.with_layout(
                        egui::Layout::left_to_right(eframe::emath::Align::Min),
                        |ui| {
                            ui.label("Player: ");

                            let active_player = &mut self.active_player;
                            let search_bar = TextEdit::singleline(active_player);

                            let search_bar =
                                ui.add_sized(Vec2::new(ui.available_width(), 0.0), search_bar);

                            if search_bar.clicked()
                                && !active_player.ends_with(' ')
                                && !self.active_player.is_empty()
                            {
                                self.message_name = Arc::new(self.active_player.clone());
                                self.update_matches(self.message_name.clone());
                            }

                            if search_bar.changed() {
                                self.zero_player();
                            }
                        },
                    );

                    ui.add_space(0.01 * full_height);

                    ui.with_layout(
                        egui::Layout::left_to_right(eframe::emath::Align::Min),
                        |ui| {
                            // Do not ask how we got here, but we got here
                            let third = ui.available_width() * 0.3;
                            ui.set_width(third * 3.1);

                            let button = Button::new("⬅").min_size(Vec2::new(third, 0.0));
                            if ui.add_enabled(self.page > 1, button).clicked() {
                                self.page -= 1;
                                self.update_matches(self.message_name.clone())
                            }

                            let label = Label::new(format!("{}", self.page));
                            ui.add_sized(Vec2::new(ui.available_width() - third, 0.0), label);

                            let button = egui::Button::new("➡").min_size(Vec2::new(third, 0.0));
                            if ui
                                .add_enabled(!self.finished_match_summeries, button)
                                .clicked()
                            {
                                self.page += 1;
                                self.update_matches(self.message_name.clone())
                            }
                        },
                    );

                    ui.add_space(0.01 * full_height);

                    ui.horizontal(|ui| {
                        ui.label("Role: ");
                        ComboBox::from_id_source("Role Select")
                            .selected_text(ROLES[self.role as usize])
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                ROLES.iter().enumerate().for_each(|(index, value)| {
                                    ui.selectable_value(&mut self.role, index as u8, *value);
                                });
                            });
                    });

                    ui.add_space(0.01 * full_height);

                    ui.horizontal(|ui| {
                        ui.label("Region: ");

                        egui::ComboBox::from_id_source("regions")
                            .selected_text(self.data_dragon.region_id_name[self.data_dragon.region])
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                self.data_dragon
                                    .region_id_name
                                    .iter()
                                    .for_each(|(index, name)| {
                                        ui.selectable_value(
                                            &mut self.data_dragon.region,
                                            *index,
                                            *name,
                                        );
                                    });
                            });
                    });

                    ui.add_space(0.01 * full_height);

                    let button = Button::new("Refresh Player")
                        .min_size(Vec2::new(ui.available_width(), 0.0));
                    if ui.add_enabled(self.refresh_enabled, button).clicked() {
                        self.update_matches(self.message_name.clone());
                    }

                    ui.add_space(0.01 * full_height);

                    let button =
                        Button::new("Update Player").min_size(Vec2::new(ui.available_width(), 0.0));
                    if ui.add_enabled(self.refresh_enabled, button).clicked() {
                        self.update_player();
                    }
                });

            let height = ui.available_height();

            if let Some(ranks) = &self.player_data.rank_scores {
                ui.add_space(0.01 * height);

                egui::TopBottomPanel::top("Top Panel").show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        if self.player_data.icon_id != -1 {
                            if let Ok(map) = self.shared_state.player_icons.try_read() {
                                if let Some(texture) = map.get(&self.player_data.icon_id) {
                                    ui.image(texture, Vec2::splat(0.1 * height));
                                } else {
                                    ui.spinner();
                                }
                            } else {
                                ui.spinner();
                            }
                        }

                        if ranks.is_empty() {
                            ui.vertical(|ui| {
                                ui.label("Unranked");
                                ui.label("LP: None");
                                ui.label("Ranking: None");
                            });
                        } else {
                            for rank in ranks.iter() {
                                if rank.queue_type.is_empty() {
                                    continue;
                                }

                                ui.vertical(|ui| {
                                    ui.label(format!("Rank: {}", rank.rank));
                                    ui.label(format!("LP: {}", rank.lp));
                                    ui.label(format!("Queue: {}", rank.queue_type));
                                });

                                ui.separator();

                                ui.vertical(|ui| {
                                    ui.label(format!("Wins: {}", rank.wins));
                                    ui.label(format!("Losses: {}", rank.losses));
                                    if let Some(ranking) = &self.player_data.ranking {
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
                    });

                    ui.add_space(0.01 * height);
                });
            }

            ui.add_space(0.01 * height);

            egui::ScrollArea::vertical()
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    if let Some(summeries) = &self.player_data.match_summaries {
                        if summeries.is_empty() {
                            ui.label("No Recent Matches");
                        } else {
                            for summary in summeries.iter() {
                                let champ = &champs[&summary.champion_id];
                                ui.add_space(0.01 * height);
                                let id = ui.make_persistent_id(summary.match_id);

                                egui::collapsing_header::CollapsingState::load_with_default_open(
                                    ctx, id, false,
                                )
                                .show_header(ui, |ui| {
                                    if let Ok(image) = &champ.image.try_read() {
                                        if let Some(texture) = &**image {
                                            ui.image(texture, Vec2::splat(0.08 * height));
                                        } else {
                                            ui.spinner();
                                        }
                                    } else {
                                        ui.spinner();
                                    }

                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(&champ.name);
                                            ui.label(UGG_ROLES_REVERSED[summary.role as usize]);
                                            let kda = format!(
                                                "{}/{}/{}",
                                                summary.kills, summary.deaths, summary.assists
                                            );

                                            ui.label(kda);
                                        });

                                        ui.horizontal(|ui| {
                                            if summary.win {
                                                ui.label(RichText::new("Win").color(Color32::KHAKI))
                                            } else {
                                                ui.label(RichText::new("Loss").color(Color32::RED))
                                            };
                                        })
                                    });
                                })
                                .body(|ui| {
                                    let map = &self.player_data.match_data_map;
                                    {
                                        if let Some(Some(md)) = map.get(&summary.match_id) {
                                            let player_data =
                                                |ui: &mut Ui, role_index: u8, name: &str| {
                                                    ui.horizontal(|ui| {
                                                        ui.label(
                                                            UGG_ROLES_REVERSED[role_index as usize],
                                                        );
                                                        ui.label(name);
                                                    });
                                                };

                                            ui.horizontal(|ui| {
                                                ui.vertical(|ui| {
                                                    for player in md.match_summary.team_a.iter() {
                                                        player_data(
                                                            ui,
                                                            player.role,
                                                            &player.summoner_name,
                                                        );
                                                    }
                                                });

                                                ui.separator();

                                                ui.vertical(|ui| {
                                                    for player in md.match_summary.team_b.iter() {
                                                        player_data(
                                                            ui,
                                                            player.role,
                                                            &player.summoner_name,
                                                        );
                                                    }
                                                });
                                            });
                                        }
                                    }
                                });
                                // ui.add_space(0.01 * height);
                                ui.separator();
                            }
                        }
                    }
                });
        });
    }
}

#[allow(unused)]
fn format_time(match_time: i64) -> String {
    let native_time = NaiveDateTime::from_timestamp_opt(match_time, 0).unwrap();
    let time: DateTime<Utc> = DateTime::from_local(native_time, Utc);
    let mut human_time = time.format("%H:%M:%S");
    if human_time.to_string().char_range(0..2) == "00" {
        human_time = time.format("%M:%S");
    }
    human_time.to_string()
}
