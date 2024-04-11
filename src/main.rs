#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::BTreeMap,
    sync::{OnceLock, RwLock},
};

use async_channel::{Receiver, Sender};
use bytes::Bytes;
use eframe::{
    egui::TextureOptions,
    epaint::{ColorImage, TextureHandle},
};
use std::collections::HashMap;
use std::fmt::Display;
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

impl SharedState {
    const fn new() -> Self {
        Self {
            champs: OnceLock::new(),
            versions: OnceLock::new(),
            player_icons: RwLock::new(BTreeMap::new()),
        }
    }

    fn update_champ_image(&self, champ_id: i64, texture: TextureHandle) {
        let map = self.champs.get().unwrap();
        let handle = map
            .get(&champ_id)
            .expect("The map is already loaded by now");
        let mut write = handle.image.write().unwrap();
        *write = Some(texture);
    }
}

static SHARED_STATE: SharedState = SharedState::new();

pub struct SharedState {
    // This is initialized once, and because of the way the GUI is set up, will always be there afterward
    champs: OnceLock<HashMap<i64, Champ>>,
    versions: OnceLock<Box<[String]>>,
    player_icons: RwLock<BTreeMap<i16, TextureHandle>>,
}

struct ThreadState {
    ctx: OnceLock<eframe::egui::Context>,
    receiver: OnceLock<Receiver<Payload>>,
    sender: OnceLock<Sender<Results>>,
    client: OnceLock<reqwest::Client>,
}

static STATE: ThreadState = ThreadState {
    ctx: OnceLock::new(),
    receiver: OnceLock::new(),
    sender: OnceLock::new(),
    client: OnceLock::new(),
};

impl ThreadState {
    fn ctx(&self) -> &eframe::egui::Context {
        self.ctx.get().unwrap()
    }

    fn receiver(&self) -> &Receiver<Payload> {
        self.receiver.get().unwrap()
    }

    fn sender(&self) -> &Sender<Results> {
        self.sender.get().unwrap()
    }

    fn client(&self) -> &reqwest::Client {
        self.client.get().unwrap()
    }
}

