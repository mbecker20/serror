use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};
use axum_extra::{headers::ContentType, TypedHeader};

use crate::serialize_error;

pub type AppResult<T> = Result<T, AppError>;

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      TypedHeader(ContentType::json()),
      serialize_error(self.0),
    )
      .into_response()
  }
}

impl<E> From<E> for AppError
where
  E: Into<anyhow::Error>,
{
  fn from(err: E) -> Self {
    Self(err.into())
  }
}

pub type AuthResult<T> = Result<T, AuthError>;

pub struct AuthError(anyhow::Error);

impl IntoResponse for AuthError {
  fn into_response(self) -> Response {
    (
      StatusCode::UNAUTHORIZED,
      TypedHeader(ContentType::json()),
      serialize_error(self.0),
    )
      .into_response()
  }
}

impl<E> From<E> for AuthError
where
  E: Into<anyhow::Error>,
{
  fn from(err: E) -> Self {
    Self(err.into())
  }
}
