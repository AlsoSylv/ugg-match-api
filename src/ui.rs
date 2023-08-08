use crate::structs::{self, ChampionJson, MatchSummary, OverallRanking, RankScore};
use crate::{
    get_champ_image, get_champ_info, get_versions, match_summaries, player_info, player_ranking,
    player_ranks, update_player, Errors,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use eframe::egui::{self, TextBuffer, TextureOptions, Vec2};
use eframe::epaint::ColorImage;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use tokio::runtime::{Builder, Runtime};

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

/// TODO: Store player data in a sub struct
pub struct MyEguiApp {
    tx: Sender<Results>,
    rx: Receiver<Results>,

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

    client: reqwest::Client,
    runtime: Runtime,
    id_name_champ_map: HashMap<i64, Champion>,
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
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let runtime = Builder::new_multi_thread()
            .worker_threads(2)
            .enable_io()
            .enable_time()
            .build()
            .unwrap();

        let (tx, rx) = std::sync::mpsc::channel();
        let roles_map = HashMap::from([
            ("Top".to_owned(), 4),
            ("Jungle".to_owned(), 1),
            ("Mid".to_owned(), 5),
            ("ADC".to_owned(), 3),
            ("Support".to_owned(), 2),
        ]);

        Self {
            tx,
            rx,
            active_player: Default::default(),
            role: 5,
            roles_map,
            summeries: None,
            rank: None,
            ranking: None,
            refresh_enabled: false,
            update_enabled: false,
            client: reqwest::Client::new(),
            player_icon: None,
            runtime,
            data_dragon: DataDragon {
                versions: None,
                ver_started: false,
                champ_json: None,
            },
            id_name_champ_map: HashMap::with_capacity(200),
        }
    }

    /// This long line of function calls, well looking like bullshit
    /// actually drives the entire state of the GUI to change!
    fn update_matches(&self, ctx: &egui::Context, name: Arc<String>) {
        match_summaries(
            name.clone(),
            self.tx.clone(),
            ctx.clone(),
            self.roles_map.get(ROLES[self.role as usize]),
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

    fn match_page(&mut self, summary: &MatchSummary, ui: &mut egui::Ui, ctx: &egui::Context) {
        let champ = self
            .id_name_champ_map
            .get_mut(&summary.champion_id)
            .unwrap();

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
                        get_champ_image(
                            &self.data_dragon.versions.as_ref().unwrap()[0],
                            champ.key(),
                            summary.champion_id,
                            self.tx.clone(),
                            ctx.clone(),
                            self.client.clone(),
                            self.runtime.handle(),
                        );
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

        update_player(
            name.clone(),
            self.tx.clone(),
            ctx.clone(),
            self.client.clone(),
            self.runtime.handle(),
        );
        self.update_matches(ctx, name.clone());
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
            get_versions(
                self.tx.clone(),
                ctx.clone(),
                self.client.clone(),
                self.runtime.handle(),
            );

            self.data_dragon.ver_started = true;
        }

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
                        let mut data = rank.data.fetch_profile_ranks.rank_scores;
                        let new_data: Vec<RankScore> = data
                            .drain(..)
                            .filter_map(|rank| {
                                if rank.queue_type.is_empty() {
                                    None
                                } else {
                                    Some(rank)
                                }
                            })
                            .collect();
                        self.rank = Some(new_data);
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
                        get_champ_info(
                            &versions[0],
                            self.tx.clone(),
                            ctx.clone(),
                            self.client.clone(),
                            self.runtime.handle(),
                        );
                        self.data_dragon.versions = Some(versions);
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::ChampJson(json) => match json {
                    Ok(json) => {
                        for data in json.data.values() {
                            let id: i64 = data.key.parse().unwrap();
                            self.id_name_champ_map
                                .insert(id, Champion::new(&data.name, &data.id));
                        }
                        self.data_dragon.champ_json = Some(json)
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

                        reader.next_frame(&mut buf).unwrap();

                        let texture = ctx.load_texture(
                            "icon",
                            ColorImage::from_rgb([x, y], &buf),
                            TextureOptions::LINEAR,
                        );

                        let handle = self.id_name_champ_map.get_mut(&id).unwrap();

                        handle.set_image(Some(texture));
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

        match &self.data_dragon.versions {
            Some(_) => match &self.data_dragon.champ_json {
                Some(_) => {
                    egui::SidePanel::left("side_panel")
                        .resizable(false)
                        .show(ctx, |ui| {
                            ui.vertical(|ui| {
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    ui.label("Player: ");

                                    let search_bar =
                                        ui.text_edit_singleline(&mut self.active_player);

                                    if search_bar.clicked() {
                                        if !self.active_player.is_empty() {
                                            self.update_matches(
                                                ctx,
                                                Arc::new(self.active_player.clone()),
                                            );
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
                                                ui.selectable_value(
                                                    &mut self.role,
                                                    index as u8,
                                                    *text,
                                                );
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

                                egui::TopBottomPanel::bottom("bottom_panel").show_inside(
                                    ui,
                                    |ui| {
                                        ui.add_space(5.0);
                                        ui.with_layout(
                                            egui::Layout::left_to_right(egui::Align::LEFT),
                                            |ui| {
                                                egui::widgets::global_dark_light_mode_switch(ui);
                                            },
                                        );
                                    },
                                );
                            });
                        });

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            ui.horizontal(|ui| {
                                if let Some(texture) = &self.player_icon {
                                    ui.image(texture, Vec2::splat(50.0));
                                } else if !self.active_player.is_empty() && self.summeries.is_some()
                                {
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
                                                self.match_page(summary, ui, ctx);
                                                ui.separator();
                                            }
                                        }
                                    });
                            }

                            self.summeries = summeries;
                        });
                    });
                }
                None => {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.spinner();
                    });
                }
            },
            None => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.spinner();
                });
            }
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
