// Copyright 2020 The Bazel Authors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Bazel persistent worker protocol implementation for process_wrapper.
//!
//! This module implements the JSON-based persistent worker protocol that allows
//! the process_wrapper to stay alive between compilations, enabling rustc's
//! incremental compilation cache to persist.
//!
//! See: https://bazel.build/remote/persistent

use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Command;

use tinyjson::JsonValue;

use crate::options::options_from_args;
use crate::{CompileResult, ProcessWrapperError};

/// State maintained across worker requests.
pub(crate) struct WorkerState {
    /// Root directory for incremental compilation caches.
    pub(crate) incremental_root: PathBuf,
    /// Cached rustc version string (obtained once at worker startup).
    pub(crate) rustc_version: Option<String>,
    /// Number of requests handled by this worker instance.
    pub(crate) requests_handled: u64,
    /// Maximum requests before worker should exit.
    max_requests: u64,
}

impl WorkerState {
    pub(crate) fn new() -> Result<Self, ProcessWrapperError> {
        let cwd = std::env::current_dir()
            .map_err(|e| ProcessWrapperError(format!("failed to get current directory: {}", e)))?;
        let incremental_root = cwd.join("rust-incremental");
        std::fs::create_dir_all(&incremental_root).map_err(|e| {
            ProcessWrapperError(format!(
                "failed to create incremental cache directory {:?}: {}",
                incremental_root, e
            ))
        })?;

        let max_requests = std::env::var("RULES_RUST_MAX_REQUESTS_PER_WORKER")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(1000);

        Ok(Self {
            incremental_root,
            rustc_version: None,
            requests_handled: 0,
            max_requests,
        })
    }

    /// Returns true if the worker should exit after completing the current request.
    pub(crate) fn should_exit(&self) -> bool {
        self.requests_handled >= self.max_requests
    }

    /// Lazily obtains and caches the rustc version.
    pub(crate) fn get_rustc_version(&mut self, rustc_path: &str) -> Option<&str> {
        if self.rustc_version.is_none() {
            self.rustc_version = get_rustc_version(rustc_path);
        }
        self.rustc_version.as_deref()
    }
}

