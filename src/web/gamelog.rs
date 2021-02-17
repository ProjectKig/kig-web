// Copyright (C) 2021 RoccoDev
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    error::Result,
    modes::GameMode,
    protos::common::BukkitDamageCause,
    protos::gamelog::{
        self, ChatEvent_ChatType,
        GameEvent_oneof_extension::{self, *},
        GameLog, TimeEvent,
    },
    AppState,
};
use actix_web::{web, HttpResponse};
use askama::Template;
use cached::{proc_macro::cached, TimedCache};
use regex::Regex;
use std::str::FromStr;
use std::{collections::HashMap, convert::TryInto, fmt};

lazy_static::lazy_static! {
    static ref MAP_ESCAPE_REGEX: Regex = Regex::new(r#"[^a-zA-Z0-9]"#).unwrap();
    static ref SPECTATORS: Team<'static> = Team {
      name: "Spec",
      players: vec![],
      score: 0,
      color: mc_to_rgb('7')
    };
}
static DEFAULT_COLORS: [char; 2] = ['c', 'e'];

#[derive(Template)]
#[template(path = "gamelog.html")]
struct GamelogTemplate<'a> {
    log: &'a GameLog,
    total_players: usize,
    game_id: &'a str,
    teams: Vec<Team<'a>>,
    events: Vec<WrappedEvent<'a>>,
    player_teams: HashMap<&'a str, &'a Team<'a>>,
    winner: Option<&'a Team<'a>>,
    mode: GameMode,
}

/// Represents a Java UUID
#[derive(Clone, Copy)]
pub struct UUID {
    lsb: u64,
    msb: u64,
}

#[derive(Clone, Copy)]
pub struct Player<'a> {
    pub uuid: UUID,
    pub name: &'a str,
}

#[derive(Clone)]
pub struct Team<'a> {
    pub name: &'a str,
    pub players: Vec<Player<'a>>,
    pub score: i32,
    pub color: &'static str,
}

struct WrappedEvent<'a>(&'a TimeEvent);

enum ChatChannel<'a> {
    Static(&'static str),
    Team(&'a str, &'static str),
    None,
}

#[cached(
    type = "TimedCache<(Vec<u8>, GameMode), GameLog>",
    create = "{ TimedCache::with_lifespan(120) }",
    convert = "{ (id.clone(), mode) }",
    result
)]
async fn get_log(state: web::Data<AppState>, mode: GameMode, id: Vec<u8>) -> Result<GameLog> {
    state
        .db
        .game_log_by_id(mode.get_database_id(), id.clone())
        .await
        .and_then(|opt| opt.ok_or(crate::error::Error::NotFound))
}

pub async fn gamelog_by_id(
    state: web::Data<AppState>,
    web::Path((mut mode, path_id)): web::Path<(String, String)>,
) -> Result<HttpResponse> {
    match base62::decode(&path_id) {
        Ok(id) => {
            mode.make_ascii_uppercase();
            let mode = GameMode::from_str(&mode).map_err(|_| crate::error::Error::ModeNotFound)?;
            let log = get_log(state, mode, id.to_be_bytes()[2..].to_vec()).await?;
            let teams: Vec<Team> = log
                .get_teams()
                .iter()
                .enumerate()
                .map(|(i, t)| Team {
                    name: t.get_name(),
                    score: t.get_score(),
                    color: get_team_color(t, i),
                    players: t
                        .get_players()
                        .iter()
                        .map(|p| Player {
                            name: p.get_name(),
                            uuid: p.get_uuid().into(),
                        })
                        .collect(),
                })
                .collect();
            let winner = teams.iter().find(|t| t.name == log.get_winner());
            let player_teams = teams
                .iter()
                .flat_map(|t| t.players.iter().map(move |p| (p.name, t)))
                .collect();
            let render = GamelogTemplate {
                log: &log,
                total_players: if log.get_start_players() == 0 {
                    log.get_teams().iter().map(|t| t.get_players().len()).sum()
                } else {
                    log.get_start_players() as usize
                },
                game_id: &path_id,
                teams: teams.clone(),
                events: log.get_events().iter().map(|e| WrappedEvent(e)).collect(),
                player_teams,
                winner,
                mode,
            }
            .render()
            .unwrap();
            Ok(HttpResponse::Ok().body(render))
        }
        Err(_) => Ok(HttpResponse::BadRequest().body("Invalid game ID")),
    }
}

impl<'a> WrappedEvent<'a> {
    fn get_time(&self) -> i32 {
        self.0.get_time()
    }

    fn get_raw_event(&self) -> &GameEvent_oneof_extension {
        self.0.get_event().extension.as_ref().unwrap()
    }

