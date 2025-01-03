use std::{ffi::CString, fs, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use pyo3::{ffi::c_str, prelude::*};
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
impl Handler for PythonHandler<'_> {
  type Spec = Python;

  async fn check(&self, conn: &mut MySqlConnection, _config: Arc<Config>, site: &str, stash: Stash) -> Result<Event> {
    let spec = Python::for_check(conn, self.check).await.context("no spec found")?;

    self.run(&spec, site, stash).await
  }

  async fn run(&self, spec: &Python, site: &str, _stash: Stash) -> Result<Event> {
    let file = format!("{}/{}.py", self.path, spec.script);
    let code = fs::read_to_string(&file)?;

    let (status, message): (u8, String) = pyo3::Python::with_gil(|py| {
      let module = PyModule::from_code(py, CString::new(code)?.as_c_str(), c_str!("check.py"), c_str!("check"))?;
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

#[cfg(test)]
mod tests {
  use super::{Handler, PythonHandler};
  use crate::{
    config::CONTROLLER_ID,
    model::{specs::Python, status::*, Check},
    stash::Stash,
  };

  const SCRIPT_OK: &str = r#"
def check():
  return (OK, "this is the OK check message")
  "#;

  const SCRIPT_CRITICAL: &str = r#"
def check():
  return (CRITICAL, "this is the CRITICAL check message")
  "#;

  const SCRIPT_SYNTAX_ERROR: &str = r#"
def check():
invalid
  "#;

  const SCRIPT_WRONG_TYPE: &str = r#"
def check():
  return (None,None)
  "#;

  #[tokio::test]
  async fn handler_python_ok() {
    write_script("ok", SCRIPT_OK);

    let handler = PythonHandler {
      check: &Check::default(),
      path: "/tmp".to_string(),
    };

    let spec = Python {
      id: 0,
      check_id: 0,
      script: "ok".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, OK);
    assert_eq!(&result.message, "this is the OK check message");
  }

  #[tokio::test]
  async fn handler_python_critical() {
    write_script("critical", SCRIPT_CRITICAL);

    let handler = PythonHandler {
      check: &Check::default(),
      path: "/tmp".to_string(),
    };

    let spec = Python {
      id: 0,
      check_id: 0,
      script: "critical".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Ok(_)));

    let result = result.unwrap();
    assert_eq!(result.status, CRITICAL);
    assert_eq!(&result.message, "this is the CRITICAL check message");
  }

  #[tokio::test]
  async fn handler_python_missing() {
    let handler = PythonHandler {
      check: &Check::default(),
      path: "/tmp".to_string(),
    };

    let spec = Python {
      id: 0,
      check_id: 0,
      script: "missing".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Err(_)));

    if let Err(err) = result {
      assert_eq!(&err.to_string(), "No such file or directory (os error 2)");
    }
  }

  #[tokio::test]
  async fn handler_python_syntax_error() {
    write_script("syntaxerror", SCRIPT_SYNTAX_ERROR);

    let handler = PythonHandler {
      check: &Check::default(),
      path: "/tmp".to_string(),
    };

    let spec = Python {
      id: 0,
      check_id: 0,
      script: "syntaxerror".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Err(_)));

    if let Err(err) = result {
      assert!(&err.to_string().contains("IndentationError"));
    }
  }

  #[tokio::test]
  async fn handler_python_no_check() {
    write_script("nocheck", "");

    let handler = PythonHandler {
      check: &Check::default(),
      path: "/tmp".to_string(),
    };

    let spec = Python {
      id: 0,
      check_id: 0,
      script: "nocheck".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Err(_)));

    if let Err(err) = result {
      assert!(&err.to_string().contains("AttributeError"));
    }
  }

  #[tokio::test]
  async fn handler_python_wrong_type() {
    write_script("wrongtype", SCRIPT_WRONG_TYPE);

    let handler = PythonHandler {
      check: &Check::default(),
      path: "/tmp".to_string(),
    };

    let spec = Python {
      id: 0,
      check_id: 0,
      script: "wrongtype".to_string(),
    };

    let result = handler.run(&spec, CONTROLLER_ID, Stash::new()).await;
    assert!(matches!(&result, Err(_)));

    if let Err(err) = result {
      assert!(&err.to_string().contains("TypeError"));
    }
  }

  fn write_script(name: &str, payload: &str) {
    use std::fs;

    fs::write(&format!("/tmp/{name}.py"), payload).unwrap();
  }
}
