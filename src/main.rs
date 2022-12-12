#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::mpsc::Sender;

use eframe::egui;
use tokio::runtime::Runtime;
use ui::Results;

mod graphql;
#[path = "networking/networking.rs"]
mod networking;
mod structs;
mod ui;

#[tokio::main]
async fn main() {
    let rt = Runtime::new().expect("Pain");

    let _enter = rt.enter();

    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        })
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "UGG API TEST",
        native_options,
        Box::new(|cc| Box::new(crate::ui::MyEguiApp::new(cc))),
    );
}

fn match_summaries(
    name: String,
    tx: Sender<Results>,
    ctx: egui::Context,
    role: Option<i64>,
    client: reqwest::Client,
) {
    tokio::spawn(async move {
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
                let _ = tx.send(Results::PlayerSuggestions(Ok(response)));
                ctx.request_repaint();
            }
            Err(error) => {
                let _ = tx.send(Results::PlayerSuggestions(Err(Errors::Request(error))));
                ctx.request_repaint();
            }
        }
    });
}

fn update_player(name: String, tx: Sender<Results>, ctx: egui::Context, client: reqwest::Client) {
    tokio::spawn(async move {
        let request = networking::update_player(name, &client).await;
        match request {
            Ok(response) => {
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

fn player_ranking(name: String, tx: Sender<Results>, ctx: egui::Context, client: reqwest::Client) {
    tokio::spawn(async move {
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

fn player_ranks(name: String, tx: Sender<Results>, ctx: egui::Context, client: reqwest::Client) {
    tokio::spawn(async move {
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

#[derive(Debug)]
pub enum Errors {
    Request(reqwest::Error),
    GenericError,
}
