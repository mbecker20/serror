use anyhow::Context;
use serde::{Serialize, Deserialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize, Debug)]
pub struct Serror {
    pub error: String,
    pub trace: Vec<String>,
}

impl TryFrom<anyhow::Error> for Serror {
    type Error = anyhow::Error;
    fn try_from(e: anyhow::Error) -> Result<Serror, anyhow::Error> {
        let e = serde_error::Error::new(&*e);
        let e = serde_json::to_string(&e).context("failed to serialize error")?;
        let e: Map<String, Value> =
            serde_json::from_str(&e).context("failed to deserialize error")?;
        let mut trace = Vec::<String>::new();
        collapse_error_into_trace(e, &mut trace);
        let serror = Serror {
            error: trace.get(0).cloned().unwrap_or_default(),
            trace,
        };
        Ok(serror)
    }
}

fn collapse_error_into_trace(mut e: Map<String, Value>, trace: &mut Vec<String>) {
    if let Some(Value::String(description)) = e.remove("description") {
        trace.push(description);
    }
    if let Some(Value::Object(e)) = e.remove("source") {
        collapse_error_into_trace(e, trace)
    }
}