    fn get_box_color(&self) -> &'static str {
        match self.get_raw_event() {
            Chat(_) => "",
            Join(_) => "list-group-item-info",
            Leave(_) => "list-group-item-dark",
            Catch(_) => "list-group-item-primary",
            Escape(_) => "list-group-item-primary",
            Capture(_) => "list-group-item-primary",
            Death(_) => "list-group-item-secondary",
        }
    }

    fn get_kill_description(&self) -> &'static str {
        match self.0.get_event().get_Death().get_cause() {
            BukkitDamageCause::ENTITY_ATTACK => "Melee",
            BukkitDamageCause::PROJECTILE => "Projectile",
            BukkitDamageCause::VOID => "Void",
            BukkitDamageCause::SUFFOCATION => "Drowning",
            BukkitDamageCause::FIRE => "Fire",
            BukkitDamageCause::FIRE_TICK => "Fire",
            BukkitDamageCause::OTHER => "Unknown cause",
        }
    }

    fn get_chat_channel(
        &self,
        player: &str,
        player_teams: &'a HashMap<&str, &Team<'a>>,
    ) -> ChatChannel<'a> {
        match self.0.get_event().get_Chat().get_field_type() {
            ChatEvent_ChatType::LOBBY => ChatChannel::Static("Lobby"),
            ChatEvent_ChatType::TEAM => player_teams
                .get(player)
                .map(|&x| ChatChannel::Team(x.name, x.color))
                .unwrap_or_else(|| ChatChannel::Team(SPECTATORS.name, SPECTATORS.color)),
            ChatEvent_ChatType::SHOUT => ChatChannel::Static("Shout"),
            ChatEvent_ChatType::BROADCAST => ChatChannel::Static("Broadcast"),
            ChatEvent_ChatType::GLOBAL => ChatChannel::None,
        }
    }

    fn is_chat(&self) -> bool {
        self.0.get_event().has_Chat()
    }
}

impl From<&[u8]> for UUID {
    fn from(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), 16);
        UUID {
            msb: u64::from_be_bytes(bytes[0..8].try_into().unwrap()),
            lsb: u64::from_be_bytes(bytes[8..16].try_into().unwrap()),
        }
    }
}

impl fmt::Display for UUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{}-{}-{}",
            &digits(self.msb >> 32, 8)[1..],
            &digits(self.msb >> 16, 4)[1..],
            &digits(self.msb, 4)[1..],
            &digits(self.lsb >> 48, 4)[1..],
            &digits(self.lsb, 12)[1..]
        )
    }
}

fn get_team_color(team: &gamelog::Team, idx: usize) -> &'static str {
    mc_to_rgb(if team.has_color() {
        team.get_color() as u8 as char
    } else {
        DEFAULT_COLORS[idx]
    })
}

/// Converts a Minecraft color to an RGB for CSS
#[inline]
fn mc_to_rgb(mc: char) -> &'static str {
    match mc {
        '1' => "#0000AA",
        '2' => "#00AA00",
        '3' => "#00AAAA",
        '4' => "#AA0000",
        '5' => "#AA00AA",
        '6' => "#FFAA00",
        '7' => "#AAAAAA",
        '8' => "#555555",
        '9' => "#5555FF",
        'a' | 'A' => "#55FF55",
        'b' | 'B' => "#55FFFF",
        'c' | 'C' => "#e00b0b",
        'd' | 'D' => "#FF55FF",
        'e' | 'E' => "#cc901e",
        'f' | 'F' => "#FFFFFF",
        _ => "#000000",
    }
}

fn digits(val: u64, n: usize) -> String {
    let high = 1u64 << (n * 4usize);
    format!("{:x}", (high | val & (high - 1u64)))
}

fn format_duration(millis: i32) -> String {
    let minutes = millis / (1000 * 60);
    let seconds = millis / 1000 % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

mod filters {
    use std::{borrow::Cow, collections::HashMap};

    use super::Team;

    pub fn format_duration(millis: &i64) -> askama::Result<String> {
        Ok(super::format_duration(*millis as i32))
    }

    pub fn format_duration_i32(millis: &i32) -> askama::Result<String> {
        Ok(super::format_duration(*millis))
    }

    pub fn map_file_name(map_name: &str) -> askama::Result<String> {
        let mut res: String = super::MAP_ESCAPE_REGEX.replace_all(map_name, "").into();
        res.make_ascii_lowercase();
        Ok(res)
    }

    pub fn team_from_idx<'a>(idx: &'a i32, teams: &'a [Team<'a>]) -> askama::Result<&'a Team<'a>> {
        Ok(teams.get(*idx as usize).unwrap_or(&super::SPECTATORS))
    }

    pub fn team_color<'a>(
        player: &'a str,
        player_teams: &'a HashMap<&str, &Team<'a>>,
    ) -> askama::Result<Cow<'a, str>> {
        Ok(match player_teams.get(player).map(|t| t.color) {
            Some(color) => Cow::Owned(format!("{} !important", color)),
            None => Cow::Borrowed("#000000"),
        })
    }
}
