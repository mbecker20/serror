use anyhow::Context;

mod serror;

pub use serror::Serror;

#[cfg(feature = "axum")]
mod axum;
#[cfg(feature = "axum")]
pub use crate::axum::{
  AddHeaders, AddHeadersError, AddStatusCode, AddStatusCodeError, Error, Json, JsonError, Result,
};

pub fn serialize_error(e: &anyhow::Error) -> String {
  try_serialize_error(e).unwrap_or_else(|_| format!("{e:#?}"))
}

pub fn try_serialize_error(e: &anyhow::Error) -> anyhow::Result<String> {
  let serror: Serror = e.into();
  let res = serde_json::to_string(&serror)?;
  Ok(res)
}

pub fn serialize_error_pretty(e: &anyhow::Error) -> String {
  try_serialize_error_pretty(e).unwrap_or_else(|_| format!("{e:#?}"))
}

pub fn try_serialize_error_pretty(e: &anyhow::Error) -> anyhow::Result<String> {
  let serror: Serror = e.into();
  let res = serde_json::to_string_pretty(&serror)?;
  Ok(res)
}

pub fn deserialize_error(json: String) -> anyhow::Error {
  serror_into_error(deserialize_serror(json))
}

pub fn deserialize_serror(json: String) -> Serror {
  try_deserialize_serror(&json).unwrap_or_else(|_| Serror {
    error: json.clone(),
    trace: Default::default(),
  })
}

pub fn try_deserialize_serror(json: &str) -> anyhow::Result<Serror> {
  serde_json::from_str(json).context("failed to deserialize string into Serror")
}

fn serror_into_error(mut serror: Serror) -> anyhow::Error {
  let mut e = match serror.trace.pop() {
    None => return anyhow::Error::msg(serror.error),
    Some(msg) => anyhow::Error::msg(msg),
  };

  while let Some(msg) = serror.trace.pop() {
    e = e.context(msg);
  }

  e = e.context(serror.error);

  e
}
