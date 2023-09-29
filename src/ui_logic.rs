use std::{collections::HashMap, sync::Arc};

use crate::ui::{self, Champ, MatchFuture, Payload, Results};

fn get_role_index(role: u8) -> Option<u8> {
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

    pub fn update_matches(&self, name: Arc<String>) {
        self.send_message(Payload::MatchSummaries {
            name: name.clone(),
            roles: get_role_index(self.role),
            region_id: self.data_dragon.region,
            page: self.page,
        });
        self.send_message(Payload::PlayerRanks {
            name: name.clone(),
            region_id: self.data_dragon.region,
        });
        self.send_message(Payload::PlayerRanking {
            name: name.clone(),
            region_id: self.data_dragon.region,
        });
        self.send_message(Payload::PlayerInfo {
            name: name.clone(),
            version_index: 0,
            region_id: self.data_dragon.region,
        });
    }

    pub fn update_data(&mut self, versions: Arc<[String]>, champs: Arc<HashMap<i64, Champ>>) {
        if let Ok(receiver) = self.receiver.try_recv() {
            match receiver {
                Results::MatchSum(match_sums) => match match_sums {
                    Ok(matches) => {
                        let data = matches.data.fetch_player_match_summaries;
                        self.finished_match_summeries = data.finished_match_summaries;
                        let summaries = data.match_summaries;
                        summaries.iter().for_each(|summary| {
                            if self.match_summeries.get(&summary.match_id).is_none() {
                                self.match_summeries.insert(summary.match_id, MatchFuture { _match: None });
                                self.send_message(Payload::GetMatchDetails { name: self.message_name.clone(), version: summary.version.clone(), id: summary.match_id, region_id: self.data_dragon.region });
                            }

                            let champ = &champs[&summary.champion_id];
                            if !champ.image_started.load(std::sync::atomic::Ordering::Relaxed) {
                                let key = &champ.key;
                                self.send_message(
                                    Payload::GetChampImage {
                                        url: format!(
                                            "http://ddragon.leagueoflegends.com/cdn/{}/img/champion/{}.png",
                                            versions[0], key
                                        ),
                                        id: summary.champion_id,
                                    },
                                );
                                champ.image_started.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                        });
                        self.summeries = Some(summaries)
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::PlayerUpdate(update) => match update {
                    Ok(updated) => {
                        let data = updated.data.update_player_profile;
                        if data.success {
                            self.update_matches(self.message_name.clone());
                        } else {
                            dbg!("{:?}", data.error_reason);
                        }
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::ProfileRanks(rank) => match rank {
                    Ok(rank) => {
                        let data = rank
                            .data
                            .fetch_profile_ranks
                            .rank_scores
                            .into_iter()
                            .filter_map(|rank| {
                                if rank.queue_type.is_empty() {
                                    None
                                } else {
                                    Some(rank)
                                }
                            })
                            .collect();
                        self.rank = Some(data);
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                Results::Ranking(ranking) => match ranking {
                    Ok(ranking) => {
                        self.ranking = ranking.data.overall_ranking;
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },
                // Todo: Display this info
                Results::PlayerInfo(info) => match info {
                    Ok(info) => {
                        let info = info.data.profile_player_info.unwrap();
                        if info.summoner_name.as_ref() == self.message_name.as_str() {
                            self.icon_id = info.icon_id;
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
                Results::MatchDetails(deets) => match deets {
                    Ok((match_details, id)) => {
                        self.match_summeries.insert(
                            id,
                            MatchFuture {
                                _match: Some(match_details.data.data_match),
                            },
                        );
                    }
                    Err(err) => {
                        dbg!("{:?}", err);
                    }
                },

                Results::ChampJson(err) => {
                    todo!("{:?}", err)
                }

                payload => unreachable!(
                    "App has reached an impossible state, this should already be covered {:?}",
                    payload
                ),
            }
        };
    }
}
