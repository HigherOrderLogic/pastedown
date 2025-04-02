use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct User {
    pub uuid: Uuid,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
}
