#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::mpsc::Sender;

use eframe::egui;
use serde_json::json;
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
    name: String,
    tx: Sender<Results>,
    ctx: egui::Context,
    role: Option<i64>,
    client: reqwest::Client,
    handle: &Handle,
) {
    handle.spawn(async move {
        let role = match role {
            Some(int) => vec![int],
            None => Vec::new(),
        };
        let request = networking::fetch_match_summaries(name, "na1", role, 1, &client).await;
        match request {
            Ok(response) => {
                let _ = tx.send(Results::MatchSum(Ok(response)));
                ctx.request_repaint();
            }
            Err(error) => {
                let _ = tx.send(Results::MatchSum(Err(Errors::Request(error))));
                ctx.request_repaint();
            }
        }
    });
}

/// Note: This is unsused because the searchbar is broken, but I'm hoping it gets fixed one day
#[allow(unused)]
fn player_suggestions(
    name: String,
    tx: Sender<Results>,
    ctx: egui::Context,
    client: reqwest::Client,
) {
    tokio::spawn(async move {
        let request = networking::player_suggestiosn(name, &client).await;
        match request {
            Ok(response) => {
                // let _ = tx.send(Results::PlayerSuggestions(Ok(response)));
                ctx.request_repaint();
            }
            Err(error) => {
                // let _ = tx.send(Results::PlayerSuggestions(Err(Errors::Request(error))));
                ctx.request_repaint();
            }
        }
    });
}

fn update_player(
    name: String,
    tx: Sender<Results>,
    ctx: egui::Context,
    client: reqwest::Client,
    handle: &Handle,
) {
    handle.spawn(async move {
        let request = networking::update_player(name, &client).await;
        match request {
            Ok(response) => {
                println!("{}", json!(response));
                let _ = tx.send(Results::PlayerUpdate(Ok(response)));
                ctx.request_repaint();
            }
            Err(error) => {
                let _ = tx.send(Results::PlayerUpdate(Err(Errors::Request(error))));
                ctx.request_repaint();
            }
        }
    });
}

fn player_ranking(
    name: String,
    tx: Sender<Results>,
    ctx: egui::Context,
    client: reqwest::Client,
    handle: &Handle,
) {
    handle.spawn(async move {
        let request = networking::player_ranking(name, &client).await;
        match request {
            Ok(response) => {
                let _ = tx.send(Results::Ranking(Ok(response)));
                ctx.request_repaint();
            }
            Err(error) => {
                let _ = tx.send(Results::Ranking(Err(Errors::Request(error))));
                ctx.request_repaint();
            }
        }
    });
}

fn player_ranks(
    name: String,
    tx: Sender<Results>,
    ctx: egui::Context,
    client: reqwest::Client,
    handle: &Handle,
) {
    handle.spawn(async move {
        let request = networking::profile_ranks(name, &client).await;
        match request {
            Ok(response) => {
                let _ = tx.send(Results::ProfileRanks(Ok(response)));
                ctx.request_repaint();
            }
            Err(error) => {
                let _ = tx.send(Results::ProfileRanks(Err(Errors::Request(error))));
                ctx.request_repaint();
            }
        }
    });
}

fn player_info(
    name: String,
    tx: Sender<Results>,
    ctx: egui::Context,
    client: reqwest::Client,
    handle: &Handle,
) {
    handle.spawn(async move {
        let val = networking::player_info(name, "na1", &client)
            .await
            .map_err(Errors::Request);
        let _ = tx.send(Results::PlayerInfo(val));
        ctx.request_repaint();
    });
}

fn get_icon(id: i64, tx: Sender<Results>, client: reqwest::Client, handle: &Handle) {
    handle.spawn(async move {
        let val = client
            .get(format!(
                "http://ddragon.leagueoflegends.com/cdn/13.14.1/img/profileicon/{id}.png"
            ))
            .send()
            .await
            .map_err(Errors::Request);

        match val {
            Ok(res) => {
                let fin = Results::PlayerIcon(res.bytes().await.map_err(Errors::Request));
                let _ = tx.send(fin);
            }
            Err(err) => {
                let _ = tx.send(Results::PlayerIcon(Err(err)));
            }
        }
    });
}

#[derive(Debug)]
pub enum Errors {
    Request(reqwest::Error),
    GenericError,
}
