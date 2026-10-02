//! The one way ratmos starts a child process. Output is captured, stdin is
//! closed unless the caller supplies bytes, and a caller that can bound the
//! wait does, so a hung `omarchy` or `systemctl` cannot hold a request open.

use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{Error, Kind, Result};

/// Long enough for a script that edits files and reloads a service, short
/// enough that a wedged one does not hold a request forever.
pub const COMMAND_TIMEOUT: Duration = Duration::from_secs(120);

pub struct Output {
    pub code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl Output {
    pub fn success(&self) -> bool {
        self.code == 0
    }

    pub fn stdout_text(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    pub fn stderr_text(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

pub struct Run {
    program: OsString,
    args: Vec<OsString>,
    envs: Vec<(OsString, OsString)>,
    input: Option<Vec<u8>>,
    timeout: Option<Duration>,
}

impl Run {
    pub fn new(program: impl AsRef<OsStr>) -> Self {
        Run {
            program: program.as_ref().to_os_string(),
            args: Vec::new(),
            envs: Vec::new(),
            input: None,
            timeout: None,
        }
    }

    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|a| a.as_ref().to_os_string()));
        self
    }

    pub fn env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.envs
            .push((key.as_ref().to_os_string(), value.as_ref().to_os_string()));
        self
    }

    pub fn input(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.input = Some(bytes.into());
        self
    }

    pub fn timeout(mut self, limit: Duration) -> Self {
        self.timeout = Some(limit);
        self
    }

    fn name(&self) -> String {
        self.program.to_string_lossy().into_owned()
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        for (key, value) in &self.envs {
            command.env(key, value);
        }
        command
    }

    /// Run to the end and return what it printed, whatever its exit code.
    pub fn output(self) -> Result<Output> {
        let name = self.name();
        let mut command = self.command();
        command
            .stdin(if self.input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .map_err(|err| Error::from(err).with_context(name.clone()))?;
        // Drain first, so a child that fills its pipes while we are still
        // writing its stdin cannot deadlock us.
        let stdout = drain(child.stdout.take());
        let stderr = drain(child.stderr.take());
        if let Some(bytes) = self.input {
            if let Some(mut stdin) = child.stdin.take() {
                // The child may exit before reading; its exit code says why.
                let _ = stdin.write_all(&bytes);
            }
        }
        let code = wait(&mut child, self.timeout, &name)?;
        Ok(Output {
            code,
            stdout: stdout.join().unwrap_or_default(),
            stderr: stderr.join().unwrap_or_default(),
        })
    }

    /// Like `output`, but a non-zero exit is an error carrying its stderr.
    pub fn checked(self) -> Result<Output> {
        let name = self.name();
        let output = self.output()?;
        if output.success() {
            return Ok(output);
        }
        let stderr = output.stderr_text();
        let detail = stderr.trim();
        let message = if detail.is_empty() {
            format!("exited {}", output.code)
        } else {
            format!("exited {}: {detail}", output.code)
        };
        Err(Error::command(message).with_context(name))
    }

    /// Start it and leave. Nothing is captured, and a thread reaps the child
    /// so a long-lived `serve` does not collect zombies.
    pub fn detach(self) -> Result<()> {
        let name = self.name();
        let mut child = self
            .command()
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| Error::from(err).with_context(name))?;
        thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    }
}

fn drain<R: Read + Send + 'static>(stream: Option<R>) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut stream) = stream {
            let _ = stream.read_to_end(&mut bytes);
        }
        bytes
    })
}

fn wait(child: &mut Child, timeout: Option<Duration>, name: &str) -> Result<i32> {
    let Some(limit) = timeout else {
        return Ok(child.wait()?.code().unwrap_or(1));
    };
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status.code().unwrap_or(1));
        }
        if started.elapsed() >= limit {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Error::new(
                Kind::Timeout,
                format!("did not finish in {}s", limit.as_secs()),
            )
            .with_context(name));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_both_streams_and_the_exit_code() {
        let out = Run::new("sh")
            .args(["-c", "echo out; echo err >&2; exit 3"])
            .output()
            .unwrap();
        assert_eq!(out.code, 3);
        assert_eq!(out.stdout_text().trim(), "out");
        assert_eq!(out.stderr_text().trim(), "err");
    }

    #[test]
    fn checked_turns_a_failure_into_an_error_with_stderr() {
        let err = Run::new("sh")
            .args(["-c", "echo nope >&2; exit 2"])
            .checked()
            .err()
            .unwrap();
        assert_eq!(err.kind, Kind::Command);
        assert!(err.to_string().contains("nope"), "{err}");
    }

    #[test]
    fn a_hung_child_times_out_and_is_killed() {
        let started = Instant::now();
        let err = Run::new("sleep")
            .arg("30")
            .timeout(Duration::from_millis(200))
            .output()
            .err()
            .unwrap();
        assert_eq!(err.kind, Kind::Timeout);
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn input_reaches_stdin() {
        let out = Run::new("cat").input("hello").output().unwrap();
        assert_eq!(out.stdout_text(), "hello");
    }

    #[test]
    fn a_missing_program_is_a_not_found_error() {
        let err = Run::new("ratmos-no-such-program").output().err().unwrap();
        assert_eq!(err.kind, Kind::NotFound);
    }
}
