#[cfg(feature = "test-support")]
pub mod archives;
#[cfg(feature = "test-support")]
pub mod fixtures;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::error::{Error, ProcessFailure, Result};
use crate::process::{ChildProcess, ProcessOutput, ProcessRunner, ProcessSpec, StreamKind, last_chars};
use crate::task::CancelToken;

type Matcher = Box<dyn Fn(&ProcessSpec) -> bool + Send + Sync>;

struct Rule {
    matcher: Matcher,
    response: FakeResponse,
}

#[derive(Debug, Clone)]
pub enum FakeResponse {
    Output(ProcessOutput),
    NotFound,
    Running { output: String },
}

#[derive(Default, Clone)]
pub struct FakeRunner {
    rules: Arc<Mutex<Vec<Rule>>>,
    calls: Arc<Mutex<Vec<ProcessSpec>>>,
    killed: Arc<Mutex<Vec<u32>>>,
}

impl FakeRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn respond(
        self,
        matcher: impl Fn(&ProcessSpec) -> bool + Send + Sync + 'static,
        response: FakeResponse,
    ) -> Self {
        self.rules.lock().unwrap().push(Rule {
            matcher: Box::new(matcher),
            response,
        });
        self
    }

    pub fn respond_program(self, program_name: &str, output: ProcessOutput) -> Self {
        let name = program_name.to_ascii_lowercase();
        self.respond(
            move |spec| {
                spec.program
                    .file_name()
                    .map(|file| file.to_string_lossy().to_ascii_lowercase() == name)
                    .unwrap_or(false)
            },
            FakeResponse::Output(output),
        )
    }

    pub fn calls(&self) -> Vec<ProcessSpec> {
        self.calls.lock().unwrap().clone()
    }

    pub fn killed_pids(&self) -> Vec<u32> {
        self.killed.lock().unwrap().clone()
    }

    fn resolve(&self, spec: &ProcessSpec) -> FakeResponse {
        self.calls.lock().unwrap().push(spec.clone());
        self.rules
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|rule| (rule.matcher)(spec))
            .map(|rule| rule.response.clone())
            .unwrap_or(FakeResponse::Output(ProcessOutput::success("")))
    }
}

impl ProcessRunner for FakeRunner {
    fn output(&self, spec: &ProcessSpec, _timeout: Option<Duration>) -> Result<ProcessOutput> {
        match self.resolve(spec) {
            FakeResponse::Output(output) => Ok(output),
            FakeResponse::NotFound => Err(not_found(spec)),
            FakeResponse::Running { output } => Ok(ProcessOutput {
                exit_code: None,
                stdout: output,
                timed_out: true,
                ..ProcessOutput::default()
            }),
        }
    }

    fn stream(
        &self,
        spec: &ProcessSpec,
        cancel: &CancelToken,
        on_chunk: &mut dyn FnMut(StreamKind, &str),
    ) -> Result<ProcessOutput> {
        cancel.check()?;
        let output = self.output(spec, None)?;
        if !output.stdout.is_empty() {
            on_chunk(StreamKind::Stdout, &output.stdout);
        }
        if !output.stderr.is_empty() {
            on_chunk(StreamKind::Stderr, &output.stderr);
        }
        Ok(output)
    }

    fn spawn(&self, spec: &ProcessSpec) -> Result<Box<dyn ChildProcess>> {
        let pid = 1000 + self.calls.lock().unwrap().len() as u32;
        match self.resolve(spec) {
            FakeResponse::NotFound => Err(not_found(spec)),
            FakeResponse::Output(output) => Ok(Box::new(FakeChild {
                pid,
                exit_code: output.exit_code.or(Some(0)),
                output: output.combined(),
                killed: self.killed.clone(),
            })),
            FakeResponse::Running { output } => Ok(Box::new(FakeChild {
                pid,
                exit_code: None,
                output,
                killed: self.killed.clone(),
            })),
        }
    }
}

fn not_found(spec: &ProcessSpec) -> Error {
    Error::Process {
        program: spec.label.clone(),
        failure: ProcessFailure::NotFound,
    }
}

pub struct FakeChild {
    pid: u32,
    exit_code: Option<i32>,
    output: String,
    killed: Arc<Mutex<Vec<u32>>>,
}

impl ChildProcess for FakeChild {
    fn pid(&self) -> u32 {
        self.pid
    }

    fn try_exit_code(&mut self) -> Result<Option<i32>> {
        Ok(self.exit_code)
    }

    fn wait_exit_code(&mut self) -> Result<i32> {
        Ok(self.exit_code.unwrap_or(-1))
    }

    fn kill(&mut self) -> Result<()> {
        self.killed.lock().unwrap().push(self.pid);
        self.exit_code.get_or_insert(-1);
        Ok(())
    }

    fn output_tail(&self, max_chars: usize) -> String {
        last_chars(&self.output, max_chars).to_owned()
    }
}
