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

use crate::error::Result;
use crate::protos::gamelog::GameLog;
use crate::web::draw::DrawImage;
use mongodb::bson::{Document, bson, doc};
use mongodb::{
    Client, Database,
    bson::{Binary, Bson},
};
use protobuf::Message;

pub struct DbHandle {
    client: Database,
}

#[derive(Clone)]
pub struct GameLogMeta {
    pub server: Option<String>,
}

impl DbHandle {
    pub async fn new() -> Result<DbHandle> {
        let uri = std::env::var("KIG_MONGO_URI")
            .unwrap_or_else(|_| String::from("mongodb://localhost:27017"));
        let db = std::env::var("KIG_MONGO_DB").unwrap_or_else(|_| String::from("kig"));
        Ok(DbHandle {
            client: Client::with_uri_str(&uri).await?.database(&db),
        })
    }
}

/// Retrieving the data from the db
impl DbHandle {
    pub async fn game_log_by_id(
        &self,
        game: &str,
        id: Vec<u8>,
    ) -> Result<Option<(GameLog, GameLogMeta)>> {
        let filter = doc! {"game_id": Self::bytes(id)};
        let res: Option<Result<(GameLog, GameLogMeta)>> = self
            .client
            .collection::<Document>(&format!("gamelogs_{}", game))
            .find_one(filter)
            .await?
            .map(|doc| {
                Ok((
                    GameLog::parse_from_bytes(&doc.get_binary_generic("data")?).unwrap(),
                    GameLogMeta {
                        server: doc.get_str("server").map(Into::into).ok(),
                    },
                ))
            });
        res.transpose()
    }

    pub async fn draw_image_by_id(&self, id: Vec<u8>) -> Result<Option<DrawImage>> {
        let filter = doc! {"_id": Self::bytes(id)};
        let res: Option<Result<DrawImage>> = self
            .client
            .collection::<Document>("draw_images")
            .find_one(filter)
            .await?
            .map(|mut doc| {
                Ok(DrawImage {
                    drawer: doc.get_str("drawer")?.to_string(),
                    word: doc.get_str("word")?.to_string(),
                    report_key: doc
                        .get_binary_generic_mut("report_key")
                        .map(std::mem::take)
                        .ok(),
                    image: std::mem::take(doc.get_binary_generic_mut("image")?),
                })
            });
        res.transpose()
    }

    #[inline]
    fn bytes(input: Vec<u8>) -> Bson {
        bson!(Binary {
            bytes: input,
            subtype: mongodb::bson::spec::BinarySubtype::Generic
        })
    }
}
