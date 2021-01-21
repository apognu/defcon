use sqlx::{
  database::{Database, HasArguments},
  encode::Encode,
  query::QueryAs,
  types::Type,
};

pub trait BindIf<'q, DB, R>
where
  DB: Database,
{
  fn bind_if<F, T: 'q>(self, predicate: F, value: T) -> QueryAs<'q, DB, R, <DB as HasArguments<'q>>::Arguments>
  where
    F: Fn() -> bool,
    T: 'q + Encode<'q, DB> + Type<DB> + Send;

  fn bind_if_or<F, T: 'q, U: 'q>(self, predicate: F, value: T, fallback: U) -> QueryAs<'q, DB, R, <DB as HasArguments<'q>>::Arguments>
  where
    F: Fn() -> bool,
    T: 'q + Encode<'q, DB> + Type<DB> + Send,
    U: 'q + Encode<'q, DB> + Type<DB> + Send,
    Option<T>: 'q + Encode<'q, DB> + Type<DB> + Send;

  fn bind_if_or_null<F, T: 'q>(self, predicate: F, value: T) -> QueryAs<'q, DB, R, <DB as HasArguments<'q>>::Arguments>
  where
    F: Fn() -> bool,
    T: 'q + Encode<'q, DB> + Type<DB> + Send,
    Option<T>: 'q + Encode<'q, DB> + Type<DB> + Send;
}

impl<'q, DB, R> BindIf<'q, DB, R> for QueryAs<'q, DB, R, <DB as HasArguments<'q>>::Arguments>
where
  DB: Database,
{
  fn bind_if<F, T: 'q>(self, predicate: F, value: T) -> QueryAs<'q, DB, R, <DB as HasArguments<'q>>::Arguments>
  where
    F: Fn() -> bool,
    T: 'q + Encode<'q, DB> + Type<DB> + Send,
  {
    if predicate() {
      self.bind(value)
    } else {
      self
    }
  }

  fn bind_if_or<F, T: 'q, U: 'q>(self, predicate: F, value: T, fallback: U) -> QueryAs<'q, DB, R, <DB as HasArguments<'q>>::Arguments>
  where
    F: Fn() -> bool,
    T: 'q + Encode<'q, DB> + Type<DB> + Send,
    U: 'q + Encode<'q, DB> + Type<DB> + Send,
  {
    if predicate() {
      self.bind(value)
    } else {
      self.bind(fallback)
    }
  }

  fn bind_if_or_null<F, T: 'q>(self, predicate: F, value: T) -> QueryAs<'q, DB, R, <DB as HasArguments<'q>>::Arguments>
  where
    F: Fn() -> bool,
    T: 'q + Encode<'q, DB> + Type<DB> + Send,
    Option<T>: 'q + Encode<'q, DB> + Type<DB> + Send,
  {
    if predicate() {
      self.bind(Some(value))
    } else {
      self.bind(None)
    }
  }
}
