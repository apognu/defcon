mod alerters;
mod checks;
pub mod error;
mod events;
pub mod middlewares;
mod runner;
mod site_outages;
pub mod types;

use rocket::{response::status::Custom, Config, Rocket, Route};
use sqlx::{MySql, Pool};

use self::error::ApiError;

type ApiResponse<T> = Result<T, Custom<Option<ApiError>>>;

pub fn server(provider: Config, pool: Pool<MySql>) -> Rocket {
  rocket::custom(provider).manage(pool).mount("/", routes()).register(catchers![not_found, unprocessable])
}

pub fn routes() -> Vec<Route> {
  routes![
    health,
    runner::list_stale,
    runner::report,
    checks::list,
    checks::get,
    checks::create,
    checks::update,
    checks::patch,
    checks::delete,
    site_outages::list,
    site_outages::list_between,
    site_outages::get,
    // outages::comment,
    events::list,
    alerters::list,
    alerters::get,
    alerters::add,
    alerters::update,
  ]
}

#[get("/api/-/health")]
fn health() {}

#[catch(404)]
pub fn not_found() -> ApiError {
  ApiError::new(404, "resource not found")
}

#[catch(422)]
pub fn unprocessable() -> ApiError {
  ApiError::new(422, "the request format could not be understood")
}

#[cfg(test)]
mod tests {
  use rocket::http::Status;

  use crate::spec;

  #[tokio::test]
  async fn health() {
    let (pool, client) = spec::api_client().await.unwrap();

    let response = client.get("/api/-/health").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    pool.cleanup().await;
  }
}
