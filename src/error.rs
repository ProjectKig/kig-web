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

use mongodb::bson::document::ValueAccessError;
use protobuf::ProtobufError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Database(mongodb::error::Error),
    MongoDeserialize(ValueAccessError),
    Protobuf(ProtobufError),
}

impl From<mongodb::error::Error> for Error {
    fn from(e: mongodb::error::Error) -> Self {
        Error::Database(e)
    }
}

impl From<ProtobufError> for Error {
    fn from(e: ProtobufError) -> Self {
        Error::Protobuf(e)
    }
}

impl From<ValueAccessError> for Error {
    fn from(e: ValueAccessError) -> Self {
        Error::MongoDeserialize(e)
    }
}
