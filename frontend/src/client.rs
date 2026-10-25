//! The one connection to ratmos: a single `ratmos serve` per window, as `services/Backend.qml`
//! keeps today (docs/adr/0008, 0010, 0012).
//!
//! A request returns a future that resolves exactly once, with the reply or with `unavailable`.
//! What the server pushes arrives on one channel as a [`Notice`]; the view awaits it and notifies,
//! so nothing polls. The way to the server is a [`Transport`]: only [`ServeProcess`] knows it is a
//! child process, and a packaged window that reaches the daemon another way replaces only that.

use std::collections::HashMap;
use std::ffi::OsString;
use std::future::Future;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use async_channel::{Receiver, Sender};
use serde_json::Value;

use crate::protocol::{self, Event, Failure, Incoming, Reply};

/// The state of the `serve` process, as Backend.qml's `link`. The daemon behind it reports
/// separately, with a `backend` event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    Starting,
    Up,
    /// The process is gone or could not start. It starts again in `retry_ms`.
    Down {
        failures: u32,
        retry_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Notice {
    Link(Link),
    Event(Event),
}

/// An open way to the server: lines out, lines in, and how to end it.
pub struct Connection {
    pub writer: Box<dyn Write + Send>,
    pub reader: Box<dyn BufRead + Send>,
    /// Ends the connection and reaps what is behind it. Called once the reader has ended.
    pub close: Box<dyn FnOnce() + Send>,
}

pub trait Transport: Send + 'static {
    fn open(&mut self) -> io::Result<Connection>;
}

/// `ratmos [--backend ID] [extra...] serve`, with its stderr passed on to ours.
pub struct ServeProcess {
    program: PathBuf,
    args: Vec<OsString>,
    env: Vec<(OsString, OsString)>,
}

impl ServeProcess {
    pub fn new(program: impl Into<PathBuf>, backend: &str, extra: &[&str]) -> Self {
        let mut args: Vec<OsString> = vec!["--backend".into(), backend.into()];
        args.extend(extra.iter().map(OsString::from));
        args.push("serve".into());
        Self {
            program: program.into(),
            args,
            env: Vec::new(),
        }
    }

    /// Sets a variable for the child only.
    pub fn env(mut self, key: &str, value: impl Into<OsString>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Where the QML window finds it (`bin/atmos`, `Backend.qml`): `ATMOS_BACKEND`, then the
    /// `bin/ratmos` of the Bundle this program is in (see [`ServeProcess::choose`]), then the app
    /// root's `bin/ratmos` and `backend/target/{release,debug}/ratmos`, then `ratmos` on PATH.
    /// The backend id is `ATMOS_BACKEND_ID`, `omarchy` by default.
    pub fn from_env() -> Self {
        let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
        Self::choose(
            var("ATMOS_BACKEND").map(PathBuf::from),
            var("ATMOS_ROOT").map(PathBuf::from),
            std::env::current_exe().ok().as_deref(),
            var("ATMOS_BACKEND_ID").and_then(|v| v.into_string().ok()),
        )
    }

    /// The `ratmos` to run. One named by `explicit` wins. Next is `bin/ratmos` beside this
    /// program's own executable, which is where the Bundle holds it (docs/adr/0014): that copy
    /// runs in the sandbox, so it cannot start a daemon and has no session to tell the daemon of.
    /// It is told so, and the daemon, which `oma` started from the real session, uses its own
    /// environment for what the window asks (`ATMOS_NO_AUTOSTART`, `ATMOS_SESSION=daemon`).
    pub fn choose(
        explicit: Option<PathBuf>,
        root: Option<PathBuf>,
        exe: Option<&Path>,
        backend_id: Option<String>,
    ) -> Self {
        let backend = backend_id.unwrap_or_else(|| "omarchy".into());
        if explicit.is_none()
            && let Some(bundled) = exe.and_then(bundled_beside)
        {
            return Self::new(bundled, &backend, &[])
                .env("ATMOS_NO_AUTOSTART", "1")
                .env("ATMOS_SESSION", "daemon");
        }
        Self::new(locate(explicit, root.as_deref()), &backend, &[])
    }
}

/// The `bin/ratmos` of the Bundle `exe` is the program of, when there is one.
fn bundled_beside(exe: &Path) -> Option<PathBuf> {
    let candidate = exe.parent()?.join("bin/ratmos");
    is_executable(&candidate).then_some(candidate)
}

/// The first of the candidates that is an executable file, else plain `ratmos` for PATH.
pub fn locate(explicit: Option<PathBuf>, root: Option<&Path>) -> PathBuf {
    let mut candidates: Vec<PathBuf> = explicit.into_iter().collect();
    if let Some(root) = root {
        candidates.push(root.join("bin/ratmos"));
        candidates.push(root.join("backend/target/release/ratmos"));
        candidates.push(root.join("backend/target/debug/ratmos"));
    }
    candidates
        .into_iter()
        .find(|path| is_executable(path))
        .unwrap_or_else(|| "ratmos".into())
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

impl Transport for ServeProcess {
    fn open(&mut self) -> io::Result<Connection> {
        let mut child = Command::new(&self.program)
            .args(&self.args)
            .envs(self.env.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.take().expect("stdin was piped");
        let stdout = child.stdout.take().expect("stdout was piped");
        let stderr = child.stderr.take().expect("stderr was piped");
        // Without this the backend's stderr, and the reason for a failed read, is lost.
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                eprintln!("ratmos: {line}");
            }
        });
        Ok(Connection {
            writer: Box::new(stdin),
            reader: Box::new(BufReader::new(stdout)),
            close: Box::new(move || reap(child)),
        })
    }
}

