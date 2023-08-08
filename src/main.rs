#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::{mpsc::Sender, Arc};

use bytes::Bytes;
use eframe::egui;
use tokio::runtime::Handle;
use ui::Results;

mod graphql;
#[path = "networking/networking.rs"]
mod networking;
mod structs;
mod ui;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "UGG API TEST",
        native_options,
        Box::new(|cc| Box::new(ui::MyEguiApp::new(cc))),
    );
}

fn match_summaries(
    name: Arc<String>,
    tx: Sender<Results>,
    ctx: egui::Context,
    role: Option<&i8>,
    client: reqwest::Client,
    handle: &Handle,
) {
    let roles = match role {
        Some(role) => vec![*role],
        None => Vec::new(),
    };

    handle.spawn(async move {
        let request = networking::fetch_match_summaries(name, "na1", roles, 1, client)
            .await
            .map_err(Errors::Request);
        let _ = tx.send(Results::MatchSum(request));
        ctx.request_repaint();
    });
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

fn update_player(
    name: Arc<String>,
    tx: Sender<Results>,
    ctx: egui::Context,
    client: reqwest::Client,
    handle: &Handle,
) {
    handle.spawn(async move {
        let request = networking::update_player(name, client)
            .await
            .map_err(Errors::Request);
        let _ = tx.send(Results::PlayerUpdate(request));
        ctx.request_repaint();
    });
}

fn player_ranking(
    name: Arc<String>,
    tx: Sender<Results>,
    ctx: egui::Context,
    client: reqwest::Client,
    handle: &Handle,
) {
    handle.spawn(async move {
        let request = networking::player_ranking(name, client)
            .await
            .map_err(Errors::Request);
        let _ = tx.send(Results::Ranking(request));
        ctx.request_repaint();
    });
}

fn player_ranks(
    name: Arc<String>,
    tx: Sender<Results>,
    ctx: egui::Context,
    client: reqwest::Client,
    handle: &Handle,
) {
    handle.spawn(async move {
        let request = networking::profile_ranks(name, client)
            .await
            .map_err(Errors::Request);
        let _ = tx.send(Results::ProfileRanks(request));
        ctx.request_repaint();
    });
}

fn player_info(
    name: Arc<String>,
    tx: Sender<Results>,
    ctx: egui::Context,
    client: reqwest::Client,
    handle: &Handle,
) {
    handle.spawn(async move {
        let val = networking::player_info(name, "na1", client.clone()).await;

        if let Ok(info) = &val {
            if let Some(info) = &info.data.profile_player_info {
                let res = get_icon(info.icon_id, client).await;
                let wrapped = Results::PlayerIcon(res.map_err(Errors::Request));
                let _ = tx.send(wrapped);
            }
        }

        let _ = tx.send(Results::PlayerInfo(val.map_err(Errors::Request)));
        ctx.request_repaint();
    });
}

fn get_versions(tx: Sender<Results>, ctx: egui::Context, client: reqwest::Client, handle: &Handle) {
    handle.spawn(async move {
        let res = client
            .get("https://ddragon.leagueoflegends.com/api/versions.json")
            .send()
            .await;
        let res = match res {
            Ok(val) => val.json().await,
            Err(err) => Err(err),
        };

        let _ = tx.send(Results::Versions(res.map_err(Errors::Request)));
        ctx.request_repaint();
    });
}

fn get_champ_info(
    version: &str,
    tx: Sender<Results>,
    ctx: egui::Context,
    client: reqwest::Client,
    handle: &Handle,
) {
    let url = format!("http://ddragon.leagueoflegends.com/cdn/{version}/data/en_US/champion.json");
    handle.spawn(async move {
        let res = client.get(url).send().await;

        let res = match res {
            Ok(val) => val.json().await,
            Err(err) => Err(err),
        };

        let _ = tx.send(Results::ChampJson(res.map_err(Errors::Request)));
        ctx.request_repaint();
    });
}

fn get_champ_image(
    version: &str,
    key: &str,
    id: i64,
    tx: Sender<Results>,
    ctx: egui::Context,
    client: reqwest::Client,
    handle: &Handle,
) {
    let url = format!("http://ddragon.leagueoflegends.com/cdn/{version}/img/champion/{key}.png");
    handle.spawn(async move {
        let res = client.get(url).send().await;

        let res = match res {
            Ok(val) => val.bytes().await.map(|bytes| (bytes, id)),
            Err(err) => Err(err),
        };

        let _ = tx.send(Results::ChampImage(res.map_err(Errors::Request)));
        ctx.request_repaint();
    });
}

async fn get_icon(id: i64, client: reqwest::Client) -> Result<Bytes, reqwest::Error> {
    let res = client
        .get(format!(
            "http://ddragon.leagueoflegends.com/cdn/13.14.1/img/profileicon/{id}.png"
        ))
        .send()
        .await?;

    res.bytes().await
}

#[derive(Debug)]
pub enum Errors {
    Request(reqwest::Error),
    GenericError,
}
