use std::path::PathBuf;

use axum::{
  body::Full,
  extract::Path,
  http::{
    header::{CACHE_CONTROL, CONTENT_TYPE},
    StatusCode,
  },
  response::{IntoResponse, Response},
};

#[derive(RustEmbed)]
#[folder = "dist/"]
struct Asset;

pub async fn robots() -> Result<impl IntoResponse, StatusCode> {
  Asset::get("robots.txt").map_or_else(
    || Err(StatusCode::NOT_FOUND),
    |asset| Ok(Response::builder().header(CONTENT_TYPE, "text/plain").body(Full::from(asset.data)).unwrap()),
  )
}

pub async fn index() -> Result<impl IntoResponse, StatusCode> {
  Asset::get("index.html").map_or_else(
    || Err(StatusCode::NOT_FOUND),
    |asset| Ok(Response::builder().header(CONTENT_TYPE, "text/html").body(Full::from(asset.data)).unwrap()),
  )
}

pub async fn assets(Path(path): Path<PathBuf>) -> Result<impl IntoResponse, StatusCode> {
  Asset::get(&path.display().to_string()).map_or_else(
    || Err(StatusCode::NOT_FOUND),
    |asset| {
      let content_type = path.as_path().to_str().and_then(|path| new_mime_guess::from_path(path).first()).ok_or(StatusCode::BAD_REQUEST)?;

      #[cfg(debug_assertions)]
      let cache = "no-cache";

      #[cfg(not(debug_assertions))]
      let cache = if path.extension().map(|e| e == "jpg" || e == "png").unwrap_or(false) {
        "max-age=86400" // 1 day
      } else {
        "max-age=31536000" // 1 year
      };

      Ok(
        Response::builder()
          .header(CONTENT_TYPE, content_type.to_string())
          .header(CACHE_CONTROL, cache)
          .body(Full::from(asset.data))
          .unwrap(),
      )
    },
  )
}
