use crate::{send_request, Errors, MatchSummeryTranslated};
use eframe::egui;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

pub enum Results {
    Result(Result<MatchSummeryTranslated, Errors>),
    _String(String),
}

pub struct MyEguiApp {
    tx: Sender<Results>,
    rx: Receiver<Results>,

    name: String,
    time: Option<String>,
    role: String,
    roles_map: HashMap<String, i64>,
}

static ROLES: [&str; 6] = ["Top", "Jungle", "Mid", "ADC", "Support", "None"];

impl Default for MyEguiApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let roles_map = HashMap::from([
            ("Top".to_owned(), 4),
            ("Jungle".to_owned(), 1),
            ("Mid".to_owned(), 5),
            ("ADC".to_owned(), 3),
            ("Support".to_owned(), 2),
            ("None".to_owned(), 6),
        ]);

        Self {
            tx,
            rx,
            name: Default::default(),
            time: Default::default(),
            role: Default::default(),
            roles_map,
        }
    }
}

impl MyEguiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Ok(pain) = self.rx.try_recv() {
                if let Results::Result(Ok(pain)) = pain {
                    self.time = Some(format!(
                        "KDA: {}    KP: {}    TIME: {}",
                        pain.kda, pain.kp, pain.time
                    ));
                } else {
                    self.time = None;
                };
            }

            ui.heading(&self.name);
            match &self.time {
                Some(a) => {
                    ui.heading(a);
                }
                None => {
                    ui.spinner();
                }
            }

            ui.horizontal(|ui| {
                egui::ComboBox::from_label(format!("Role: {}", self.role))
                    .selected_text(self.role.clone())
                    .show_ui(ui, |ui| {
                        for option in ROLES {
                            ui.selectable_value(&mut self.role, option.to_string(), option);
                        }
                    });

                if ui.button("WAAAA").clicked() {
                    let role = &self.role;
                    let role = self.roles_map.get(role as &str).unwrap();
                    send_request(self.name.clone(), self.tx.clone(), ctx.clone(), *role);
                }
            });
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.name);
            })
        });
    }
}
