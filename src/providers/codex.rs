use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdout, Command, Stdio},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;

use crate::model::{ProviderState, Window};

pub const APP_SERVER_COMMAND: &str = "codex";
pub const RATE_LIMIT_METHOD: &str = "account/rateLimits/read";
const RATE_LIMIT_RESPONSE_ID: u64 = 2;
const APP_SERVER_RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Deserialize)]
struct CodexJsonRpcResponse {
    result: Option<CodexResult>,
    error: Option<CodexJsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct CodexResult {
    #[serde(rename = "rateLimits")]
    rate_limits: CodexRateLimits,
}

#[derive(Debug, Deserialize)]
struct CodexRateLimits {
    primary: CodexWindow,
    secondary: CodexWindow,
}

#[derive(Debug, Deserialize)]
struct CodexWindow {
    #[serde(rename = "usedPercent")]
    used_percent: i64,
    #[serde(rename = "resetsAt")]
    resets_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct CodexJsonRpcLine {
    id: Option<u64>,
    error: Option<CodexJsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct CodexJsonRpcError {
    code: Option<i64>,
    message: String,
}

enum AppServerEvent {
    Line(String),
    Eof,
    ReadError(String),
}

pub fn fetch_state() -> Result<ProviderState> {
    let response = read_rate_limits_from_app_server()?;
    parse_rate_limits(&response)
}

pub fn parse_rate_limits(input: &str) -> Result<ProviderState> {
    let response: CodexJsonRpcResponse =
        serde_json::from_str(input).context("parse Codex rate limits response")?;
    if let Some(error) = response.error {
        return Err(codex_json_rpc_error(error));
    }

    let result = response
        .result
        .context("Codex rate limits response missing result")?;

    Ok(ProviderState::available(
        result.rate_limits.primary.into_window("5h")?,
        result.rate_limits.secondary.into_window("wk")?,
    ))
}

impl CodexWindow {
    fn into_window(self, label: &str) -> Result<Window> {
        let resets_at = self
            .resets_at
            .map(|epoch| {
                DateTime::<Utc>::from_timestamp(epoch, 0)
                    .ok_or_else(|| anyhow!("invalid Codex resetsAt epoch: {epoch}"))
            })
            .transpose()?;

        Ok(Window {
            label: label.to_string(),
            used_percentage: self.used_percent as f64,
            resets_at,
        })
    }
}

fn read_rate_limits_from_app_server() -> Result<String> {
    let mut child = Command::new(APP_SERVER_COMMAND)
        .arg("app-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("spawn codex app-server")?;

    let (result, reader) = read_rate_limits_from_child(&mut child);
    let terminate_result = terminate_child(&mut child);
    let join_result = match reader {
        Some(reader) => join_stdout_reader(reader),
        None => Ok(()),
    };

    let mut cleanup_errors = Vec::new();
    if let Err(error) = terminate_result {
        cleanup_errors.push(format!("terminate codex app-server: {error}"));
    }
    if let Err(error) = join_result {
        cleanup_errors.push(error.to_string());
    }

    match (result, cleanup_errors.is_empty()) {
        (Ok(response), true) => Ok(response),
        (Ok(_), false) => Err(anyhow!(cleanup_errors.join("; "))),
        (Err(error), true) => Err(error),
        (Err(error), false) => Err(anyhow!("{error}; {}", cleanup_errors.join("; "))),
    }
}

fn read_rate_limits_from_child(child: &mut Child) -> (Result<String>, Option<JoinHandle<()>>) {
    let stdout = match child
        .stdout
        .take()
        .context("capture codex app-server stdout")
    {
        Ok(stdout) => stdout,
        Err(error) => return (Err(error), None),
    };
    let (receiver, reader) = spawn_stdout_reader(stdout);

    let mut stdin = match child.stdin.take().context("open codex app-server stdin") {
        Ok(stdin) => stdin,
        Err(error) => return (Err(error), Some(reader)),
    };

    let response = send_initialize(&mut stdin)
        .and_then(|()| send_rate_limits_request(&mut stdin))
        .and_then(|()| wait_for_rate_limits_response(&receiver));
    drop(stdin);
    (response, Some(reader))
}

fn send_initialize(stdin: &mut impl Write) -> Result<()> {
    let message = json!({
        "id": 1,
        "method": "initialize",
        "params": {
            "clientInfo": {
                "name": "wabi",
                "version": env!("CARGO_PKG_VERSION"),
            },
        },
    });
    write_json_line(stdin, &message).context("send Codex app-server initialize request")
}

fn send_rate_limits_request(stdin: &mut impl Write) -> Result<()> {
    let message = json!({
        "id": RATE_LIMIT_RESPONSE_ID,
        "method": RATE_LIMIT_METHOD,
    });
    write_json_line(stdin, &message).context("send Codex rate limits request")
}

fn write_json_line(stdin: &mut impl Write, message: &serde_json::Value) -> Result<()> {
    serde_json::to_writer(&mut *stdin, message).context("serialize Codex app-server request")?;
    stdin
        .write_all(b"\n")
        .context("write Codex app-server request newline")?;
    stdin.flush().context("flush Codex app-server stdin")
}

fn spawn_stdout_reader(stdout: ChildStdout) -> (Receiver<AppServerEvent>, JoinHandle<()>) {
    let (sender, receiver) = mpsc::channel();
    let reader = thread::spawn(move || {
        let mut stdout = BufReader::new(stdout);
        let mut line = String::new();

        loop {
            line.clear();
            match stdout.read_line(&mut line) {
                Ok(0) => {
                    if sender.send(AppServerEvent::Eof).is_err() {
                        break;
                    }
                    break;
                }
                Ok(_) => {
                    let event = AppServerEvent::Line(line.trim_end().to_string());
                    if sender.send(event).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    if sender
                        .send(AppServerEvent::ReadError(error.to_string()))
                        .is_err()
                    {
                        break;
                    }
                    break;
                }
            }
        }
    });

    (receiver, reader)
}

fn wait_for_rate_limits_response(receiver: &Receiver<AppServerEvent>) -> Result<String> {
    loop {
        let event = receiver
            .recv_timeout(APP_SERVER_RESPONSE_TIMEOUT)
            .map_err(receive_timeout_error)?;

        match event {
            AppServerEvent::Line(line) => {
                if line.is_empty() {
                    continue;
                }

                let envelope: CodexJsonRpcLine = serde_json::from_str(&line)
                    .with_context(|| format!("parse Codex app-server JSON line: {line}"))?;
                if envelope.id != Some(RATE_LIMIT_RESPONSE_ID) {
                    continue;
                }
                if let Some(error) = envelope.error {
                    return Err(codex_json_rpc_error(error));
                }

                return Ok(line);
            }
            AppServerEvent::Eof => {
                return Err(anyhow!(
                    "codex app-server exited before {RATE_LIMIT_METHOD} response"
                ));
            }
            AppServerEvent::ReadError(error) => {
                return Err(anyhow!("read codex app-server stdout: {error}"));
            }
        }
    }
}

fn receive_timeout_error(error: RecvTimeoutError) -> anyhow::Error {
    match error {
        RecvTimeoutError::Timeout => anyhow!(
            "timed out waiting for codex app-server {RATE_LIMIT_METHOD} response after {}s",
            APP_SERVER_RESPONSE_TIMEOUT.as_secs()
        ),
        RecvTimeoutError::Disconnected => {
            anyhow!(
                "codex app-server stdout reader disconnected before {RATE_LIMIT_METHOD} response"
            )
        }
    }
}

fn terminate_child(child: &mut Child) -> Result<()> {
    if child
        .try_wait()
        .context("check codex app-server process status")?
        .is_none()
    {
        child.kill().context("kill codex app-server")?;
    }
    child.wait().context("wait for codex app-server to exit")?;
    Ok(())
}

fn join_stdout_reader(reader: JoinHandle<()>) -> Result<()> {
    reader
        .join()
        .map_err(|_| anyhow!("codex app-server stdout reader panicked"))
}

fn codex_json_rpc_error(error: CodexJsonRpcError) -> anyhow::Error {
    match error.code {
        Some(code) => anyhow!("Codex rate limits JSON-RPC error {code}: {}", error.message),
        None => anyhow!("Codex rate limits JSON-RPC error: {}", error.message),
    }
}
