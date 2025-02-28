use serde::Serialize;

#[derive(Serialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}
