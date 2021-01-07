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

use crate::{error::Result, protos::gamelog::GameLog, AppState};
use actix_web::{web, HttpResponse};
use cached::{proc_macro::cached, TimedCache};
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
    web::Path(id): web::Path<String>,
) -> Result<HttpResponse> {
    match base62::decode(&id) {
        Ok(id) => {
            let log = get_log(state, id.to_be_bytes()[2..].to_vec()).await?;
            Ok(HttpResponse::Ok().body(format!("{:?}", log)))
        }
        Err(_) => Ok(HttpResponse::BadRequest().body("Invalid game ID")),
    }
}
