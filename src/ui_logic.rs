use std::{collections::HashMap, sync::Arc};

use eframe::egui::TextBuffer;

use crate::{
    structs::RankScore,
    ui::{self, Champ, Payload, Results},
};

const fn get_role_index(role: u8) -> Option<u8> {
    match role {
        0 => Some(4), // Top
        1 => Some(1), // Jungle
        2 => Some(5), // Mid
        3 => Some(3), // ADC
        4 => Some(2), // Support
        _ => None,    // No role, used to map to an empty vec
    }
}

impl ui::MyEguiApp {
    pub fn send_message(&self, payload: Payload) {
        self.messenger.try_send(payload).unwrap();
    }

    pub fn update_matches(&self, name: &Arc<String>, tag_line: &Arc<String>) {
        // These need to be disabled for now
        self.send_message(Payload::MatchSummaries {
            name: name.clone(),
            tag_line: tag_line.clone(),
            roles: get_role_index(self.role),
            region_id: self.data_dragon.region,
            page: self.page,
        });
        // self.send_message(Payload::PlayerRanks {
        //     name: name.clone(),
        //     region_id: self.data_dragon.region,
        // });
        // self.send_message(Payload::PlayerRanking {
        //     name: name.clone(),
        //     region_id: self.data_dragon.region,
        // });
        self.send_message(Payload::PlayerInfo {
            name: name.clone(),
            tag_line: tag_line.clone(),
            version_index: 0,
            region_id: self.data_dragon.region,
        });
    }

    pub fn update_data(&mut self, versions: &[String], champs: &HashMap<i64, Champ>) {
        if let Ok(receiver) = self.receiver.try_recv() {
            match receiver {
                Results::MatchSum(match_sums) => match match_sums {
                    Ok(matches) => {
                        let data = matches.data.fetch_player_match_summaries;
                        self.finished_match_summaries = data.match_summaries.len() != 20;
                        let mut summaries = data.match_summaries;
                        summaries.iter_mut().for_each(|summary| {
                            if self.player_data.match_data_map.get(&summary.match_id).is_none() {
                                self.player_data.match_data_map.insert(summary.match_id, None);
                                // self.send_message(Payload::GetMatchDetails { name: self.riot_user_name.clone(), version: summary.version.take(), id: summary.match_id, region_id: self.data_dragon.region });
                            }

                            let champ = &champs[&summary.champion_id];
                            if !champ.image_started.load(std::sync::atomic::Ordering::Relaxed) {
                                let key = &champ.key;
                                self.send_message(
                                    Payload::GetChampImage {
                                        url: format!(
                                            "https://ddragon.leagueoflegends.com/cdn/{}/img/champion/{}.png",
                                            versions[0], key
                                        ),
                                        id: summary.champion_id,
                                    },
                                );
                                champ.image_started.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                        });
                        self.player_data.match_summaries = Some(summaries)
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::PlayerUpdate(update) => match update {
                    Ok(updated) => {
                        let data = updated.data.update_player_profile;
                        if data.success {
                            self.update_matches(&self.riot_user_name, &self.riot_tag_line);
                        } else {
                            dbg!("{:?}", data.error_reason);
                        }
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::Ranking(ranking) => match ranking {
                    Ok(ranking) => {
                        self.player_data.ranking = ranking.data.overall_ranking;
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                // Todo: Display this info
                Results::PlayerInfo(info) => match info {
                    Ok(data) => {
                        let info = data.data.profile_init_simple.unwrap();
                        let rank = data.data.fetch_profile_ranks.unwrap();
                        if info.player_info.riot_user_name.as_str() == self.riot_user_name.as_str()
                            && info.player_info.riot_tag_line.as_str()
                                == self.riot_tag_line.as_str()
                        {
                            self.player_data.icon_id = info.player_info.icon_id;
                            let data: Vec<RankScore> = rank
                                .rank_scores
                                .into_vec()
                                .into_iter()
                                .filter_map(|val| {
                                    if val.queue_type.is_empty() {
                                        None
                                    } else {
                                        Some(val)
                                    }
                                })
                                .collect();
                            self.player_data.rank_scores = Some(data.into());
                        }
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::PlayerIcon(data) => {
                    todo!("{:?}", data)
                }
                Results::ChampImage(image_errors) => {
                    todo!("{:?}", image_errors)
                }
                Results::MatchDetails(result) => match result {
                    Ok((match_details, id)) => {
                        self.player_data
                            .match_data_map
                            .insert(id, Some(match_details.data.data_match));
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },

                Results::ChampJson(err) => {
                    todo!("{:?}", err)
                }

                Results::PlayerSuggestions(result) => match result {
                    Ok(suggestions) => self.player_suggestions = suggestions,
                    Err(e) => todo!("{:?}", e),
                },

                payload => unreachable!(
                    "App has reached an impossible state, this should already be covered {:?}",
                    payload
                ),
            }
        };
    }
}
