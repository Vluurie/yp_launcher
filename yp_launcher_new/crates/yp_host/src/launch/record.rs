use std::path::Path;
use std::time::Duration;

use jiff::Timestamp;
use serde_json::{Map, Value};
use yp_core::process::last_chars;
use yp_core::{AppPaths, Result};

pub const TAIL_CHARS: usize = 6 * 1024;
const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchRecord {
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
    pub exit_code: Option<i32>,
    pub label: String,
    pub output_tail: String,
}

impl LaunchRecord {
    pub fn started(label: impl Into<String>) -> Self {
        Self {
            started_at: Timestamp::now(),
            ended_at: None,
            exit_code: None,
            label: label.into(),
            output_tail: String::new(),
        }
    }

    pub fn duration(&self) -> Option<Duration> {
        let ended = self.ended_at?;
        let span = ended.duration_since(self.started_at);
        Duration::try_from(span).ok()
    }

    pub fn still_running(&self) -> bool {
        self.ended_at.is_none()
    }

    pub fn finished(&self, code: i32, tail: impl Into<String>) -> Self {
        Self {
            ended_at: Some(Timestamp::now()),
            exit_code: Some(code),
            output_tail: tail.into(),
            ..self.clone()
        }
    }

    pub fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("started_at".into(), Value::String(format_timestamp(self.started_at)));
        if let Some(ended) = self.ended_at {
            map.insert("ended_at".into(), Value::String(format_timestamp(ended)));
        }
        if let Some(code) = self.exit_code {
            map.insert("exit_code".into(), Value::from(code));
        }
        map.insert("label".into(), Value::String(self.label.clone()));
        map.insert("output_tail".into(), Value::String(self.output_tail.clone()));
        Value::Object(map)
    }

    pub fn from_json(json: &Value) -> Option<Self> {
        let map = json.as_object()?;
        let started_at = parse_timestamp(map.get("started_at")?)?;
        let ended_at = map.get("ended_at").and_then(parse_timestamp);
        let exit_code = map
            .get("exit_code")
            .and_then(Value::as_i64)
            .and_then(|code| i32::try_from(code).ok());
        let text = |key: &str| map.get(key).and_then(Value::as_str).unwrap_or_default().to_owned();
        Some(Self {
            started_at,
            ended_at,
            exit_code,
            label: text("label"),
            output_tail: text("output_tail"),
        })
    }
}

pub fn format_timestamp(stamp: Timestamp) -> String {
    stamp.strftime(TIMESTAMP_FORMAT).to_string()
}

fn parse_timestamp(value: &Value) -> Option<Timestamp> {
    value.as_str()?.trim().parse().ok()
}

pub fn tail_of(output: &str) -> String {
    last_chars(output, TAIL_CHARS).to_owned()
}

pub fn write_file(file: &Path, record: &LaunchRecord) -> Result<()> {
    let text = serde_json::to_string(&record.to_json()).map_err(|source| yp_core::Error::Json {
        path: Some(file.to_path_buf()),
        source,
    })?;
    yp_core::fs::write_text_atomic(file, &text)
}

pub fn write(paths: &AppPaths, record: &LaunchRecord) -> Result<()> {
    write_file(&paths.launch_record(), record)
}

pub fn write_quiet(paths: &AppPaths, record: &LaunchRecord) -> bool {
    match write(paths, record) {
        Ok(()) => true,
        Err(err) => {
            log::warn!("could not write launch record: {err}");
            false
        }
    }
}

pub fn read_file(file: &Path) -> Option<LaunchRecord> {
    let text = std::fs::read_to_string(file).ok()?;
    LaunchRecord::from_json(&serde_json::from_str(&text).ok()?)
}

pub fn read(paths: &AppPaths) -> Option<LaunchRecord> {
    read_file(&paths.launch_record())
}
