use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::error::{Error, ProcessFailure, Result};
use crate::host::{HostEnv, Os};
use crate::task::CancelToken;

const POLL_INTERVAL: Duration = Duration::from_millis(25);
const TAIL_CAPACITY: usize = 64 * 1024;

static HELPER_PIDS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn helper_pids() -> MutexGuard<'static, Vec<u32>> {
    HELPER_PIDS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn excluding_helpers<T>(probe: impl FnOnce(&[u32]) -> T) -> T {
    let helpers = helper_pids();
    probe(&helpers)
}

struct HelperRun {
    registry: Option<MutexGuard<'static, Vec<u32>>>,
    pid: Option<u32>,
}

impl HelperRun {
    fn begin() -> Self {
        Self {
            registry: Some(helper_pids()),
            pid: None,
        }
    }

    fn started(&mut self, pid: u32) {
        if let Some(mut registry) = self.registry.take() {
            registry.push(pid);
        }
        self.pid = Some(pid);
    }
}

impl Drop for HelperRun {
    fn drop(&mut self) {
        drop(self.registry.take());
        if let Some(pid) = self.pid {
            helper_pids().retain(|id| *id != pid);
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessSpec {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
    pub label: String,
    pub hide_console: bool,
}

impl ProcessSpec {
    pub fn new(program: impl Into<PathBuf>) -> Self {
        let program = program.into();
        let label = program
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            program,
            label,
            ..Self::default()
        }
    }

    pub fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn cwd(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cwd = Some(dir.into());
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn envs(mut self, vars: impl IntoIterator<Item = (String, String)>) -> Self {
        self.env.extend(vars);
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn hide_console(mut self) -> Self {
        self.hide_console = true;
        self
    }

    pub fn args_lossy(&self) -> Vec<String> {
        self.args.iter().map(|arg| arg.to_string_lossy().into_owned()).collect()
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args).envs(&self.env);
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }
        #[cfg(windows)]
        if self.hide_console {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        command
    }

    fn spawn_error(&self, err: std::io::Error) -> Error {
        let failure = if err.kind() == std::io::ErrorKind::NotFound {
            ProcessFailure::NotFound
        } else {
            ProcessFailure::Spawn(err.to_string())
        };
        Error::Process {
            program: self.label.clone(),
            failure,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

impl ProcessOutput {
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            exit_code: Some(0),
            stdout: stdout.into(),
            ..Self::default()
        }
    }

    pub fn failure(code: i32, stderr: impl Into<String>) -> Self {
        Self {
            exit_code: Some(code),
            stderr: stderr.into(),
            ..Self::default()
        }
    }

    pub fn is_success(&self) -> bool {
        self.exit_code == Some(0) && !self.timed_out
    }

    pub fn combined(&self) -> String {
        match (self.stdout.is_empty(), self.stderr.is_empty()) {
            (_, true) => self.stdout.clone(),
            (true, false) => self.stderr.clone(),
            (false, false) => format!("{}\n{}", self.stdout, self.stderr),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamKind {
    Stdout,
    Stderr,
}

pub trait ProcessRunner: Send + Sync {
    fn output(&self, spec: &ProcessSpec, timeout: Option<Duration>) -> Result<ProcessOutput>;
    fn stream(
        &self,
        spec: &ProcessSpec,
        cancel: &CancelToken,
        on_chunk: &mut dyn FnMut(StreamKind, &str),
    ) -> Result<ProcessOutput>;
    fn spawn(&self, spec: &ProcessSpec) -> Result<Box<dyn ChildProcess>>;
}

pub trait ChildProcess: Send {
    fn pid(&self) -> u32;
    fn try_exit_code(&mut self) -> Result<Option<i32>>;
    fn wait_exit_code(&mut self) -> Result<i32>;
    fn kill(&mut self) -> Result<()>;
    fn output_tail(&self, max_chars: usize) -> String;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemRunner;

impl ProcessRunner for SystemRunner {
    fn output(&self, spec: &ProcessSpec, timeout: Option<Duration>) -> Result<ProcessOutput> {
        let mut helper = HelperRun::begin();
        let mut child = spec
            .command()
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| spec.spawn_error(err))?;
        helper.started(child.id());
        let stdout = read_all(child.stdout.take());
        let stderr = read_all(child.stderr.take());
        let deadline = timeout.map(|limit| Instant::now() + limit);
        let mut timed_out = false;
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|err| spec.spawn_error(err))? {
                break Some(status);
            }
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                let _ = child.kill();
                let _ = child.wait();
                timed_out = true;
                break None;
            }
            std::thread::sleep(POLL_INTERVAL);
        };
        Ok(ProcessOutput {
            exit_code: status.and_then(|status| status.code()),
            stdout: join_text(stdout),
            stderr: join_text(stderr),
            timed_out,
        })
    }

    fn stream(
        &self,
        spec: &ProcessSpec,
        cancel: &CancelToken,
        on_chunk: &mut dyn FnMut(StreamKind, &str),
    ) -> Result<ProcessOutput> {
        let mut helper = HelperRun::begin();
        let mut child = spec
            .command()
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| spec.spawn_error(err))?;
        helper.started(child.id());
        let (sender, receiver) = mpsc::channel();
        let readers = [
            child.stdout.take().map(|pipe| forward(pipe, StreamKind::Stdout, sender.clone())),
            child.stderr.take().map(|pipe| forward(pipe, StreamKind::Stderr, sender.clone())),
        ];
        drop(sender);
        let mut output = ProcessOutput::default();
        let result = pump(&receiver, cancel, &mut child, &mut output, on_chunk);
        for reader in readers.into_iter().flatten() {
            let _ = reader.join();
        }
        result?;
        let status = child.wait().map_err(|err| spec.spawn_error(err))?;
        output.exit_code = status.code();
        Ok(output)
    }

    fn spawn(&self, spec: &ProcessSpec) -> Result<Box<dyn ChildProcess>> {
        let mut child = spec
            .command()
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| spec.spawn_error(err))?;
        let tail = Arc::new(Mutex::new(String::new()));
        if let Some(pipe) = child.stdout.take() {
            capture_tail(pipe, tail.clone());
        }
        if let Some(pipe) = child.stderr.take() {
            capture_tail(pipe, tail.clone());
        }
        Ok(Box::new(SystemChild {
            child,
            tail,
            label: spec.label.clone(),
        }))
    }
}

fn pump(
    receiver: &Receiver<(StreamKind, String)>,
    cancel: &CancelToken,
    child: &mut Child,
    output: &mut ProcessOutput,
    on_chunk: &mut dyn FnMut(StreamKind, &str),
) -> Result<()> {
    loop {
        if cancel.is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Error::Cancelled);
        }
        match receiver.recv_timeout(POLL_INTERVAL) {
            Ok((kind, text)) => {
                match kind {
                    StreamKind::Stdout => output.stdout.push_str(&text),
                    StreamKind::Stderr => output.stderr.push_str(&text),
                }
                on_chunk(kind, &text);
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
    }
}

struct SystemChild {
    child: Child,
    tail: Arc<Mutex<String>>,
    label: String,
}

impl SystemChild {
    fn failure(&self, err: std::io::Error) -> Error {
        Error::Process {
            program: self.label.clone(),
            failure: ProcessFailure::Spawn(err.to_string()),
        }
    }
}

impl ChildProcess for SystemChild {
    fn pid(&self) -> u32 {
        self.child.id()
    }

    fn try_exit_code(&mut self) -> Result<Option<i32>> {
        match self.child.try_wait() {
            Ok(status) => Ok(status.map(|status| status.code().unwrap_or(-1))),
            Err(err) => Err(self.failure(err)),
        }
    }

    fn wait_exit_code(&mut self) -> Result<i32> {
        match self.child.wait() {
            Ok(status) => Ok(status.code().unwrap_or(-1)),
            Err(err) => Err(self.failure(err)),
        }
    }

    fn kill(&mut self) -> Result<()> {
        match self.child.kill() {
            Ok(()) => {
                let _ = self.child.wait();
                Ok(())
            }
            Err(err) if err.kind() == std::io::ErrorKind::InvalidInput => Ok(()),
            Err(err) => Err(self.failure(err)),
        }
    }

    fn output_tail(&self, max_chars: usize) -> String {
        let tail = self.tail.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        last_chars(&tail, max_chars).to_owned()
    }
}

pub fn last_chars(text: &str, max_chars: usize) -> &str {
    let count = text.chars().count();
    if count <= max_chars {
        return text;
    }
    let skip = count - max_chars;
    let start = text.char_indices().nth(skip).map(|(index, _)| index).unwrap_or(text.len());
    &text[start..]
}

#[derive(Debug, Default)]
pub struct Utf8Decoder {
    pending: Vec<u8>,
}

impl Utf8Decoder {
    pub fn push(&mut self, bytes: &[u8]) -> String {
        self.pending.extend_from_slice(bytes);
        let mut out = String::new();
        loop {
            match std::str::from_utf8(&self.pending) {
                Ok(text) => {
                    out.push_str(text);
                    self.pending.clear();
                    return out;
                }
                Err(err) => {
                    let valid = err.valid_up_to();
                    out.push_str(std::str::from_utf8(&self.pending[..valid]).unwrap_or_default());
                    match err.error_len() {
                        Some(bad) => {
                            out.push(char::REPLACEMENT_CHARACTER);
                            self.pending.drain(..valid + bad);
                        }
                        None => {
                            self.pending.drain(..valid);
                            return out;
                        }
                    }
                }
            }
        }
    }

    pub fn finish(&mut self) -> String {
        let rest = String::from_utf8_lossy(&self.pending).into_owned();
        self.pending.clear();
        rest
    }
}

fn read_all(pipe: Option<impl Read + Send + 'static>) -> Option<JoinHandle<Vec<u8>>> {
    pipe.map(|mut pipe| {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = pipe.read_to_end(&mut bytes);
            bytes
        })
    })
}

