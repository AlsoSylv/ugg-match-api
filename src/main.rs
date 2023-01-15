use std::{str::FromStr, sync::mpsc::{Sender, Receiver}};

use chrono::{self, DateTime, NaiveDateTime, Utc};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::json;
use eframe::egui;
use tokio::runtime::Runtime;

mod structs;

/*
    let mut roles = Vec::new();
    let role_map = HashMap::from([
        ("Top", 4),
        ("Jungle", 1),
        ("Mid", 5),
        ("ADC", 3),
        ("Support", 2),
        ("None", 6),
    ]);
    let vec_roles: Vec<&str> = vec![];
    for x in vec_roles {
        roles.push(*(role_map.get(x).unwrap()))
    }
    request("xayah na", "na1", roles, 1).await;
 */

#[tokio::main]
async fn main() {
    let rt = Runtime::new().expect("Pain");

    let _enter = rt.enter();

    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
            }
        })
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native("UGG API TEST", native_options, Box::new(|cc| Box::new(MyEguiApp::new(cc))));
}

struct MyEguiApp {
    tx: Sender<String>,
    rx: Receiver<String>,

    name: String,
    time: String,
}

impl Default for MyEguiApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        Self { 
            tx, 
            rx, 
            name: Default::default(),
            time: Default::default(),
        }
    }
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Ok(pain) = self.rx.try_recv() {
                self.time = pain;
            }
            ui.heading(&self.name);
            ui.heading(&self.time);
            ui.horizontal(|ui| {
                if ui.text_edit_singleline(&mut self.name).changed() {
                    send_request(self.name.clone(), self.tx.clone(), ctx.clone());
                }
            })
        });
    }
 }

fn send_request(name: String, tx: Sender<String>, ctx: egui::Context) {
    tokio::spawn(async move {
        let request = request(&name, "na1", vec![], 1).await;
        let _ = tx.send(request.unwrap());
        ctx.request_repaint();
    });
}

async fn request(name: &str, region_id: &str, role: Vec<i32>, page: i16) -> Result<String, reqwest::Error> {
    let json = json!(
        {
        "operationName": "FetchMatchSummaries",
        "variables": {
            "regionId": region_id,
            "summonerName": name,
            "queueType": [],
            "duoName": "",
            "role": role,
            "seasonIds": [
                18
            ],
            "championId": [],
            "page": page
        },
        "query": "query FetchMatchSummaries(
            $championId: [Int], 
            $page: Int, 
            $queueType: [Int], 
            $duoName: String, 
            $regionId: String!, 
            $role: [Int], 
            $seasonIds: [Int]!, 
            $summonerName: String!
            ) {
            fetchPlayerMatchSummaries(
                championId: $championId
                page: $page
                queueType: $queueType
                duoName: $duoName
                regionId: $regionId
                role: $role
                seasonIds: $seasonIds
                summonerName: $summonerName) {
                    finishedMatchSummaries
                    totalNumMatches
                    matchSummaries {
                        assists
                        championId
                        cs
                        damage
                        deaths
                        gold
                        items
                        jungleCs
                        killParticipation
                        kills
                        level
                        matchCreationTime
                        matchDuration
                        matchId
                        maximumKillStreak
                        primaryStyle
                        queueType
                        regionId
                        role
                        runes
                        subStyle
                        summonerName
                        summonerSpells
                        psHardCarry
                        psTeamPlay
                        lpInfo {
                            lp
                            placement
                            promoProgress
                            promoTarget
                            promotedTo {
                                tier
                                rank
                                __typename
                            }
                        __typename
                        }
                        teamA {
                            championId
                            summonerName
                            teamId
                            role
                            hardCarry
                            teamplay
                            __typename
                        }
                        teamB {
                            championId
                            summonerName
                            teamId
                            role
                            hardCarry
                            teamplay
                            __typename
                        }
                        version
                        visionScore
                        win
                        __typename
                    }
                    __typename
                }
            }"
        }
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_str("Accept-Language").unwrap(),
        HeaderValue::from_str("en-US").unwrap(),
    );
    headers.insert(
        HeaderName::from_str("content-type").unwrap(),
        HeaderValue::from_str("application/json").unwrap(),
    );
    let client = reqwest::Client::new();
    let request = client
        .post("https://u.gg/api")
        .headers(headers)
        .json(&json)
        .send()
        .await;
    match request {
        Ok(reponse) => {
            let text: Result<structs::Root, reqwest::Error> = reponse.json().await;
            match text {
                Ok(json) => {
                    let summeries = json.data.fetch_player_match_summaries.match_summaries;
                    let last_match = &summeries[0];

                    let kda = format!(
                        "{}/{}/{}",
                        last_match.kills, last_match.deaths, last_match.assists
                    );
                    let gold = last_match.gold;
                    let kp = format!("{}%", last_match.kill_participation);

                    let time = format_time(last_match.match_duration);
                    println!("{}", time);
                    println!("{}", kda);
                    println!("{:#}", gold);
                    println!("{}", kp);
                    Ok(time)
                }
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error),
    }
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