fn reap(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

struct State {
    writer: Option<Box<dyn Write + Send>>,
    pending: HashMap<u64, Sender<Reply>>,
    /// What the window holds, sent again whenever the server comes back.
    topics: Vec<(String, u64)>,
}

struct Inner {
    state: Mutex<State>,
    next: AtomicU64,
    notices: Sender<Notice>,
}

impl Inner {
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::Relaxed)
    }

    fn notify(&self, notice: Notice) {
        // The channel is unbounded and the view may be gone: a lost notice is not an error.
        let _ = self.notices.try_send(notice);
    }

    fn write(&self, state: &mut State, line: &str) -> io::Result<()> {
        let writer = state
            .writer
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "no server"))?;
        writer.write_all(line.as_bytes())?;
        writer.flush()
    }

    /// A server is there: say what the window holds.
    fn attach(&self, writer: Box<dyn Write + Send>) {
        let mut state = self.state();
        state.writer = Some(writer);
        if !state.topics.is_empty() {
            let topics: Vec<(&str, u64)> =
                state.topics.iter().map(|(k, v)| (k.as_str(), *v)).collect();
            let line = protocol::encode(self.id(), &protocol::sub_set(&topics));
            let _ = self.write(&mut state, &line);
        }
        drop(state);
        self.notify(Notice::Link(Link::Up));
    }

    /// The server is gone: nothing waiting for it will be answered.
    fn detach(&self, why: &str) {
        let mut state = self.state();
        state.writer = None;
        let pending = std::mem::take(&mut state.pending);
        drop(state);
        for (_, reply) in pending {
            let _ = reply.try_send(Err(Failure::unavailable(why)));
        }
    }

    fn route(&self, line: &str) {
        match protocol::decode(line) {
            Some(Incoming::Reply {
                id: Some(id),
                reply,
            }) => {
                if let Some(waiting) = self.state().pending.remove(&id) {
                    let _ = waiting.try_send(reply);
                }
            }
            Some(Incoming::Reply { id: None, reply }) => {
                eprintln!("ratmos: a reply without an id: {reply:?}");
            }
            Some(Incoming::Event(event)) => self.notify(Notice::Event(event)),
            None => eprintln!("ratmos: not a protocol line: {line}"),
        }
    }

    fn read(&self, reader: Box<dyn BufRead + Send>) {
        for line in reader.lines().map_while(Result::ok) {
            self.route(&line);
        }
    }
}

/// Held by every clone of a [`Client`] and not by the supervisor, so it is dropped when the last
/// clone is: closing stdin is how the server learns its window is gone, and dropping `stop`
/// ends the supervisor's wait and the supervisor with it.
struct Guard {
    inner: Arc<Inner>,
    _stop: mpsc::Sender<()>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        self.inner.state().writer = None;
    }
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
    _guard: Arc<Guard>,
}

