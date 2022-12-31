use std::path::PathBuf;

use axum::{
  body::Full,
  extract::Path,
  http::StatusCode,
  response::{IntoResponse, Response},
};

#[derive(RustEmbed)]
#[folder = "dist/"]
struct Asset;

pub async fn robots() -> Result<impl IntoResponse, impl IntoResponse> {
  Asset::get("robots.txt").map_or_else(
    || Err(StatusCode::NOT_FOUND),
    |asset| Ok(Response::builder().header("content-type", "text/plain").body(Full::from(asset.data)).unwrap()),
  )
}

pub async fn index() -> Result<impl IntoResponse, impl IntoResponse> {
  Asset::get("index.html").map_or_else(
    || Err(StatusCode::NOT_FOUND),
    |asset| Ok(Response::builder().header("content-type", "text/html").body(Full::from(asset.data)).unwrap()),
  )
}

pub async fn assets(Path(path): Path<PathBuf>) -> Result<impl IntoResponse, impl IntoResponse> {
  Asset::get(&path.display().to_string()).map_or_else(
    || Err(StatusCode::NOT_FOUND),
    |asset| {
      let content_type = path.as_path().to_str().and_then(|path| new_mime_guess::from_path(path).first()).ok_or(StatusCode::BAD_REQUEST)?;

      let age = if path.extension().map(|e| e == "jpg" || e == "png").unwrap_or(false) {
        "86400" // 1 day
      } else {
        "31536000" // 1 year
      };

      Ok(
        Response::builder()
          .header("content-type", content_type.to_string())
          .header("cache-control", format!("max-age={}", age))
          .body(Full::from(asset.data))
          .unwrap(),
      )
    },
  )
}
