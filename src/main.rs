#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, OnceLock, RwLock};

use async_channel::{Receiver, Sender};
use bytes::Bytes;
use eframe::{
    egui::TextureOptions,
    epaint::{ColorImage, TextureHandle},
};
use std::collections::HashMap;
use structs::ChampionJson;
use tokio::runtime::Runtime;
use ui::{Champ, Payload, Results};

mod graphql;
#[path = "networking/networking.rs"]
mod networking;
mod structs;
mod ui;
mod ui_logic;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "UGG API TEST",
        native_options,
        Box::new(|cc| Box::new(ui::MyEguiApp::new(cc))),
    );
}

pub struct SharedState {
    // This is initilized once, and because of the way the GUI is setup, will always be there afterwards
    champs: OnceLock<HashMap<i64, Champ>>,
    versions: OnceLock<Box<[String]>>,
    player_icons: RwLock<HashMap<i16, TextureHandle>>,
}

impl SharedState {
    fn new() -> Self {
        Self {
            champs: OnceLock::new(),
            versions: OnceLock::new(),
            player_icons: RwLock::new(HashMap::new()),
        }
    }

    async fn update_champ_image(&self, champ_id: i64, texture: eframe::egui::TextureHandle) {
        let map = self.champs.get().unwrap();
        let handle = map
            .get(&champ_id)
            .expect("The map is already loaded by now");
        let mut write = handle.image.write().unwrap();
        *write = Some(texture);
    }
}

#[derive(Clone)]
struct ThreadState {
    ctx: eframe::egui::Context,
    receiver: Receiver<Payload>,
    sender: Sender<Results>,
    client: reqwest::Client,
}

async fn message_sender(
    message: Results,
    ctx: &eframe::egui::Context,
    thread_sender: &Sender<Results>,
) {
    thread_sender.send(message).await.unwrap();
    ctx.request_repaint();
}

