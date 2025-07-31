mod create_user;
mod edit_self;
mod get_self;
mod get_user;
mod user_login;
mod user_logout;
mod user_refresh;

use axum::{
    Router,
    routing::{get, post},
};

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(create_user::handler))
        .route("/me", get(get_self::handler).patch(edit_self::handler))
        .route("/{uuid}", get(get_user::handler))
        .route("/auth/login", post(user_login::handler))
        .route("/auth/logout", post(user_logout::handler))
        .route("/auth/refresh", post(user_refresh::handler))
}
