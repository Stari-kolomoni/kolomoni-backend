use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserRegister {
    pub username: String,
    pub display_name: Option<String>,
    pub password: String
}