impl Client {
    /// Opens the way to the server now, so a request made at once reaches it, and keeps it open:
    /// when the server stops, it starts again after one, two, four seconds and so on up to thirty
    /// (a server that lived five seconds restarts the count). The receiver gets the link's state
    /// and everything the server pushes.
    pub fn start(mut transport: impl Transport) -> (Self, Receiver<Notice>) {
        let (notices, receiver) = async_channel::unbounded();
        let (stop, stopped) = mpsc::channel();
        let inner = Arc::new(Inner {
            state: Mutex::new(State {
                writer: None,
                pending: HashMap::new(),
                topics: Vec::new(),
            }),
            next: AtomicU64::new(1),
            notices,
        });
        inner.notify(Notice::Link(Link::Starting));
        let first = try_open(&inner, &mut transport, &stopped);
        let supervised = Arc::clone(&inner);
        thread::spawn(move || supervise(supervised, transport, first, stopped));
        let guard = Arc::new(Guard {
            inner: Arc::clone(&inner),
            _stop: stop,
        });
        (
            Self {
                inner,
                _guard: guard,
            },
            receiver,
        )
    }

    /// Sends a request body. The future resolves once, with the reply or with `unavailable`
    /// when the server is not there, stops, or cannot be written to.
    pub fn request(&self, body: &Value) -> impl Future<Output = Reply> + use<> {
        let (reply, waiting) = async_channel::bounded(1);
        let inner = &self.inner;
        let id = inner.id();
        let line = protocol::encode(id, body);
        let sent = {
            let mut state = inner.state();
            state.pending.insert(id, reply.clone());
            inner.write(&mut state, &line).inspect_err(|_| {
                state.pending.remove(&id);
            })
        };
        if let Err(error) = sent {
            let _ = reply.try_send(Err(Failure::unavailable(&format!(
                "the Atmos backend is not running ({error})"
            ))));
        }
        async move {
            waiting
                .recv()
                .await
                .unwrap_or_else(|_| Err(Failure::unavailable("the Atmos backend stopped")))
        }
    }

    /// Holds these topics (name and interval in milliseconds, 0 for "when it changes"),
    /// replacing the last set. They are held again after a restart. The reply to the server is
    /// the future's, so a caller can see a topic the backend refused.
    pub fn hold(&self, topics: &[(&str, u64)]) -> impl Future<Output = Reply> + use<> {
        self.inner.state().topics = topics.iter().map(|(k, v)| ((*k).into(), *v)).collect();
        self.request(&protocol::sub_set(topics))
    }
}

/// A connection the client is writing to, and what is left to do with it.
struct Attached {
    reader: Box<dyn BufRead + Send>,
    close: Box<dyn FnOnce() + Send>,
    started: Instant,
}

/// Opens the way to the server and starts writing to it, or says why it could not and returns
/// `None`. A window that went while the server was starting gets nothing either.
fn try_open(
    inner: &Inner,
    transport: &mut impl Transport,
    stopped: &mpsc::Receiver<()>,
) -> Option<Attached> {
    let connection = match transport.open() {
        Ok(connection) => connection,
        Err(error) => {
            eprintln!("ratmos: cannot start: {error}");
            return None;
        }
    };
    if matches!(stopped.try_recv(), Err(mpsc::TryRecvError::Disconnected)) {
        drop(connection.writer);
        (connection.close)();
        return None;
    }
    inner.attach(connection.writer);
    Some(Attached {
        reader: connection.reader,
        close: connection.close,
        started: Instant::now(),
    })
}

