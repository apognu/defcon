mod alerters;
mod checks;
pub mod error;
mod events;
mod outages;
pub mod types;

use rocket::{response::status::Custom, Route};

use self::error::ApiError;

type ApiResponse<T> = Result<T, Custom<Option<ApiError>>>;

pub fn routes() -> Vec<Route> {
  routes![
    health,
    checks::list,
    checks::get,
    checks::add,
    checks::update,
    checks::patch,
    checks::delete,
    outages::list,
    outages::list_between,
    outages::get,
    outages::comment,
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
