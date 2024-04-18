use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};
use axum_extra::{headers::ContentType, TypedHeader};

use crate::serialize_error;

pub type Result<const HTTP_CODE: u16, T> = std::result::Result<T, Error<HTTP_CODE>>;

pub struct Error<const HTTP_CODE: u16>(anyhow::Error);

impl<const HTTP_CODE: u16> IntoResponse for Error<HTTP_CODE> {
  fn into_response(self) -> Response {
    (
      StatusCode::from_u16(HTTP_CODE).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
      TypedHeader(ContentType::json()),
      serialize_error(&self.0),
    )
      .into_response()
  }
}

impl<const HTTP_CODE: u16, E> From<E> for Error<HTTP_CODE>
where
  E: Into<anyhow::Error>,
{
  fn from(err: E) -> Self {
    Self(err.into())
  }
}
