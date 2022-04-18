mod frontend;

use rocket::{
  Build, Config as RocketConfig, Rocket, Route,
};

pub fn server(provider: RocketConfig) -> Rocket<Build> {
    rocket::custom(provider).mount("/", routes())
}

pub fn routes() -> Vec<Route> {
  routes![self::frontend::robots, self::frontend::index, self::frontend::catchall, self::frontend::assets]
}
