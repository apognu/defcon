#[derive(Deserialize)]
pub struct UserPatch {
  pub name: Option<String>,
  pub email: Option<String>,
  pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct NewPassword {
  pub password: String,
  pub new_password: String,
}