/// `first` is the attempt `start` made, so a server that could not start waits out its delay
/// like one that stopped.
fn supervise(
    inner: Arc<Inner>,
    mut transport: impl Transport,
    first: Option<Attached>,
    stopped: mpsc::Receiver<()>,
) {
    let mut failures: u32 = 0;
    let mut attempt = Some(first);
    loop {
        let opened = match attempt.take() {
            Some(done) => done,
            None => try_open(&inner, &mut transport, &stopped),
        };
        let Some(Attached {
            reader,
            close,
            started,
        }) = opened
        else {
            failures += 1;
            if !wait(&inner, failures, &stopped) {
                return;
            }
            continue;
        };
        inner.read(reader);
        inner.detach("the Atmos backend stopped");
        close();
        failures = if started.elapsed() >= Duration::from_millis(protocol::STABLE_MS) {
            1
        } else {
            failures + 1
        };
        if !wait(&inner, failures, &stopped) {
            return;
        }
    }
}

/// Says the link is down and waits out the restart delay. False when the client was dropped.
fn wait(inner: &Inner, failures: u32, stopped: &mpsc::Receiver<()>) -> bool {
    let retry_ms = protocol::restart_delay_ms(failures);
    inner.notify(Notice::Link(Link::Down { failures, retry_ms }));
    match stopped.recv_timeout(Duration::from_millis(retry_ms)) {
        Err(mpsc::RecvTimeoutError::Timeout) => {
            inner.notify(Notice::Link(Link::Starting));
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn locate_prefers_an_explicit_executable_and_falls_back_to_path() {
        assert_eq!(locate(None, None), PathBuf::from("ratmos"));
        assert_eq!(
            locate(
                Some("/definitely/not/here".into()),
                Some(Path::new("/nope"))
            ),
            PathBuf::from("ratmos")
        );
        let sh = Path::new("/bin/sh");
        if is_executable(sh) {
            assert_eq!(locate(Some(sh.into()), None), sh);
        }
    }

    #[test]
    fn the_ratmos_in_the_bundle_runs_with_the_daemons_session_and_never_starts_a_daemon() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("atmos-ui-bundle-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        let ratmos = dir.join("bin/ratmos");
        std::fs::write(&ratmos, "").unwrap();
        std::fs::set_permissions(&ratmos, std::fs::Permissions::from_mode(0o755)).unwrap();
        let exe = dir.join("atmos-ui");

        let env = |p: &ServeProcess| -> Vec<(String, String)> {
            p.env
                .iter()
                .map(|(k, v)| (k.to_string_lossy().into(), v.to_string_lossy().into()))
                .collect()
        };
        let bundled = ServeProcess::choose(None, None, Some(&exe), None);
        assert_eq!(bundled.program, ratmos);
        assert_eq!(
            env(&bundled),
            [
                ("ATMOS_NO_AUTOSTART".to_owned(), "1".to_owned()),
                ("ATMOS_SESSION".to_owned(), "daemon".to_owned())
            ]
        );

        // `ATMOS_BACKEND` still wins, and then the window is an ordinary one.
        let named = ServeProcess::choose(Some("/x/ratmos".into()), None, Some(&exe), None);
        assert!(env(&named).is_empty());
        // A program with no `bin/ratmos` beside it (a build in the source tree) is ordinary too.
        let loose = ServeProcess::choose(None, None, Some(&dir.join("elsewhere/atmos-ui")), None);
        assert!(env(&loose).is_empty());
        assert_eq!(loose.program, PathBuf::from("ratmos"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn serve_runs_after_the_backend_and_root_flags() {
        let p = ServeProcess::new("ratmos", "plain", &["--root", "/x"]);
        let args: Vec<_> = p
            .args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, ["--backend", "plain", "--root", "/x", "serve"]);
    }

    #[test]
    fn a_line_reader_over_a_cursor_routes_a_reply_once() {
        let (notices, _rx) = async_channel::unbounded();
        let inner = Inner {
            state: Mutex::new(State {
                writer: None,
                pending: HashMap::new(),
                topics: Vec::new(),
            }),
            next: AtomicU64::new(1),
            notices,
        };
        let (tx, rx) = async_channel::bounded(1);
        inner.state().pending.insert(4, tx);
        inner.read(Box::new(Cursor::new(
            "{\"id\":4,\"ok\":true,\"result\":1}\n{\"id\":4,\"ok\":true,\"result\":2}\n",
        )));
        assert_eq!(rx.try_recv().unwrap(), Ok(Value::from(1)));
        assert!(
            rx.try_recv().is_err(),
            "the second reply found nobody waiting"
        );
    }
}
