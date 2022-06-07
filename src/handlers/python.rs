use std::{fs, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use pyo3::prelude::*;
use sqlx::MySqlConnection;

use crate::{
  config::Config,
  handlers::Handler,
  model::{specs::Python, status::*, Check, Event},
  stash::Stash,
};

pub struct PythonHandler<'h> {
  pub check: &'h Check,
  pub path: String,
}

#[async_trait]
impl<'h> Handler for PythonHandler<'h> {
  type Spec = Python;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str, stash: Stash) -> Result<Event> {
    let spec = Python::for_check(conn, self.check).await.context("no spec found")?;

    self.run(&spec, site, stash).await
  }

  async fn run(&self, spec: &Python, site: &str, _stash: Stash) -> Result<Event> {
    let file = format!("{}/{}.py", self.path, spec.script);
    let code = fs::read_to_string(&file)?;

    let (status, message): (u8, String) = pyo3::Python::with_gil(|py| {
      let module = PyModule::from_code(py, &code, &file, &file)?;
      module.setattr("OK", OK)?;
      module.setattr("WARNING", WARNING)?;
      module.setattr("CRITICAL", CRITICAL)?;

      let (status, message) = module.getattr("check")?.call0()?.extract()?;

      PyResult::Ok((status, message))
    })?;

    let event = Event {
      check_id: self.check.id,
      site: site.to_string(),
      status,
      message,
      ..Default::default()
    };

    Ok(event)
  }
}