/// Gets the rustc version by running `rustc --version`.
fn get_rustc_version(rustc_path: &str) -> Option<String> {
    Command::new(rustc_path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
}

/// Parses a JSON WorkRequest from a line of input.
fn parse_work_request(line: &str) -> Result<WorkRequest, ProcessWrapperError> {
    let parsed: JsonValue = line
        .parse()
        .map_err(|_| ProcessWrapperError("failed to parse WorkRequest JSON".to_string()))?;

    let (arguments, request_id) = match &parsed {
        JsonValue::Object(map) => {
            let args = match map.get("arguments") {
                Some(JsonValue::Array(args)) => args
                    .iter()
                    .filter_map(|v| {
                        if let JsonValue::String(s) = v {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
                _ => Vec::new(),
            };
            let req_id = match map.get("requestId") {
                Some(JsonValue::Number(n)) => Some(*n as i32),
                _ => None,
            };
            (args, req_id)
        }
        _ => {
            return Err(ProcessWrapperError(
                "WorkRequest is not a JSON object".to_string(),
            ))
        }
    };

    Ok(WorkRequest {
        arguments,
        request_id,
    })
}

/// Serializes a WorkResponse to JSON.
fn serialize_work_response(response: &WorkResponse) -> Result<String, ProcessWrapperError> {
    let mut map = HashMap::new();
    map.insert(
        "exitCode".to_string(),
        JsonValue::Number(response.exit_code as f64),
    );
    map.insert(
        "output".to_string(),
        JsonValue::String(response.output.clone()),
    );
    if let Some(id) = response.request_id {
        map.insert("requestId".to_string(), JsonValue::Number(id as f64));
    }

    JsonValue::Object(map)
        .stringify()
        .map_err(|_| ProcessWrapperError("failed to serialize WorkResponse".to_string()))
}

/// A Bazel WorkRequest containing arguments for one compilation.
struct WorkRequest {
    arguments: Vec<String>,
    request_id: Option<i32>,
}

/// A Bazel WorkResponse containing the result of one compilation.
struct WorkResponse {
    exit_code: i32,
    output: String,
    request_id: Option<i32>,
}

/// Runs the persistent worker loop, reading WorkRequests from stdin and
/// writing WorkResponses to stdout.
pub(crate) fn run_worker_loop(
    startup_args: Vec<String>,
    run_compile: impl Fn(crate::options::Options) -> Result<CompileResult, ProcessWrapperError>,
) -> Result<(), ProcessWrapperError> {
    // Filter out --persistent_worker from startup args (it's consumed by us, not rustc)
    let _filtered_startup_args: Vec<String> = startup_args
        .into_iter()
        .filter(|a| a != "--persistent_worker")
        .collect();

    let mut state = WorkerState::new()?;

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdin_lock = stdin.lock();
    let mut stdout_lock = stdout.lock();

    loop {
        let mut line = String::new();
        let read = stdin_lock
            .read_line(&mut line)
            .map_err(|e| ProcessWrapperError(format!("error reading worker request: {}", e)))?;

        if read == 0 {
            // EOF - Bazel is shutting us down
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request_id = parse_work_request(trimmed)
            .ok()
            .and_then(|r| r.request_id);

        let response = match handle_request(&mut state, trimmed, &run_compile) {
            Ok(r) => r,
            Err(e) => WorkResponse {
                exit_code: 1,
                output: format!("process_wrapper error: {}", e),
                request_id,
            },
        };

        let json = serialize_work_response(&response)?;
        writeln!(stdout_lock, "{}", json)
            .map_err(|e| ProcessWrapperError(format!("error writing worker response: {}", e)))?;
        stdout_lock.flush().ok();

        state.requests_handled += 1;

        if state.should_exit() {
            break;
        }
    }

    Ok(())
}

/// Handles a single WorkRequest.
fn handle_request(
    state: &mut WorkerState,
    request_json: &str,
    run_compile: &impl Fn(crate::options::Options) -> Result<CompileResult, ProcessWrapperError>,
) -> Result<WorkResponse, ProcessWrapperError> {
    let request = parse_work_request(request_json)?;

    // Build the full argument list as if process_wrapper were invoked directly.
    // The first element should be the program name.
    let full_args: Vec<String> = std::iter::once("process_wrapper".to_string())
        .chain(request.arguments)
        .collect();

    let mut opts = options_from_args(full_args)
        .map_err(|e| ProcessWrapperError(format!("failed to parse options: {}", e)))?;

    // Inject incremental compilation flag
    inject_incremental_flag(state, &mut opts)?;

    // Run the actual compilation
    let result = run_compile(opts)?;

    Ok(WorkResponse {
        exit_code: result.exit_code,
        output: result.output,
        request_id: request.request_id,
    })
}

/// Injects the `-C incremental=<path>` flag into the rustc arguments if not already present.
fn inject_incremental_flag(
    state: &mut WorkerState,
    opts: &mut crate::options::Options,
) -> Result<(), ProcessWrapperError> {
    // Check if user already passed -C incremental
    let has_incremental = opts
        .child_arguments
        .iter()
        .any(|arg| arg.starts_with("-Cincremental=") || arg.starts_with("-C incremental="));

    if has_incremental {
        return Ok(());
    }

    // Compute cache key based on crate identity
    let crate_key = compute_crate_key(state, opts);
    let incremental_dir = state.incremental_root.join(&crate_key);

    std::fs::create_dir_all(&incremental_dir).map_err(|e| {
        ProcessWrapperError(format!(
            "failed to create incremental cache directory {:?}: {}",
            incremental_dir, e
        ))
    })?;

    let dir_str = incremental_dir
        .to_str()
        .ok_or_else(|| ProcessWrapperError("incremental dir path not valid UTF-8".to_string()))?;

    opts.child_arguments
        .push(format!("-Cincremental={}", dir_str));

    Ok(())
}

/// Computes a cache key for the current crate based on its identity.
/// This key is used to namespace incremental compilation caches.
fn compute_crate_key(state: &mut WorkerState, opts: &crate::options::Options) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Hash the rustc executable path
    opts.executable.hash(&mut hasher);

    // Get and hash rustc version
    if let Some(version) = state.get_rustc_version(&opts.executable) {
        version.hash(&mut hasher);
    }

    // Extract crate name and other identity from arguments
    let mut crate_name = None;
    let mut crate_type = None;
    let mut args_iter = opts.child_arguments.iter();

    while let Some(arg) = args_iter.next() {
        // Hash all arguments except -C incremental (which we're injecting)
        if !arg.starts_with("-Cincremental=") && !arg.starts_with("-C incremental=") {
            arg.hash(&mut hasher);
        }

        // Extract crate name for a more readable directory name
        if arg == "--crate-name" {
            if let Some(name) = args_iter.next() {
                crate_name = Some(name.clone());
                name.hash(&mut hasher);
            }
        } else if arg.starts_with("--crate-name=") {
            crate_name = Some(arg.strip_prefix("--crate-name=").unwrap().to_string());
        }

        // Extract crate type
        if arg == "--crate-type" {
            if let Some(t) = args_iter.next() {
                crate_type = Some(t.clone());
            }
        } else if arg.starts_with("--crate-type=") {
            crate_type = Some(arg.strip_prefix("--crate-type=").unwrap().to_string());
        }
    }

    let hash = format!("{:016x}", hasher.finish());
    let name = crate_name.unwrap_or_else(|| "unknown".to_string());
    let type_suffix = crate_type.map(|t| format!("-{}", t)).unwrap_or_default();

    format!("{}{}-{}", name, type_suffix, hash)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_work_request() {
        let json = r#"{"arguments": ["--flag", "value", "file.rs"]}"#;
        let request = parse_work_request(json).unwrap();
        assert_eq!(
            request.arguments,
            vec![
                "--flag".to_string(),
                "value".to_string(),
                "file.rs".to_string()
            ]
        );
        assert_eq!(request.request_id, None);
    }

    #[test]
    fn test_parse_work_request_with_request_id() {
        let json = r#"{"arguments": ["--flag"], "requestId": 42}"#;
        let request = parse_work_request(json).unwrap();
        assert_eq!(request.arguments, vec!["--flag".to_string()]);
        assert_eq!(request.request_id, Some(42));
    }

    #[test]
    fn test_parse_work_request_with_request_id_zero() {
        let json = r#"{"arguments": [], "requestId": 0}"#;
        let request = parse_work_request(json).unwrap();
        assert!(request.arguments.is_empty());
        assert_eq!(request.request_id, Some(0));
    }

    #[test]
    fn test_parse_work_request_empty_args() {
        let json = r#"{"arguments": []}"#;
        let request = parse_work_request(json).unwrap();
        assert!(request.arguments.is_empty());
    }

    #[test]
    fn test_serialize_work_response() {
        let response = WorkResponse {
            exit_code: 0,
            output: "success".to_string(),
            request_id: None,
        };
        let json = serialize_work_response(&response).unwrap();
        assert!(json.contains("\"exitCode\""));
        assert!(json.contains("\"output\""));
        assert!(!json.contains("\"requestId\""));
    }

    #[test]
    fn test_serialize_work_response_with_request_id() {
        let response = WorkResponse {
            exit_code: 0,
            output: "".to_string(),
            request_id: Some(42),
        };
        let json = serialize_work_response(&response).unwrap();
        assert!(json.contains("\"exitCode\""));
        assert!(json.contains("\"requestId\""));
    }

    #[test]
    fn test_request_id_echo() {
        let json = r#"{"arguments": ["--flag"], "requestId": 0}"#;
        let request = parse_work_request(json).unwrap();
        let response = WorkResponse {
            exit_code: 0,
            output: "".to_string(),
            request_id: request.request_id,
        };
        let response_json = serialize_work_response(&response).unwrap();
        assert!(response_json.contains("\"requestId\""));
    }

    #[test]
    fn test_worker_state_max_requests() {
        // Test default behavior
        std::env::remove_var("RULES_RUST_MAX_REQUESTS_PER_WORKER");
        let state = WorkerState {
            incremental_root: PathBuf::from("/tmp"),
            rustc_version: None,
            requests_handled: 0,
            max_requests: 1000,
        };
        assert!(!state.should_exit());

        let state = WorkerState {
            incremental_root: PathBuf::from("/tmp"),
            rustc_version: None,
            requests_handled: 1000,
            max_requests: 1000,
        };
        assert!(state.should_exit());
    }
}
