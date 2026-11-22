use serde_json::Value;

pub const STATUS_OK: &str = "ok";
pub const STATUS_FAIL: &str = "fail";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerifyStatus {
    Ok,
    Failed,
    NoRuntime,
    SteamNotRunning,
    TimedOut,
    CannotParse,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyCheck {
    pub name: String,
    pub status: String,
    pub detail: Option<String>,
}

impl VerifyCheck {
    pub fn ok(&self) -> bool {
        self.status == STATUS_OK
    }

    pub fn from_json(json: &Value) -> Self {
        let text = |key: &str| json.get(key).and_then(Value::as_str).map(str::to_owned);
        Self {
            name: text("name").unwrap_or_default(),
            status: text("status").unwrap_or_else(|| STATUS_FAIL.to_owned()),
            detail: text("detail"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyResult {
    pub ok: bool,
    pub checks: Vec<VerifyCheck>,
}

impl VerifyResult {
    pub fn from_json(json: &Value) -> Self {
        Self {
            ok: json.get("ok").and_then(Value::as_bool).unwrap_or(false),
            checks: json
                .get("checks")
                .and_then(Value::as_array)
                .map(|checks| {
                    checks
                        .iter()
                        .filter(|check| check.is_object())
                        .map(VerifyCheck::from_json)
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    pub fn check(&self, name: &str) -> Option<&VerifyCheck> {
        self.checks.iter().find(|check| check.name == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyOutcome {
    pub status: VerifyStatus,
    pub result: Option<VerifyResult>,
    pub exit_code: Option<i32>,
    pub raw: Option<String>,
}

impl VerifyOutcome {
    pub fn status(status: VerifyStatus) -> Self {
        Self {
            status,
            result: None,
            exit_code: None,
            raw: None,
        }
    }

    pub fn with_exit_code(mut self, code: Option<i32>) -> Self {
        self.exit_code = code;
        self
    }

    pub fn with_raw(mut self, raw: impl Into<String>) -> Self {
        self.raw = Some(raw.into());
        self
    }
}

pub fn outcome_from_output(exit_code: Option<i32>, stdout: &str, stderr: &str) -> VerifyOutcome {
    if matches!(exit_code, Some(20..=22)) {
        return VerifyOutcome::status(VerifyStatus::SteamNotRunning).with_exit_code(exit_code);
    }
    if exit_code == Some(64) {
        return VerifyOutcome::status(VerifyStatus::Error).with_exit_code(exit_code);
    }
    let decoded: Option<Value> = serde_json::from_str(stdout).ok();
    let Some(decoded) = decoded.filter(|value| value.get("checks").is_some_and(Value::is_array)) else {
        return VerifyOutcome::status(VerifyStatus::CannotParse)
            .with_exit_code(exit_code)
            .with_raw(format!("{stdout}{stderr}"));
    };
    let result = VerifyResult::from_json(&decoded);
    VerifyOutcome {
        status: if result.ok { VerifyStatus::Ok } else { VerifyStatus::Failed },
        result: Some(result),
        exit_code,
        raw: None,
    }
}
