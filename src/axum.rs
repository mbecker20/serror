use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};
use axum_extra::{headers::ContentType, TypedHeader};

use crate::serialize_error;

pub type Result<T> = std::result::Result<T, Error>;

/// Intermediate error type which can be converted to from any error using `?`.
/// The standard `impl From<E> for Error` will attach StatusCode::INTERNAL_SERVER_ERROR,
/// so if an alternative StatusCode is desired, you should use `.status_code` ([AddStatusCode])
/// to add the status before using `?`.
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

/// Convenience trait to convert any Error into serror::Error by adding status
/// and converting error into anyhow error.
pub trait AddStatusCodeError: Into<anyhow::Error> {
  fn status_code(self, status_code: StatusCode) -> Error {
    Error(status_code, self.into())
  }
}

impl<E> AddStatusCodeError for E where E: Into<anyhow::Error> {}

/// Convenience trait to convert Result into serror::Result by adding status to the inner error, if it exists.
pub trait AddStatusCode<T, E>: Into<std::result::Result<T, E>>
where
  E: Into<anyhow::Error>,
{
  fn status_code(self, status_code: StatusCode) -> Result<T> {
    self.into().map_err(|e| e.status_code(status_code))
  }
}

impl<R, T, E> AddStatusCode<T, E> for R
where
  R: Into<std::result::Result<T, E>>,
  E: Into<anyhow::Error>,
{
}
