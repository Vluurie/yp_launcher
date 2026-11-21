#![allow(dead_code)]

use std::path::Path;
use std::sync::Mutex;

use indexmap::IndexMap;
use serde_json::{Value as Json, json};
use tempfile::TempDir;
use yp_config::Result;
use yp_config::cli::{CliOutput, NamsCli};
use yp_core::GamePaths;

pub struct GameDir {
    pub dir: TempDir,
    pub game: GamePaths,
}

impl GameDir {
    pub fn empty() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let game = GamePaths::new(dir.path().join("game"));
        std::fs::create_dir_all(game.root()).unwrap();
        Self { dir, game }
    }

    pub fn root(&self) -> &Path {
        self.game.root()
    }
}

#[derive(Default)]
pub struct FakeCli {
    pub values: Mutex<IndexMap<String, Json>>,
    pub rejected: Vec<String>,
    pub calls: Mutex<Vec<Vec<String>>>,
}

impl FakeCli {
    pub fn with(values: &[(&str, Json)]) -> Self {
        Self {
            values: Mutex::new(
                values
                    .iter()
                    .map(|(id, value)| ((*id).to_owned(), value.clone()))
                    .collect(),
            ),
            ..Self::default()
        }
    }

    pub fn rejecting(mut self, ids: &[&str]) -> Self {
        self.rejected = ids.iter().map(|id| (*id).to_owned()).collect();
        self
    }

    pub fn value(&self, id: &str) -> Option<Json> {
        self.values.lock().unwrap().get(id).cloned()
    }

    pub fn set_calls(&self) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|args| args[0] == "set")
            .count()
    }
}

fn answer(document: Json) -> Result<CliOutput> {
    Ok(CliOutput {
        exit_code: Some(if document["ok"] == json!(false) { 65 } else { 0 }),
        stdout: document.to_string(),
        ..CliOutput::default()
    })
}

impl NamsCli for FakeCli {
    fn run(&self, args: &[String], stdin: Option<&[u8]>) -> Result<CliOutput> {
        self.calls.lock().unwrap().push(args.to_vec());
        match args[0].as_str() {
            "get" => {
                let values = self.values.lock().unwrap().clone();
                answer(json!({ "ok": true, "values": values }))
            }
            "set" => {
                let input: IndexMap<String, Json> =
                    serde_json::from_slice(stdin.unwrap_or_default()).unwrap();
                let rejected: Vec<&String> = input
                    .keys()
                    .filter(|id| self.rejected.contains(id))
                    .collect();
                if !rejected.is_empty() {
                    let problems: Vec<Json> = rejected
                        .iter()
                        .map(|id| json!({ "code": "invalid_value", "severity": "error", "message": "rejected", "id": id }))
                        .collect();
                    return answer(json!({ "ok": false, "code": "validation_failed", "error": "rejected", "problems": problems }));
                }
                let mut values = self.values.lock().unwrap();
                let mut changed = Vec::new();
                for (id, value) in input {
                    let old = values.insert(id.clone(), value.clone()).unwrap_or(Json::Null);
                    changed.push(json!({ "id": id, "old": old, "new": value }));
                }
                answer(json!({ "ok": true, "changed": changed }))
            }
            other => panic!("unexpected command {other}"),
        }
    }
}
