use mongodb::Client;

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

use mongodb::error::Result;

pub struct DbHandle {
    client: Client,
}

impl DbHandle {
    pub async fn new() -> Result<DbHandle> {
        let uri = std::env::var("KIG_MONGO_URI")
            .unwrap_or_else(|_| String::from("mongodb://localhost:27017"));
        Ok(DbHandle {
            client: Client::with_uri_str(&uri).await?,
        })
    }
}