fn join_text(handle: Option<JoinHandle<Vec<u8>>>) -> String {
    handle
        .and_then(|handle| handle.join().ok())
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default()
}

fn forward(
    mut pipe: impl Read + Send + 'static,
    kind: StreamKind,
    sender: mpsc::Sender<(StreamKind, String)>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut decoder = Utf8Decoder::default();
        let mut buffer = [0u8; 8192];
        loop {
            match pipe.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(read) => {
                    let text = decoder.push(&buffer[..read]);
                    if !text.is_empty() && sender.send((kind, text)).is_err() {
                        return;
                    }
                }
            }
        }
        let rest = decoder.finish();
        if !rest.is_empty() {
            let _ = sender.send((kind, rest));
        }
    })
}

fn capture_tail(mut pipe: impl Read + Send + 'static, tail: Arc<Mutex<String>>) {
    std::thread::spawn(move || {
        let mut decoder = Utf8Decoder::default();
        let mut buffer = [0u8; 8192];
        loop {
            let text = match pipe.read(&mut buffer) {
                Ok(0) | Err(_) => decoder.finish(),
                Ok(read) => decoder.push(&buffer[..read]),
            };
            let done = text.is_empty() && decoder.pending.is_empty();
            if !text.is_empty() {
                let mut tail = tail.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                tail.push_str(&text);
                if tail.len() > TAIL_CAPACITY * 2 {
                    let keep = last_chars(&tail, TAIL_CAPACITY).to_owned();
                    *tail = keep;
                }
            }
            if done {
                break;
            }
        }
    });
}

