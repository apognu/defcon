use defcon::{api::types as api, model::specs::*};
use prettytable::{cell, format::consts::*, row, Table};

pub trait DisplaySpec {
  fn display(&self);
}

impl DisplaySpec for api::Spec {
  fn display(&self) {
    match self {
      api::Spec::Http(spec) => spec.display(),
      _ => println!("NOPE"),
    }
  }
}

impl DisplaySpec for Http {
  fn display(&self) {
    let mut table = Table::new();
    table.set_format(*FORMAT_NO_BORDER_LINE_SEPARATOR);

    table.add_row(row![b -> "URL", self.url]);
    if let Some(code) = self.code {
      table.add_row(row![b -> "Expected code", &format!("{}", code)]);
    }
    if let Some(content) = &self.content {
      table.add_row(row![b -> "Expected content", content]);
    }
    if let Some(digest) = &self.digest {
      table.add_row(row![b -> "Expected checksum", digest]);
    }

    table.printstd();
  }
}
