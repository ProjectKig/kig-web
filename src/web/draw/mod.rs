// Copyright (C) 2026 KIG Network
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
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use actix_web::{
    HttpResponse, Result,
    error::{ErrorBadRequest, ErrorUnauthorized},
    http::header::ContentType,
    web,
};
use askama::Template;
use cached::{TimedCache, proc_macro::cached};
use image::{ImageFormat, ImageReader, imageops::FilterType};
use std::{io::Cursor, time::Duration};

use crate::{AppState, web::get_current_time};

#[derive(serde::Deserialize)]
pub struct Auth {
    key: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DrawImage {
    pub drawer_id: String,
    pub drawer_name: String,
    pub word: String,
    pub report_key: Option<ReportKey>,
    pub image: Vec<u8>,
}

#[derive(Template)]
#[template(path = "draw.html")]
struct DrawTemplate<'a> {
    image: &'a DrawImage,
    image_url: String,
    current_year: String,
}

type ImageId = Vec<u8>;
type ReportKey = Vec<u8>;

pub async fn page_by_id(
    state: web::Data<AppState>,
    id_src: web::Path<String>,
    auth: web::Query<Auth>,
) -> Result<HttpResponse> {
    let id = decode_id(&id_src, 43)?;
    let image = get_image(state, id).await?;
    validate_auth(&image, &auth)?;

    let current_time = get_current_time();
    let image_url = match auth.key.as_ref() {
        Some(key) => format!("/sketch/img/{id_src}?key={key}"),
        None => format!("/sketch/img/{id_src}"),
    };
    let render = DrawTemplate {
        image: &image,
        image_url,
        current_year: current_time.year().to_string(),
    }
    .render()
    .unwrap();
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(render))
}

pub async fn image_by_id(
    state: web::Data<AppState>,
    id: web::Path<String>,
    auth: web::Query<Auth>,
) -> Result<HttpResponse> {
    let id = decode_id(&id, 43)?;
    let image = get_image(state, id).await?;
    validate_auth(&image, &auth)?;

    Ok(HttpResponse::Ok()
        .content_type("image/png")
        .body(image.image))
}

fn validate_auth(image: &DrawImage, auth: &Auth) -> Result<()> {
    // ignore key parsing errors, treat as no key
    let key = auth.key.as_ref().and_then(|k| decode_id(k, 22).ok());
    match (image.report_key.as_ref(), key) {
        (Some(_), None) => {
            return Err(ErrorUnauthorized(
                "this image was reported, so it can't be accessed",
            ));
        }
        (Some(exp), Some(act)) if exp != &act => {
            return Err(ErrorUnauthorized(
                "this image was reported, so it can't be accessed",
            ));
        }
        _ => Ok(()),
    }
}

fn decode_id(id: &str, expected_length: usize) -> Result<Vec<u8>> {
    if id.len() != expected_length {
        return Err(ErrorBadRequest("invalid id"));
    }
    rust_base62::decode(id.as_bytes()).map_err(|_| ErrorBadRequest("invalid id"))
}

#[cached(
    ty = "TimedCache<Vec<u8>, DrawImage>",
    create = "{ TimedCache::with_lifespan(Duration::from_secs(120)) }",
    convert = "{ id.clone() }",
    result
)]
async fn get_image(state: web::Data<AppState>, id: ImageId) -> crate::error::Result<DrawImage> {
    let mut image = state
        .db
        .draw_image_by_id(id)
        .await
        .and_then(|opt| opt.ok_or(crate::error::Error::NotFound))?;
    let resized = actix_web::rt::task::spawn_blocking(move || {
        let src = std::mem::take(&mut image.image);
        resize_image(src).map(|resized| {
            image.image = resized;
            image
        })
    })
    .await
    .unwrap()
    .map_err(|_| crate::error::Error::BadImage)?;
    Ok(resized)
}

fn resize_image(src: Vec<u8>) -> image::ImageResult<Vec<u8>> {
    let img = ImageReader::new(Cursor::new(src))
        .with_guessed_format()?
        .decode()?;
    let resized = img.resize(65 * 8, 43 * 8, FilterType::Nearest);
    let mut resized_enc = Vec::new();
    resized.write_to(&mut Cursor::new(&mut resized_enc), ImageFormat::Png)?;
    Ok(resized_enc)
}
