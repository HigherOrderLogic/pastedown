mod error;

pub use self::error::ApiError;
use axum::{http::StatusCode, response::IntoResponse};

pub type ApiResult<T> = Result<(StatusCode, T), (StatusCode, ApiError)>;

pub trait WithStatusCode {
    fn with_status_code(self, status_code: StatusCode) -> (StatusCode, Self);
}

impl<T> WithStatusCode for T
where
    T: IntoResponse,
{
    fn with_status_code(self, status_code: StatusCode) -> (StatusCode, Self) {
        (status_code, self)
    }
}
