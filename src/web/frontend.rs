use rocket::{
  http::{ContentType, Status},
  request::Request,
  response::{status::Custom, Responder, Response, Result as RocketResult},
};
use std::{ffi::OsStr, io::Cursor, path::PathBuf};

#[derive(RustEmbed)]
#[folder = "dist/"]
struct Asset;

pub struct StaticResponse(Response<'static>);

impl<'r, 'o: 'r> Responder<'r, 'o> for StaticResponse {
  fn respond_to(self, _request: &'r Request<'_>) -> RocketResult<'o> {
    Ok(self.0)
  }
}

#[get("/robots.txt")]
pub fn robots() -> Result<StaticResponse, Custom<()>> {
  Asset::get("static/robots.txt").map_or_else(
    || Err(Custom(Status::NotFound, ())),
    |asset| Ok(StaticResponse(Response::build().header(ContentType::Plain).sized_body(asset.len(), Cursor::new(asset)).finalize())),
  )
}

#[get("/")]
pub fn index() -> Result<StaticResponse, Custom<()>> {
  Asset::get("index.html").map_or_else(
    || Err(Custom(Status::NotFound, ())),
    |asset| Ok(StaticResponse(Response::build().header(ContentType::HTML).sized_body(asset.len(), Cursor::new(asset)).finalize())),
  )
}

#[allow(unused_variables)]
#[get("/<path..>", rank = 20)]
pub fn catchall(path: PathBuf) -> Result<StaticResponse, Custom<()>> {
  Asset::get("index.html").map_or_else(
    || Err(Custom(Status::NotFound, ())),
    |asset| Ok(StaticResponse(Response::build().header(ContentType::HTML).sized_body(asset.len(), Cursor::new(asset)).finalize())),
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
          .sized_body(asset.len(), Cursor::new(asset))
          .finalize(),
      ))
    },
  )
}
