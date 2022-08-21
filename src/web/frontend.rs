use rocket::{
  http::{ContentType, Status},
  response::{status::Custom, Response},
};
use std::{ffi::OsStr, io::Cursor, path::PathBuf};

use crate::api::StaticResponse;

#[derive(RustEmbed)]
#[folder = "dist/"]
struct Asset;

#[get("/robots.txt")]
pub fn robots() -> Result<StaticResponse, Custom<()>> {
  Asset::get("static/robots.txt").map_or_else(
    || Err(Custom(Status::NotFound, ())),
    |asset| {
      Ok(StaticResponse(
        Response::build().header(ContentType::Plain).sized_body(asset.data.len(), Cursor::new(asset.data)).finalize(),
      ))
    },
  )
}

#[get("/")]
pub fn index() -> Result<StaticResponse, Custom<()>> {
  Asset::get("index.html").map_or_else(
    || Err(Custom(Status::NotFound, ())),
    |asset| {
      Ok(StaticResponse(
        Response::build().header(ContentType::HTML).sized_body(asset.data.len(), Cursor::new(asset.data)).finalize(),
      ))
    },
  )
}

#[allow(unused_variables)]
#[get("/<path..>", rank = 20)]
pub fn catchall(path: PathBuf) -> Result<StaticResponse, Custom<()>> {
  Asset::get("index.html").map_or_else(
    || Err(Custom(Status::NotFound, ())),
    |asset| {
      Ok(StaticResponse(
        Response::build().header(ContentType::HTML).sized_body(asset.data.len(), Cursor::new(asset.data)).finalize(),
      ))
    },
  )
}

#[get("/assets/<path..>")]
pub fn assets(path: PathBuf) -> Result<StaticResponse, Custom<()>> {
  Asset::get(&path.display().to_string()).map_or_else(
    || Err(Custom(Status::NotFound, ())),
    |asset| {
      let content_type = path
        .as_path()
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ContentType::from_extension)
        .ok_or(Custom(Status::BadRequest, ()))?;

      let age = if path.extension().map(|e| e == "jpg" || e == "png").unwrap_or(false) {
        "86400" // 1 day
      } else {
        "31536000" // 1 year
      };

      Ok(StaticResponse(
        Response::build()
          .header(content_type)
          .raw_header("cache-control", format!("max-age={}", age))
          .sized_body(asset.data.len(), Cursor::new(asset.data))
          .finalize(),
      ))
    },
  )
}