pub fn spawn_gui_shit(
    _ctx: &eframe::egui::Context,
) -> (
    Runtime,
    Sender<Payload>,
    Receiver<Results>,
    Arc<SharedState>,
) {
    let shared_state = Arc::new(SharedState::new());

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let (gui_sender, thread_receiver) = async_channel::unbounded::<Payload>();
    let (thread_sender, gui_receiver) = async_channel::unbounded();
    let client = reqwest::Client::new();

    let state = ThreadState {
        ctx: _ctx.clone(),
        receiver: thread_receiver,
        sender: thread_sender,
        client,
    };

    let runtime_loop = || {
        let state = state.clone();
        let shared_state = shared_state.clone();

        async move {
            while let Ok(message) = state.receiver.recv().await {
                match message {
                    ui::Payload::MatchSummaries {
                        name,
                        roles,
                        region_id,
                        page,
                    } => {
                        let request = networking::fetch_match_summaries(
                            name,
                            region_id,
                            roles.map_or_else(Vec::new, |role| vec![role]),
                            page as i64,
                            &state.client,
                        )
                        .await
                        .map_err(Errors::Request);

                        message_sender(Results::MatchSum(request), &state.ctx, &state.sender).await;
                    }
                    ui::Payload::PlayerRanks { name, region_id } => {
                        let request = networking::profile_ranks(name, &state.client, region_id)
                            .await
                            .map_err(Errors::Request);

                        message_sender(Results::ProfileRanks(request), &state.ctx, &state.sender)
                            .await;
                    }
                    ui::Payload::UpdatePlayer { name, region_id } => {
                        let request = networking::update_player(name, &state.client, region_id)
                            .await
                            .map_err(Errors::Request);

                        message_sender(Results::PlayerUpdate(request), &state.ctx, &state.sender)
                            .await;
                    }
                    ui::Payload::PlayerRanking { name, region_id } => {
                        let request = networking::player_ranking(name, &state.client, region_id)
                            .await
                            .map_err(Errors::Request);

                        message_sender(Results::Ranking(request), &state.ctx, &state.sender).await;
                    }
                    ui::Payload::PlayerInfo {
                        name,
                        version_index,
                        region_id,
                    } => {
                        let val = networking::player_info(name, region_id, &state.client).await;

                        if let Ok(info) = &val {
                            if let Some(info) = &info.data.profile_player_info {
                                let res = get_icon(
                                    info.icon_id,
                                    &shared_state.versions.get().unwrap()[version_index],
                                    &state.client,
                                )
                                .await;
                                match res {
                                    Ok(bytes) => {
                                        let mut decoder = png::Decoder::new(&*bytes);
                                        let headers = decoder.read_header_info().expect(
                                            "This is always a PNG, so this shouldn't ever fail",
                                        );

                                        let x = headers.height as usize;
                                        let y = headers.width as usize;

                                        let mut reader = decoder.read_info().unwrap();

                                        let mut buf = vec![0; reader.output_buffer_size()];

                                        reader.next_frame(&mut buf).unwrap();

                                        let texture = state.ctx.load_texture(
                                            "icon",
                                            ColorImage::from_rgb([x, y], &buf),
                                            TextureOptions::LINEAR,
                                        );
                                        let _ = shared_state
                                            .player_icons
                                            .write()
                                            .unwrap()
                                            .insert(info.icon_id, texture);
                                    }
                                    Err(err) => {
                                        let wrapped = Results::PlayerIcon(Errors::Request(err));
                                        message_sender(wrapped, &state.ctx, &state.sender).await;
                                    }
                                }
                            }
                        }

                        message_sender(
                            Results::PlayerInfo(val.map_err(Errors::Request)),
                            &state.ctx,
                            &state.sender,
                        )
                        .await;
                    }
                    ui::Payload::GetVersions => {
                        let res = state
                            .client
                            .get("https://ddragon.leagueoflegends.com/api/versions.json")
                            .send()
                            .await;

                        let res = match res {
                            Ok(val) => val.json().await,
                            Err(err) => Err(err),
                        };

                        match res {
                            Ok(json) => {
                                shared_state.versions.get_or_init(|| json);
                            }
                            Err(err) => {
                                message_sender(
                                    Results::Versions(Errors::Request(err)),
                                    &state.ctx,
                                    &state.sender,
                                )
                                .await;
                            }
                        };
                    }
                    ui::Payload::GetChampInfo { url } => {
                        let res = state.client.get(url).send().await;

                        let res = match res {
                            Ok(res) => res,
                            Err(err) => {
                                message_sender(
                                    Results::ChampJson(Errors::Request(err)),
                                    &state.ctx,
                                    &state.sender,
                                )
                                .await;
                                continue;
                            }
                        };

                        let json = res.json::<ChampionJson>().await;
                        let json = match json {
                            Ok(json) => json,
                            Err(err) => {
                                message_sender(
                                    Results::ChampJson(Errors::Request(err)),
                                    &state.ctx,
                                    &state.sender,
                                )
                                .await;
                                continue;
                            }
                        };

                        let mut champs: HashMap<i64, Champ> = HashMap::with_capacity(200);
                        for (_, data) in json.data {
                            let id: i64 = data.id.parse().unwrap();
                            champs.insert(id, data.into());
                        }
                        shared_state.champs.get_or_init(|| champs);
                    }
                    ui::Payload::GetChampImage { url, id } => {
                        let res = state.client.get(url).send().await;
                        let res = match res {
                            Ok(res) => res,
                            Err(err) => {
                                message_sender(
                                    Results::ChampImage(Errors::Request(err)),
                                    &state.ctx,
                                    &state.sender,
                                )
                                .await;
                                continue;
                            }
                        };
                        let bytes = &*match res.bytes().await {
                            Ok(bytes) => bytes,
                            Err(err) => {
                                message_sender(
                                    Results::ChampImage(Errors::Request(err)),
                                    &state.ctx,
                                    &state.sender,
                                )
                                .await;
                                continue;
                            }
                        };

                        let mut decoder = png::Decoder::new(bytes);
                        let headers = decoder
                            .read_header_info()
                            .map_err(|err| println!("{:?}", err))
                            .expect("This is always a PNG, so this shouldn't ever fail");

                        let x = headers.height as usize;
                        let y = headers.width as usize;

                        let mut reader = decoder.read_info().unwrap();
                        let mut buf = vec![0; reader.output_buffer_size()];

                        reader
                            .next_frame(&mut buf)
                            .expect("If the champ does not exist in the map, something is wrong");

                        let texture = state.ctx.load_texture(
                            "icon",
                            ColorImage::from_rgb([x, y], &buf),
                            TextureOptions::LINEAR,
                        );

                        shared_state.update_champ_image(id, texture).await;
                    }
                    ui::Payload::GetMatchDetails {
                        name,
                        id,
                        region_id,
                        version,
                    } => {
                        let res = networking::fetch_match(
                            name,
                            region_id,
                            &id.to_string(),
                            &version,
                            &state.client,
                        )
                        .await
                        .map(|json| (Box::new(json), id))
                        .map_err(Errors::Request);
                        message_sender(Results::MatchDetails(res), &state.ctx, &state.sender).await;
                    }
                };
            }
        }
    };

    runtime.spawn(runtime_loop());
    runtime.spawn(runtime_loop());
    runtime.spawn(runtime_loop());
    runtime.spawn(runtime_loop());

    (runtime, gui_sender, gui_receiver, shared_state.clone())
}

// Note: This is unsused because the searchbar is broken, but I'm hoping it gets fixed one day
// fn player_suggestions(
//     name: Arc<String>,
//     tx: Sender<Results>,
//     ctx: egui::Context,
//     client: reqwest::Client,
// ) {
//     tokio::spawn(async move {
//         let request = networking::player_suggestiosn(name, &state.client).await;
//         match request {
//             Ok(response) => {
//                 // let _ = tx.send(Results::PlayerSuggestions(Ok(response)));
//                 ctx.request_repaint();
//             }
//             Err(error) => {
//                 // let _ = tx.send(Results::PlayerSuggestions(Err(Errors::Request(error))));
//                 ctx.request_repaint();
//             }
//         }
//     });
// }

async fn get_icon(
    id: i16,
    version: &str,
    client: &reqwest::Client,
) -> Result<Bytes, reqwest::Error> {
    let res = client
        .get(format!(
            "http://ddragon.leagueoflegends.com/cdn/{version}/img/profileicon/{id}.png"
        ))
        .send()
        .await?;

    res.bytes().await
}

#[derive(Debug)]
pub enum Errors {
    Request(reqwest::Error),
}

impl ToString for Errors {
    fn to_string(&self) -> String {
        match self {
            Errors::Request(err) => err.to_string(),
        }
    }
}
