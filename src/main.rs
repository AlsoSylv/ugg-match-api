use std::sync::mpsc::Sender;

use chrono::{self, DateTime, NaiveDateTime, Utc};
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

fn send_request(name: String, tx: Sender<Results>, ctx: egui::Context, role: i64) {
    tokio::spawn(async move {
        let mut name = name.clone();
        let request =
            networking::fetch_match_summaries(&mut name, "na1", vec![Some(role)], 1).await;
        match request {
            Ok(response) => {
                let _ = tx.send(Results::Result(Ok(response)));
                ctx.request_repaint();
            }
            Err(error) => {
                let _ = tx.send(Results::Result(Err(Errors::Request(error))));
                ctx.request_repaint();
            }
        }
        // let _ = tx.send(request.unwrap());
        // ctx.request_repaint();
    });
}

#[derive(Debug)]
pub enum Errors {
    Request(reqwest::Error),
    GenericError,
}

pub struct MatchSummeryTranslated {
    time: String,
    kda: String,
    kp: String,
}

fn format_time(match_time: i64) -> String {
    let native_time = NaiveDateTime::from_timestamp_opt(match_time, 0).unwrap();
    let time: DateTime<Utc> = DateTime::from_local(native_time, Utc);
    let human_time = time.format("%H:%M:%S");
    if human_time.to_string().split(':').collect::<Vec<&str>>()[0] == "00" {
        let human_time = time.format("%M:%S");
        human_time.to_string()
    } else {
        human_time.to_string()
    }
}
