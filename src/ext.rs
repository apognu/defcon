use anyhow::Result;
use std::time::Duration;

use extend::ext;
use humantime::parse_duration;

pub trait Run<T> {
  type Target;

  fn run<F>(self, f: F) -> Self::Target
  where
    F: FnOnce(T) -> Self::Target;
}

impl<T> Run<T> for Option<T> {
  type Target = ();

  fn run<F>(self, f: F)
  where
    F: FnOnce(T),
  {
    if let Some(value) = self {
      f(value);
    }
  }
}

#[ext(pub, name = EnvExt)]
impl<E> Result<String, E>
where
  E: std::error::Error,
{
  fn or_string(self, default: &str) -> String {
    self.unwrap_or_else(|_| default.to_owned())
  }

  fn or_duration_min(self, default: &str, min: Duration) -> Result<Duration> {
    let value = self.unwrap_or_else(|_| default.to_owned());

    match parse_duration(&value) {
      Ok(duration) if duration > min => Ok(duration),
      Ok(_) => Ok(min),
      Err(err) => Err(err.into()),
    }
  }
}

pub const fn to_true() -> bool {
  true
}

pub const fn to_false() -> bool {
  false
}
