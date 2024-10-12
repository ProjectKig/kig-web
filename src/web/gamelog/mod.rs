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
    db::GameLogMeta,
    error::Result,
    modes::GameMode,
    protos::gamelog::{self, ChatEvent_ChatType, GameLog, TimeEvent},
    AppState,
};
use actix_web::{
    http::header::{ContentType, IntoHeaderValue},
    web, HttpResponse,
};
use askama::Template;
use cached::{proc_macro::cached, TimedCache};
use event::EventType::{self, *};
use gamelog::{BukkitDamageCause, ChatEvent, GameEvent};
use regex::Regex;
use std::{borrow::Cow, str::FromStr};
use std::{collections::HashMap, convert::TryInto, fmt};

mod bed;
mod bp;
mod cai;
mod event;
mod grav;
mod halloween;
mod timv;

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
    events: Vec<WrappedEvent>,
    player_teams: HashMap<&'a str, &'a Team<'a>>,
    winner: Option<Team<'a>>,
    mode: GameMode,
    functions: Functions,
    extension: WrappedExtension,
    server: Option<String>,
}

// Extensions - each mode can implement its own version
pub struct Functions {
    extension: Box<dyn GameLogExtension>,
}

pub trait GameLogExtension {
    fn parse_event(&self, event: &GameEvent) -> event::EventType;
    fn get_box_color(&self, event: &EventType) -> &'static str;
    fn supports_score(&self) -> bool {
        true
    }
    fn get_map<'slf, 'log: 'slf>(&'slf self, log: &'log GameLog) -> Cow<str> {
        Cow::Borrowed(log.get_map())
    }
}

#[derive(Clone)]
pub enum WrappedExtension {
    Cai(cai::CaiExtension),
    Timv(timv::TimvExtension),
    Bp(bp::BpExtension),
    Grav(grav::GravExtension),
    Bed(bed::BedExtension),
    Halloween(halloween::HalloweenExtension),
}

impl WrappedExtension {
    fn boxed(self) -> Box<dyn GameLogExtension> {
        use self::WrappedExtension::*;
        match self {
            Cai(ext) => Box::new(ext),
            Timv(ext) => Box::new(ext),
            Bp(ext) => Box::new(ext),
            Grav(ext) => Box::new(ext),
            Bed(ext) => Box::new(ext),
            Halloween(ext) => Box::new(ext),
        }
    }
}

impl GameMode {
    fn to_gamelog_ext(&self, log: &GameLog) -> WrappedExtension {
        use self::WrappedExtension::*;
        match self {
            GameMode::CAI => Cai(cai::CaiExtension {}),
            GameMode::TIMV => Timv(timv::TimvExtension {}),
            GameMode::BP => Bp(bp::BpExtension {}),
            GameMode::GRAV => Grav(grav::GravExtension::new(log)),
            GameMode::BED => Bed(bed::BedExtension::new(log)),
            GameMode::Halloween2023 | GameMode::Halloween2024 => {
                Halloween(halloween::HalloweenExtension {})
            }
        }
    }
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
    pub nick: Option<&'a str>,
}

#[derive(Clone)]
pub struct Team<'a> {
    pub name: &'a str,
    pub players: Vec<Player<'a>>,
    pub score: i32,
    pub color: &'static str,
}

pub struct WrappedEvent {
    event: EventType,
    time: i32,
}

enum ChatChannel<'a> {
    Static(&'static str),
    Team(&'a str, &'static str),
    None,
}

#[cached(
    type = "TimedCache<(Vec<u8>, GameMode), (GameLog, GameLogMeta)>",
    create = "{ TimedCache::with_lifespan(120) }",
    convert = "{ (id.clone(), mode) }",
    result
)]
async fn get_log(
    state: web::Data<AppState>,
    mode: GameMode,
    id: Vec<u8>,
) -> Result<(GameLog, GameLogMeta)> {
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
            let (log, meta) = get_log(state, mode, id.to_be_bytes()[2..].to_vec()).await?;
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
                            nick: p.has_nick().then(|| p.get_nick()),
                        })
                        .collect(),
                })
                .collect();
            let winner = log
                .has_winner()
                .then(|| log.get_winner())
                .and_then(|winner| {
                    teams
                        .iter()
                        .find(|t| t.name == winner)
                        .cloned()
                        .or_else(|| {
                            Some(Team {
                                name: winner,
                                color: "",
                                score: 0,
                                players: vec![],
                            })
                        })
                });
            let player_teams = teams
                .iter()
                .flat_map(|t| t.players.iter().map(move |p| (p.name, t)))
                .collect();
            let extension = mode.to_gamelog_ext(&log);
            let extension_ptr = extension.clone().boxed();
            let render = GamelogTemplate {
                log: &log,
                total_players: if log.get_start_players() == 0 {
                    log.get_teams().iter().map(|t| t.get_players().len()).sum()
                } else {
                    log.get_start_players() as usize
                },
                game_id: &path_id,
                teams: teams.clone(),
                events: log
                    .get_events()
                    .iter()
                    .map(|e| WrappedEvent::parse(e, &*extension_ptr))
                    .collect(),
                player_teams,
                winner,
                mode,
                functions: Functions {
                    extension: extension_ptr,
                },
                extension,
                server: meta.server,
            }
            .render()
            .unwrap();
            Ok(HttpResponse::Ok()
                .content_type(IntoHeaderValue::try_into(ContentType::html()).unwrap())
                .body(render))
        }
        Err(_) => Ok(HttpResponse::BadRequest().body("Invalid game ID")),
    }
}

