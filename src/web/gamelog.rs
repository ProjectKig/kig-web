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
    protos::gamelog::{
        DeathEvent_DeathCause,
        GameEvent_oneof_extension::{self, *},
        GameLog, TimeEvent, TimeEvent_ModeState,
    },
    AppState,
};
use actix_web::{web, HttpResponse};
use askama::Template;
use cached::{proc_macro::cached, TimedCache};
use regex::Regex;
use std::{convert::TryInto, fmt};

lazy_static::lazy_static! {
    static ref MAP_ESCAPE_REGEX: Regex = Regex::new(r#"[^a-zA-Z0-9]"#).unwrap();
}

#[derive(Template)]
#[template(path = "gamelog.html")]
struct GamelogTemplate<'a> {
    log: &'a GameLog,
    total_players: usize,
    game_id: &'a str,
    teams: Vec<Team<'a>>,
    events: Vec<WrappedEvent<'a>>,
}

/// Represents a Java UUID
struct UUID {
    lsb: u64,
    msb: u64,
}

struct Player<'a> {
    pub uuid: UUID,
    pub name: &'a str,
}

struct Team<'a> {
    pub name: &'a str,
    pub players: Vec<Player<'a>>,
    pub score: i32,
}

struct WrappedEvent<'a>(&'a TimeEvent);

#[cached(
    type = "TimedCache<Vec<u8>, GameLog>",
    create = "{ TimedCache::with_lifespan(120) }",
    convert = "{ id.clone() }",
    result
)]
async fn get_log(state: web::Data<AppState>, id: Vec<u8>) -> Result<GameLog> {
    state
        .db
        .game_log_by_id("cai", id.clone())
        .await
        .and_then(|opt| opt.ok_or(crate::error::Error::NotFound))
}

pub async fn gamelog_id(
    state: web::Data<AppState>,
    web::Path(path_id): web::Path<String>,
) -> Result<HttpResponse> {
    match base62::decode(&path_id) {
        Ok(id) => {
            let log = get_log(state, id.to_be_bytes()[2..].to_vec()).await?;
            let teams = log
                .get_teams()
                .iter()
                .map(|t| Team {
                    name: t.get_name(),
                    score: t.get_score(),
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
            let render = GamelogTemplate {
                log: &log,
                total_players: if log.get_start_players() == 0 {
                    log.get_teams().iter().map(|t| t.get_players().len()).sum()
                } else {
                    log.get_start_players() as usize
                },
                game_id: &path_id,
                teams,
                events: log.get_events().iter().map(|e| WrappedEvent(e)).collect(),
            }
            .render()
            .unwrap();
            Ok(HttpResponse::Ok().body(crate::web::serve_html(
                &format!("Game {}", path_id),
                &render,
            )))
        }
        Err(_) => Ok(HttpResponse::BadRequest().body("Invalid game ID")),
    }
}

impl<'a> WrappedEvent<'a> {
    fn get_time(&self) -> i32 {
        self.0.get_time()
    }

    fn get_state(&self) -> TimeEvent_ModeState {
        self.0.get_state()
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
            DeathEvent_DeathCause::ENTITY_ATTACK => "Melee",
            DeathEvent_DeathCause::PROJECTILE => "Projectile",
            DeathEvent_DeathCause::VOID => "Void",
            DeathEvent_DeathCause::SUFFOCATION => "Drowning",
            DeathEvent_DeathCause::FIRE => "Fire",
            DeathEvent_DeathCause::FIRE_TICK => "Fire",
            DeathEvent_DeathCause::OTHER => "Unknown cause",
        }
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

fn digits(val: u64, n: usize) -> String {
    let high = 1u64 << n * 4usize;
    format!("{:x}", (high | val & high - 1u64))
}

fn format_duration(millis: i32) -> String {
    let minutes = millis / (1000 * 60);
    let seconds = millis / 1000 % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

mod filters {
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
}
