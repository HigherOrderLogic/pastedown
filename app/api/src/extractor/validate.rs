use crate::response::{ApiError, WithStatusCode};
use axum::{
    extract::{rejection::JsonRejection, FromRequest, FromRequestParts, Request},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use validator::{Validate, ValidationErrors};

#[derive(FromRequest)]
#[from_request(via(AxumJson), rejection(JsonRejection))]
pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        AxumJson(self.0).into_response()
    }
}

impl<T> Validate for Json<T>
where
    T: Validate,
{
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.0.validate()
    }
}

pub struct Validated<T>(pub T);

impl<T> Validate for Validated<T>
where
    T: Validate,
{
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.0.validate()
    }
}

pub enum ValidationRejection<T> {
    ExtractorError(T),
    ValidationError(ValidationErrors),
}

impl<T> IntoResponse for ValidationRejection<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Self::ExtractorError(e) => {
                let (parts, _) = e.into_response().into_parts();

                (parts, ApiError::with_detail("Invalid request")).into_response()
            },
            Self::ValidationError(e) => ApiError::from_error(&e)
                .with_status_code(StatusCode::BAD_REQUEST)
                .into_response(),
        }
    }
}

impl<S, T> FromRequestParts<S> for Validated<T>
where
    S: Send + Sync,
    T: FromRequestParts<S> + Validate,
{
    type Rejection = ValidationRejection<<T as FromRequestParts<S>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let extractor_result = T::from_request_parts(parts, state)
            .await
            .map_err(|e| ValidationRejection::ExtractorError(e))?;
        extractor_result
            .validate()
            .map_err(ValidationRejection::ValidationError)?;

        Ok(Self(extractor_result))
    }
}

impl<S, T> FromRequest<S> for Validated<T>
where
    S: Send + Sync,
    T: FromRequest<S> + Validate,
{
    type Rejection = ValidationRejection<<T as FromRequest<S>>::Rejection>;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let extractor_result = T::from_request(req, state)
            .await
            .map_err(|e| ValidationRejection::ExtractorError(e))?;
        extractor_result
            .validate()
            .map_err(ValidationRejection::ValidationError)?;

        Ok(Self(extractor_result))
    }
}
