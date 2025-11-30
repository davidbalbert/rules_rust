//! Tests for persistent worker mode in process_wrapper.
//!
//! This tests the JSON worker protocol implementation.

#[cfg(test)]
mod test {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};

    use runfiles::Runfiles;

    /// Helper to run process_wrapper in worker mode and send/receive messages.
    fn run_worker_test<F>(test_fn: F)
    where
        F: FnOnce(&mut std::process::ChildStdin, &mut BufReader<std::process::ChildStdout>),
    {
        let r = Runfiles::create().unwrap();
        let process_wrapper =
            runfiles::rlocation!(r, env!("PROCESS_WRAPPER_RLOCATIONPATH")).unwrap();
        let _fake_rustc = runfiles::rlocation!(r, env!("FAKE_RUSTC_RLOCATIONPATH")).unwrap();

        let mut child = Command::new(&process_wrapper)
            .arg("--persistent_worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start process_wrapper in worker mode");

        let stdin = child.stdin.as_mut().expect("Failed to get stdin");
        let stdout = child.stdout.take().expect("Failed to get stdout");
        let mut reader = BufReader::new(stdout);

        test_fn(stdin, &mut reader);

        // Close stdin to signal EOF
        drop(child.stdin.take());

        // Wait for the worker to exit
        let status = child.wait().expect("Failed to wait for worker");
        assert!(
            status.success(),
            "Worker exited with non-zero status: {:?}",
            status
        );
    }

    #[test]
    fn test_worker_starts_and_exits_on_eof() {
        run_worker_test(|_stdin, _reader| {
            // Just test that worker starts and exits cleanly on EOF
        });
    }

    #[test]
    fn test_worker_responds_to_empty_request() {
        run_worker_test(|stdin, reader| {
            // Send a minimal work request with empty arguments
            let request = r#"{"arguments": []}"#;
            writeln!(stdin, "{}", request).expect("Failed to write request");
            stdin.flush().expect("Failed to flush");

            // Read the response
            let mut response = String::new();
            reader
                .read_line(&mut response)
                .expect("Failed to read response");

            // Parse response - should have exitCode field
            assert!(
                response.contains("\"exitCode\""),
                "Response should contain exitCode: {}",
                response
            );
        });
    }

    #[test]
    fn test_worker_handles_multiple_requests() {
        run_worker_test(|stdin, reader| {
            // Send multiple requests
            for i in 0..3 {
                let request = r#"{"arguments": []}"#;
                writeln!(stdin, "{}", request).expect("Failed to write request");
                stdin.flush().expect("Failed to flush");

                let mut response = String::new();
                reader
                    .read_line(&mut response)
                    .expect("Failed to read response");

                assert!(
                    response.contains("\"exitCode\""),
                    "Request {} response should contain exitCode: {}",
                    i,
                    response
                );
            }
        });
    }
}
