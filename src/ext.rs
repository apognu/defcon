use extend::ext;

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
  fn or_string(self, value: &str) -> String {
    self.unwrap_or_else(|_| value.to_owned())
  }
}

pub const fn to_true() -> bool {
  true
}

pub const fn to_false() -> bool {
  false
}
