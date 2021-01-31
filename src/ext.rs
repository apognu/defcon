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

pub const fn to_true() -> bool {
  true
}

pub const fn to_false() -> bool {
  false
}
