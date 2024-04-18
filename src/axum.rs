use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};
use axum_extra::{headers::ContentType, TypedHeader};

use crate::serialize_error;

pub type Result<T> = std::result::Result<T, Error>;

/// Intermediate error type which can be converted to from any error using `?`.
/// The standard `impl From<E> for Error` will attach StatusCode::INTERNAL_SERVER_ERROR,
/// so if an alternative StatusCode is desired, you must use `.map_err` to serror::Error before using `?`.
pub struct Error(pub StatusCode, pub anyhow::Error);

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    (
      self.0,
      TypedHeader(ContentType::json()),
      serialize_error(&self.1),
    )
      .into_response()
  }
}

impl<E> From<E> for Error
where
  E: Into<anyhow::Error>,
{
  fn from(err: E) -> Self {
    Self(StatusCode::INTERNAL_SERVER_ERROR, err.into())
  }
}
