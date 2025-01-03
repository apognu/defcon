mod frontend;

use axum::{routing::get, Router};

pub fn router(router: Router) -> Router {
  router
    .route("/robots.txt", get(self::frontend::robots))
    .route("/index.html", get(self::frontend::index))
    .route("/assets/{*path}", get(self::frontend::assets))
    .fallback(self::frontend::index)
}
