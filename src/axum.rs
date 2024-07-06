use std::error::Error as StdError;

use anyhow::anyhow;
use axum::{
  body::Body,
  extract::{rejection::JsonRejection, FromRequest},
  http::{header::IntoHeaderName, HeaderMap, HeaderValue, StatusCode},
  response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::serialize_error;

pub type Result<T> = std::result::Result<T, Error>;

/// Intermediate error type which can be converted to from any error using `?`.
/// The standard `impl From<E> for Error` will attach StatusCode::INTERNAL_SERVER_ERROR,
/// so if an alternative StatusCode is desired, you should use `.status_code` ([AddStatusCode] or [AddStatusCodeError])
/// to add the status and `.header` ([AddHeader] or [AddHeaderError]) before using `?`.
pub struct Error {
  pub status: StatusCode,
  pub headers: HeaderMap,
  pub error: anyhow::Error,
}

impl Error {
  pub fn status_code(mut self, status_code: StatusCode) -> Error {
    self.status = status_code;
    self
  }

  pub fn header(mut self, name: impl IntoHeaderName, value: HeaderValue) -> Error {
    self.headers.append(name, value);
    self
  }

  pub fn headers(mut self, headers: HeaderMap) -> Error {
    self.headers = headers;
    self
  }
}

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    let mut response = Response::new(Body::new(serialize_error(&self.error)));
    *response.status_mut() = self.status;

    let headers = response.headers_mut();
    headers.append("Content-Type", HeaderValue::from_static("application/json"));
    headers.extend(self.headers);

    response
  }
}

impl<E> From<E> for Error
where
  E: Into<anyhow::Error>,
{
  fn from(err: E) -> Self {
    Self {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      headers: Default::default(),
      error: err.into(),
    }
  }
}

/// Convenience trait to convert any Error into serror::Error by adding status
/// and converting error into anyhow error.
pub trait AddStatusCodeError: Into<anyhow::Error> {
  fn status_code(self, status_code: StatusCode) -> Error {
    Error {
      status: status_code,
      headers: Default::default(),
      error: self.into(),
    }
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

/// Convenience trait to convert any Error into serror::Error by adding headers
/// and converting error into anyhow error.
pub trait AddHeadersError: Into<anyhow::Error> {
  fn header(self, name: impl IntoHeaderName, value: HeaderValue) -> Error {
    let mut headers = HeaderMap::with_capacity(1);
    headers.append(name, value);
    Error {
      headers,
      status: StatusCode::INTERNAL_SERVER_ERROR,
      error: self.into(),
    }
  }
  fn headers(self, headers: HeaderMap) -> Error {
    Error {
      headers,
      status: StatusCode::INTERNAL_SERVER_ERROR,
      error: self.into(),
    }
  }
}

impl<E> AddHeadersError for E where E: Into<anyhow::Error> {}

/// Convenience trait to add headers to a serror::Result directly.
pub trait AddHeaders<T, E>: Into<std::result::Result<T, E>>
where
  E: Into<anyhow::Error>,
{
  fn header(self, name: impl IntoHeaderName, value: HeaderValue) -> Result<T> {
    self.into().map_err(|e| e.header(name, value))
  }

  /// Some headers might want to be attached in both Ok case and Err case.
  /// Borrow headers here so they can be used later, as they will only be cloned in err case.
  fn headers(self, headers: &HeaderMap) -> Result<T> {
    self.into().map_err(|e| e.headers(headers.clone()))
  }
}

impl<R, T, E> AddHeaders<T, E> for R
where
  R: Into<std::result::Result<T, E>>,
  E: Into<anyhow::Error>,
{
}

/// Wrapper for axum::Json that converts parsing error to serror::Error
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(JsonError))]
pub struct Json<T>(pub T);

impl<T: Serialize> IntoResponse for Json<T> {
  fn into_response(self) -> Response {
    axum::Json(self.0).into_response()
  }
}

pub struct JsonError(Error);

/// Convert the JsonRejection into JsonError(serror::Error)
impl From<JsonRejection> for JsonError {
  fn from(rejection: JsonRejection) -> Self {
    Self(Error {
      status: rejection.status(),
      headers: Default::default(),
      error: anyhow!("{:?}", rejection.source())
        .context("Failed to deserialize the JSON body into the target type"),
    })
  }
}

impl IntoResponse for JsonError {
  fn into_response(self) -> Response {
    self.0.into_response()
  }
}
