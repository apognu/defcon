use crate::model::specs::SpecMeta;

#[derive(Debug, Serialize, Deserialize)]
pub struct Unsupported;

impl SpecMeta for Unsupported {
  fn name(&self) -> &'static str {
    "Unsupported handler"
  }

  fn fields(&self) -> Vec<(&'static str, String)> {
    vec![]
  }
}
