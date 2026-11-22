use serde::de::DeserializeOwned;
use serde_json::{Map, Value as Json};

use crate::error::{ConfigError, Result};
use crate::reports::{MigrateReport, Problem, SetReport, ValidateReport, Values};
use crate::schema::Schema;

pub const SCHEMA: &str = "schema";
pub const GET: &str = "get";
pub const SET: &str = "set";
pub const RESET: &str = "reset";
pub const MIGRATE: &str = "migrate";
pub const VALIDATE: &str = "validate";
pub const LANG_FLAG: &str = "--lang";
pub const STDIN_FLAG: &str = "--stdin";
pub const DRY_RUN_FLAG: &str = "--dry-run";
pub const CATEGORY_FLAG: &str = "--category";
pub const ALL_FLAG: &str = "--all";
pub const IDS_SEPARATOR: &str = "--";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CliOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

pub trait NamsCli: Send + Sync {
    fn run(&self, args: &[String], stdin: Option<&[u8]>) -> Result<CliOutput>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResetTarget {
    Ids(Vec<String>),
    Category(String),
    All,
}

pub fn decode(output: &CliOutput) -> Result<Json> {
    if output.timed_out {
        return Err(ConfigError::TimedOut);
    }
    let text = output.stdout.trim_start_matches('\u{feff}').trim();
    let document: Json = serde_json::from_str(text).map_err(|error| ConfigError::Output {
        exit_code: output.exit_code,
        detail: detail_of(output, &error.to_string()),
    })?;
    if document.get("ok").and_then(Json::as_bool) == Some(false) {
        let problems: Vec<Problem> = document
            .get("problems")
            .cloned()
            .and_then(|problems| serde_json::from_value(problems).ok())
            .unwrap_or_default();
        return Err(ConfigError::Nams {
            code: text_of(&document, "code"),
            message: text_of(&document, "error"),
            exit_code: document
                .get("exit_code")
                .and_then(Json::as_i64)
                .and_then(|code| i32::try_from(code).ok())
                .or(output.exit_code),
            problems,
        });
    }
    Ok(document)
}

fn detail_of(output: &CliOutput, fallback: &str) -> String {
    const LIMIT: usize = 400;
    let stderr = output.stderr.trim();
    let detail = if stderr.is_empty() { fallback } else { stderr };
    detail.chars().take(LIMIT).collect()
}

fn text_of(document: &Json, key: &str) -> String {
    document.get(key).and_then(Json::as_str).unwrap_or_default().to_owned()
}

fn run_json<T: DeserializeOwned>(cli: &dyn NamsCli, args: Vec<String>, stdin: Option<&[u8]>) -> Result<T> {
    let output = cli.run(&args, stdin)?;
    let document = decode(&output)?;
    serde_json::from_value(document).map_err(|error| ConfigError::Output {
        exit_code: output.exit_code,
        detail: error.to_string(),
    })
}

pub fn schema(cli: &dyn NamsCli, lang: Option<&str>) -> Result<Schema> {
    let mut args = vec![SCHEMA.to_owned()];
    if let Some(lang) = lang.filter(|lang| !lang.is_empty()) {
        args.push(LANG_FLAG.to_owned());
        args.push(lang.to_owned());
    }
    run_json(cli, args, None)
}

pub fn get(cli: &dyn NamsCli) -> Result<Values> {
    run_json(cli, vec![GET.to_owned()], None)
}

pub fn assignments_json(assignments: &[(String, Json)]) -> Vec<u8> {
    let object: Map<String, Json> = assignments.iter().cloned().collect();
    serde_json::to_vec(&Json::Object(object)).unwrap_or_default()
}

pub fn set(cli: &dyn NamsCli, assignments: &[(String, Json)], dry_run: bool) -> Result<SetReport> {
    if assignments.is_empty() {
        return Ok(SetReport {
            dry_run,
            ..SetReport::default()
        });
    }
    let mut args = vec![SET.to_owned(), STDIN_FLAG.to_owned()];
    if dry_run {
        args.push(DRY_RUN_FLAG.to_owned());
    }
    let input = assignments_json(assignments);
    run_json(cli, args, Some(&input))
}

pub fn set_accepted(cli: &dyn NamsCli, assignments: &[(String, Json)]) -> Result<(SetReport, Vec<Problem>)> {
    match set(cli, assignments, false) {
        Ok(report) => Ok((report, Vec::new())),
        Err(error) => {
            let rejected = error.rejected_ids();
            if rejected.is_empty() {
                return Err(error);
            }
            let kept: Vec<(String, Json)> = assignments
                .iter()
                .filter(|(id, _)| !rejected.contains(id))
                .cloned()
                .collect();
            let report = set(cli, &kept, false)?;
            Ok((report, error.problems().to_vec()))
        }
    }
}

pub fn reset(cli: &dyn NamsCli, target: &ResetTarget, dry_run: bool) -> Result<SetReport> {
    let mut args = vec![RESET.to_owned()];
    if dry_run {
        args.push(DRY_RUN_FLAG.to_owned());
    }
    match target {
        ResetTarget::All => args.push(ALL_FLAG.to_owned()),
        ResetTarget::Category(category) => {
            args.push(CATEGORY_FLAG.to_owned());
            args.push(category.clone());
        }
        ResetTarget::Ids(ids) => {
            if ids.is_empty() {
                return Ok(SetReport {
                    dry_run,
                    ..SetReport::default()
                });
            }
            args.push(IDS_SEPARATOR.to_owned());
            args.extend(ids.iter().cloned());
        }
    }
    run_json(cli, args, None)
}

pub fn migrate(cli: &dyn NamsCli, dry_run: bool) -> Result<MigrateReport> {
    let mut args = vec![MIGRATE.to_owned()];
    if dry_run {
        args.push(DRY_RUN_FLAG.to_owned());
    }
    run_json(cli, args, None)
}

pub fn validate(cli: &dyn NamsCli) -> Result<ValidateReport> {
    run_json(cli, vec![VALIDATE.to_owned()], None)
}
