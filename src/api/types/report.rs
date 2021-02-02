#[derive(Debug, Serialize, Deserialize)]
pub struct ReportEvent {
  pub check: String,
  pub status: u8,
  pub message: String,
}