pub fn find_on_path(host: &HostEnv, names: &[&str]) -> Option<PathBuf> {
    let path = host.var("PATH")?;
    let extensions: Vec<String> = match host.os() {
        Os::Windows => {
            let mut list: Vec<String> = host
                .var("PATHEXT")
                .unwrap_or(".EXE;.CMD;.BAT;.COM")
                .split(';')
                .filter(|ext| !ext.is_empty())
                .map(str::to_ascii_lowercase)
                .collect();
            list.insert(0, String::new());
            list
        }
        Os::Linux | Os::MacOs => vec![String::new()],
    };
    std::env::split_paths(path).find_map(|dir| {
        names.iter().find_map(|name| {
            extensions
                .iter()
                .map(|ext| dir.join(format!("{name}{ext}")))
                .find(|candidate| is_file(candidate))
        })
    })
}

fn is_file(path: &Path) -> bool {
    path.metadata().map(|meta| meta.is_file()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_holds_back_split_multibyte_sequences() {
        let bytes = "äöü€".as_bytes();
        let mut decoder = Utf8Decoder::default();
        let mut out = String::new();
        for byte in bytes {
            out.push_str(&decoder.push(std::slice::from_ref(byte)));
        }
        out.push_str(&decoder.finish());
        assert_eq!(out, "äöü€");
    }

    #[test]
    fn decoder_replaces_invalid_bytes() {
        let mut decoder = Utf8Decoder::default();
        assert_eq!(decoder.push(&[b'a', 0xFF, b'b']), "a\u{FFFD}b");
    }

    #[test]
    fn last_chars_counts_characters() {
        assert_eq!(last_chars("abcdef", 3), "def");
        assert_eq!(last_chars("äöü", 2), "öü");
        assert_eq!(last_chars("ab", 5), "ab");
    }

    #[test]
    fn a_helper_run_is_listed_until_it_ends() {
        let pid = 4_242_424_242;
        let mut run = HelperRun::begin();
        run.started(pid);
        assert!(excluding_helpers(|helpers| helpers.contains(&pid)));
        drop(run);
        assert!(!excluding_helpers(|helpers| helpers.contains(&pid)));

        let failed = HelperRun::begin();
        drop(failed);
        assert!(!excluding_helpers(|helpers| helpers.contains(&pid)));
    }

    #[test]
    fn missing_program_reports_not_found() {
        let spec = ProcessSpec::new("definitely-not-a-real-program-yp");
        let err = SystemRunner.output(&spec, None).unwrap_err();
        assert!(matches!(err, Error::Process { failure: ProcessFailure::NotFound, .. }));
    }

    #[test]
    fn output_captures_stdout_and_exit_code() {
        let spec = if cfg!(windows) {
            ProcessSpec::new("cmd").args(["/C", "echo hello& exit 3"]).hide_console()
        } else {
            ProcessSpec::new("sh").args(["-c", "echo hello; exit 3"])
        };
        let output = SystemRunner.output(&spec, Some(Duration::from_secs(20))).unwrap();
        assert_eq!(output.exit_code, Some(3));
        assert_eq!(output.stdout.trim(), "hello");
        assert!(!output.is_success());
    }

    #[test]
    fn stream_delivers_chunks_and_honours_cancel() {
        let spec = if cfg!(windows) {
            ProcessSpec::new("cmd").args(["/C", "echo one"]).hide_console()
        } else {
            ProcessSpec::new("sh").args(["-c", "echo one"])
        };
        let mut seen = String::new();
        let output = SystemRunner
            .stream(&spec, &CancelToken::new(), &mut |_, text| seen.push_str(text))
            .unwrap();
        assert_eq!(seen.trim(), "one");
        assert_eq!(output.exit_code, Some(0));

        let cancel = CancelToken::new();
        cancel.cancel();
        let err = SystemRunner.stream(&spec, &cancel, &mut |_, _| {}).unwrap_err();
        assert!(err.is_cancelled());
    }
}
