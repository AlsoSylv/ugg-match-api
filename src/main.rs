#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bytes::Bytes;
use ui::{Message, Results};

mod graphql;
#[path = "networking/networking.rs"]
mod networking;
mod structs;
mod ui;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let (gui_sender, thread_receiver) = async_channel::unbounded::<Message>();
    let (thread_sender, gui_receiver) = async_channel::unbounded();
    let client = reqwest::Client::new();

    let runtime_loop = || {
        let thread_receiver = thread_receiver.clone();
        let thread_sender = thread_sender.clone();
        let client = client.clone();
        async move {
            while let Ok(message) = thread_receiver.recv().await {
                let ctx = message.ctx;
                let message = match message.payload {
                    ui::Payload::MatchSummaries { name, roles } => {
                        let request = networking::fetch_match_summaries(
                            name,
                            "na1",
                            roles,
                            1,
                            client.clone(),
                        )
                        .await
                        .map_err(Errors::Request);

                        Results::MatchSum(request)
                    }
                    ui::Payload::PlayerRanks { name } => {
                        let request = networking::profile_ranks(name, client.clone())
                            .await
                            .map_err(Errors::Request);

                        Results::ProfileRanks(request)
                    }
                    ui::Payload::UpdatePlayer { name } => {
                        let request = networking::update_player(name, client.clone())
                            .await
                            .map_err(Errors::Request);

                        Results::PlayerUpdate(request)
                    }
                    ui::Payload::PlayerRanking { name } => {
                        let request = networking::player_ranking(name, client.clone())
                            .await
                            .map_err(Errors::Request);

                        Results::Ranking(request)
                    }
                    ui::Payload::PlayerInfo {
                        name,
                        version,
                        version_index,
                    } => {
                        let val = networking::player_info(name, "na1", client.clone()).await;

                        if let Ok(info) = &val {
                            if let Some(info) = &info.data.profile_player_info {
                                let res =
                                    get_icon(info.icon_id, &version[version_index], client.clone())
                                        .await;
                                let wrapped = Results::PlayerIcon(res.map_err(Errors::Request));
                                thread_sender.send(wrapped).await.unwrap();
                            }
                        }

                        Results::PlayerInfo(val.map_err(Errors::Request))
                    }
                    ui::Payload::GetVersions => {
                        println!("Help me");
                        let res = client
                            .get("https://ddragon.leagueoflegends.com/api/versions.json")
                            .send()
                            .await;

                        println!("Request Finished!");

                        let res = match res {
                            Ok(val) => val.json().await,
                            Err(err) => Err(err),
                        };

                        Results::Versions(res.map_err(Errors::Request))
                    }
                    ui::Payload::GetChampInfo { url } => {
                        let res = client.get(url).send().await;

                        let res = match res {
                            Ok(val) => val.json().await,
                            Err(err) => Err(err),
                        };

                        Results::ChampJson(res.map_err(Errors::Request))
                    }
                    ui::Payload::GetChampImage { url, id } => {
                        let res = client.get(url).send().await;

                        let res = match res {
                            Ok(val) => val.bytes().await.map(|bytes| (bytes, id)),
                            Err(err) => Err(err),
                        };

                        Results::ChampImage(res.map_err(Errors::Request))
                    }
                    ui::Payload::GetMatchDetails { name, version, id } => {
                        let id_i64: i64 = id.as_str().parse().unwrap();
                        let res = networking::fetch_match(name, "na1", id, version, client.clone())
                            .await
                            .map(|json| (json, id_i64))
                            .map_err(Errors::Request);

                        Results::MatchDetails(Box::new(res))
                    }
                };

                thread_sender.send(message).await.unwrap();
                ctx.request_repaint();
            }
        }
    };

    runtime.spawn(runtime_loop());
    runtime.spawn(runtime_loop());
    runtime.spawn(runtime_loop());
    runtime.spawn(runtime_loop());

    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "UGG API TEST",
        native_options,
        Box::new(|cc| Box::new(ui::MyEguiApp::new(cc, gui_sender, gui_receiver))),
    );
}

// Note: This is unsused because the searchbar is broken, but I'm hoping it gets fixed one day
// fn player_suggestions(
//     name: Arc<String>,
//     tx: Sender<Results>,
//     ctx: egui::Context,
//     client: reqwest::Client,
// ) {
//     tokio::spawn(async move {
//         let request = networking::player_suggestiosn(name, &client).await;
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
    id: i64,
    version: &str,
    client: reqwest::Client,
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