async fn try_message_sender<T>(
    result: Result<T, Results>,
    ctx: &eframe::egui::Context,
    thread_sender: &Sender<Results>,
) -> Option<T> {
    match result {
        Ok(t) => Some(t),
        Err(e) => {
            message_sender(e, ctx, thread_sender).await;
            None
        }
    }
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
) -> (Runtime, Sender<Payload>, Receiver<Results>) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let (gui_sender, thread_receiver) = async_channel::unbounded::<Payload>();
    let (thread_sender, gui_receiver) = async_channel::unbounded();

    STATE.receiver.get_or_init(|| thread_receiver);
    STATE.ctx.get_or_init(|| _ctx.clone());
    STATE.sender.get_or_init(|| thread_sender);
    STATE.client.get_or_init(reqwest::Client::new);

    let runtime_loop = || {
        async move {
            let state = &STATE;
            let shared_state = &SHARED_STATE;

            while let Ok(message) = state.receiver().recv().await {
                match message {
                    Payload::MatchSummaries {
                        name,
                        tag_line,
                        roles,
                        region_id,
                        page,
                    } => {
                        let request = networking::fetch_match_summaries(
                            &name,
                            &tag_line,
                            region_id,
                            roles.as_slice(),
                            page,
                            state.client(),
                        )
                        .await
                        .map_err(Errors::Request);

                        message_sender(Results::MatchSum(request), state.ctx(), state.sender())
                            .await;
                    }
                    Payload::UpdatePlayer {
                        name,
                        tag_line,
                        region_id,
                    } => {
                        let request =
                            networking::update_player(&name, &tag_line, state.client(), region_id)
                                .await
                                .map_err(Errors::Request);

                        message_sender(Results::PlayerUpdate(request), state.ctx(), state.sender())
                            .await;
                    }
                    Payload::PlayerRanking {
                        name,
                        tag_line,
                        region_id,
                    } => {
                        let request =
                            networking::player_ranking(&name, &tag_line, state.client(), region_id)
                                .await
                                .map_err(Errors::Request);

                        message_sender(Results::Ranking(request), state.ctx(), state.sender())
                            .await;
                    }
                    Payload::PlayerInfo {
                        name,
                        tag_line,
                        version_index,
                        region_id,
                    } => {
                        let val =
                            networking::player_info(name, tag_line, region_id, state.client())
                                .await
                                .map_err(|e| Results::PlayerInfo(Err(e.into())));

                        if let Some(info) = try_message_sender(val, state.ctx(), state.sender()).await {
                            if let Some(info) = &info.data.profile_init_simple {
                                let res = get_icon(
                                    info.player_info.icon_id,
                                    &shared_state.versions.get().unwrap()[version_index],
                                    state.client(),
                                )
                                .await
                                .map_err(|e| Results::PlayerIcon(e.into()));

                                if let Some(bytes) =
                                    try_message_sender(res, state.ctx(), state.sender()).await
                                {
                                    let mut decoder = png::Decoder::new(&*bytes);
                                    let headers = decoder.read_header_info().expect(
                                        "This is always a PNG, so this shouldn't ever fail",
                                    );

                                    let x = headers.height as usize;
                                    let y = headers.width as usize;

                                    let mut reader = decoder.read_info().unwrap();

                                    let mut buf = vec![0; reader.output_buffer_size()];

                                    reader.next_frame(&mut buf).unwrap();

                                    let texture = state.ctx().load_texture(
                                        "icon",
                                        ColorImage::from_rgb([x, y], &buf),
                                        TextureOptions::LINEAR,
                                    );
                                    let mut map = shared_state.player_icons.write().unwrap();
                                    map.insert(info.player_info.icon_id, texture);
                                }
                            }
                        }
                    }
                    Payload::GetVersions => {
                        let res = state
                            .client()
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
                                    state.ctx(),
                                    state.sender(),
                                )
                                .await;
                            }
                        };
                    }
                    Payload::GetChampInfo { url } => {
                        let res = state.client().get(url).send().await;

                        let json = match res {
                            Ok(res) => res.json().await,
                            Err(err) => {
                                message_sender(
                                    Results::ChampJson(Errors::Request(err)),
                                    state.ctx(),
                                    state.sender(),
                                )
                                .await;
                                continue;
                            }
                        };
                        let json: ChampionJson = match json {
                            Ok(json) => json,
                            Err(err) => {
                                message_sender(
                                    Results::ChampJson(Errors::Request(err)),
                                    state.ctx(),
                                    state.sender(),
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
                    Payload::GetChampImage { url, id } => {
                        // TODO: Check the returned data is a valid image
                        let res = state.client().get(url).send().await;
                        let res = match res {
                            Ok(res) => res,
                            Err(err) => {
                                message_sender(
                                    Results::ChampImage(err.into()),
                                    state.ctx(),
                                    state.sender(),
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
                                    state.ctx(),
                                    state.sender(),
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

                        let texture = state.ctx().load_texture(
                            "icon",
                            ColorImage::from_rgb([x, y], &buf),
                            TextureOptions::LINEAR,
                        );

                        shared_state.update_champ_image(id, texture);
                    }
                    Payload::GetMatchDetails {
                        name,
                        tag_line,
                        id,
                        region_id,
                        version,
                    } => {
                        let res = networking::fetch_match(
                            &name,
                            &tag_line,
                            region_id,
                            &id.to_string(),
                            &version,
                            state.client(),
                        )
                        .await
                        .map(|json| (Box::new(json), id))
                        .map_err(Errors::Request);
                        message_sender(Results::MatchDetails(res), state.ctx(), state.sender())
                            .await;
                    }
                    Payload::GetPlayerSuggestions { name } => {
                        let res = networking::player_suggestions(name, state.client())
                            .await
                            .map_err(Errors::Request);
                        message_sender(
                            Results::PlayerSuggestions(res),
                            state.ctx(),
                            state.sender(),
                        )
                        .await;
                    }
                };
            }
        }
    };

    runtime.spawn(runtime_loop());
    runtime.spawn(runtime_loop());
    runtime.spawn(runtime_loop());
    runtime.spawn(runtime_loop());

    (runtime, gui_sender, gui_receiver)
}

//noinspection SpellCheckingInspection
async fn get_icon(
    id: i16,
    version: &str,
    client: &reqwest::Client,
) -> Result<Bytes, reqwest::Error> {
    let res = client
        .get(format!(
            "https://ddragon.leagueoflegends.com/cdn/{version}/img/profileicon/{id}.png"
        ))
        .send()
        .await?;

    res.bytes().await
}

#[derive(Debug)]
pub enum Errors {
    Request(reqwest::Error),
}

impl From<reqwest::Error> for Errors {
    fn from(value: reqwest::Error) -> Self {
        Errors::Request(value)
    }
}

impl Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Errors::Request(err) => err.to_string(),
            }
        )
    }
}