impl Functions {
    fn get_box_color(&self, event: &WrappedEvent) -> &str {
        match event.get_raw_event() {
            EventType::Chat(_) => "",
            EventType::Join(_) => "list-group-item-info",
            EventType::Leave(_) => "list-group-item-dark",
            _ => self.extension.get_box_color(&event.event),
        }
    }

    fn get_map<'slf, 'log: 'slf>(&'slf self, log: &'log GameLog) -> Cow<str> {
        self.extension.get_map(log)
    }
}

impl WrappedEvent {
    fn parse(event: &TimeEvent, extension: &dyn GameLogExtension) -> Self {
        let time = event.get_time();
        WrappedEvent {
            time,
            event: Self::parse_event(event, extension),
        }
    }

    /// Attempts to parse the event, interpreting it as a default event if possible.
    fn parse_event(event: &TimeEvent, extension: &dyn GameLogExtension) -> EventType {
        let event = event.get_event();
        match extension.parse_event(event) {
            EventType::Unknown => {
                use crate::protos::gamelog::exts::*;
                if let Some(event) = chat.get(event) {
                    EventType::Chat(event)
                } else if let Some(event) = join.get(event) {
                    EventType::Join(event)
                } else if let Some(event) = leave.get(event) {
                    EventType::Leave(event)
                } else {
                    EventType::Unknown
                }
            }
            event => event,
        }
    }

    fn get_time(&self) -> i32 {
        self.time
    }

    fn get_raw_event(&self) -> &EventType {
        &self.event
    }

    fn get_chat_channel<'a>(
        &self,
        event: &ChatEvent,
        log: &GamelogTemplate<'a>,
    ) -> ChatChannel<'a> {
        if let EventType::Chat(chat_event) = &self.event {
            match chat_event.get_field_type() {
                ChatEvent_ChatType::LOBBY => ChatChannel::Static("Lobby"),
                ChatEvent_ChatType::TEAM => if event.has_team() {
                    log.teams.get(event.get_team() as usize)
                } else {
                    log.player_teams.get(event.get_sender()).copied()
                }
                .map(|t| ChatChannel::Team(t.name, t.color))
                .unwrap_or_else(|| ChatChannel::Team(SPECTATORS.name, SPECTATORS.color)),
                ChatEvent_ChatType::SHOUT => ChatChannel::Static("Shout"),
                ChatEvent_ChatType::BROADCAST => ChatChannel::Static("Broadcast"),
                ChatEvent_ChatType::GLOBAL => ChatChannel::None,
            }
        } else {
            ChatChannel::None
        }
    }

    fn is_chat(&self) -> bool {
        matches!(self.event, EventType::Chat(_))
    }
}

impl BukkitDamageCause {
    fn get_damage_desc(&self) -> &'static str {
        match self {
            BukkitDamageCause::ENTITY_ATTACK => "Melee",
            BukkitDamageCause::PROJECTILE => "Projectile",
            BukkitDamageCause::VOID => "Void",
            BukkitDamageCause::SUFFOCATION => "Suffocation",
            BukkitDamageCause::FIRE => "Fire",
            BukkitDamageCause::FIRE_TICK => "Fire",
            BukkitDamageCause::FALL => "Fall",
            BukkitDamageCause::DROWNING => "Drowning",
            BukkitDamageCause::OTHER => "Unknown cause",
        }
    }
}

impl From<&[u8]> for UUID {
    fn from(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), 16);
        UUID {
            msb: u64::from_be_bytes(TryInto::try_into(&bytes[0..8]).unwrap()),
            lsb: u64::from_be_bytes(TryInto::try_into(&bytes[8..16]).unwrap()),
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
    pub use super::grav::filters::*;
    use std::{borrow::Cow, collections::HashMap};

    use super::Team;

    pub fn format_duration(millis: &i64) -> askama::Result<String> {
        Ok(super::format_duration(*millis as i32))
    }

    pub fn format_duration_i32(millis: &i32) -> askama::Result<String> {
        Ok(super::format_duration(*millis))
    }

    pub fn map_file_name(map_name: &str) -> askama::Result<String> {
        if map_name.is_empty() {
            return Ok(String::from("default"));
        }
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
