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

use actix_files::Files;
use actix_web::{web, Scope};
use askama::Template;

mod gamelog;

#[derive(Template)]
#[template(path = "master-template.html", escape = "none")]
struct MasterTemplate<'a> {
    title: &'a str,
    content: &'a str,
}

pub fn add_routes() -> Scope {
    web::scope("/").route("/game/{id}", web::get().to(gamelog::gamelog_id))
}

pub fn static_files() -> Files {
    Files::new("/static", "static").show_files_listing()
}

pub fn serve_html(title: &str, content: &str) -> String {
    MasterTemplate { title, content }.render().unwrap()
}
