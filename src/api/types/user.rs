#[derive(Serialize, Deserialize)]
pub struct Credentials {
  pub email: String,
  pub password: String,
}

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

#[derive(Serialize)]
pub struct ApiKey {
  pub api_key: String,
}
