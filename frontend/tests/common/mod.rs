//! A scripted server for the tests that need the backend's end of a connection: a socket pair per
//! connection, so what the client does on a restart is exact, and a way to wait on a future.

#![allow(dead_code)]

pub mod qml;
pub mod window;

use std::collections::VecDeque;
use std::future::Future;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::pin::pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};
use std::time::{Duration, Instant};

use async_channel::Receiver;
use atmos_ui::client::{Connection, Notice, Transport};
use serde_json::Value;

struct Unpark(Thread);

impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// Runs a future to its end on this thread, and fails the test rather than wait forever.
pub fn block_on<T>(future: impl Future<Output = T>) -> T {
    let mut future = pin!(future);
    let waker = Waker::from(Arc::new(Unpark(thread::current())));
    let mut cx = Context::from_waker(&waker);
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(value) => return value,
            Poll::Pending => {
                let left = deadline.saturating_duration_since(Instant::now());
                assert!(!left.is_zero(), "nothing happened in 20 seconds");
                thread::park_timeout(left);
            }
        }
    }
}

pub fn next(notices: &Receiver<Notice>) -> Notice {
    block_on(async { notices.recv().await.expect("the client ended") })
}

/// Hands the client one prepared connection per `open`, then fails.
pub struct Scripted(Arc<Mutex<VecDeque<UnixStream>>>);

impl Transport for Scripted {
    fn open(&mut self) -> io::Result<Connection> {
        let stream = self
            .0
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no more servers"))?;
        let reader = stream.try_clone()?;
        Ok(Connection {
            writer: Box::new(stream),
            reader: Box::new(BufReader::new(reader)),
            close: Box::new(|| {}),
        })
    }
}

/// The server's end of a connection.
pub struct Server {
    lines: BufReader<UnixStream>,
    out: UnixStream,
}

impl Server {
    /// The next request the client sent.
    pub fn read(&mut self) -> Value {
        let mut line = String::new();
        self.lines.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    }

    /// A reply or an event, as one line.
    pub fn say(&mut self, value: Value) {
        writeln!(self.out, "{value}").unwrap();
    }
}

/// A transport that can open `connections` times, and the server's end of each.
pub fn script(connections: usize) -> (Scripted, Vec<Server>) {
    let queue = Arc::new(Mutex::new(VecDeque::new()));
    let mut servers = Vec::new();
    for _ in 0..connections {
        let (client, server) = UnixStream::pair().unwrap();
        queue.lock().unwrap().push_back(client);
        servers.push(Server {
            lines: BufReader::new(server.try_clone().unwrap()),
            out: server,
        });
    }
    (Scripted(queue), servers)
}

/// What a scripted daemon answers a request with.
pub enum Reply {
    /// `ok: true`, with this as the result.
    Ok(Value),
    /// `ok: false`, with this code and message.
    Err(&'static str, &'static str),
}

/// A server that reads every request, keeps it, and answers it, on a thread of its own, and can
/// push an event whenever the test says so.
pub struct Auto {
    seen: Arc<Mutex<Vec<Value>>>,
    out: Arc<Mutex<UnixStream>>,
}

impl Server {
    /// Answers each request by `reply` for as long as the client is connected.
    pub fn auto(self, reply: impl Fn(&Value) -> Reply + Send + 'static) -> Auto {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let out = Arc::new(Mutex::new(self.out.try_clone().unwrap()));
        let (kept, speak) = (seen.clone(), out.clone());
        let mut lines = self.lines;
        thread::spawn(move || {
            let mut line = String::new();
            loop {
                line.clear();
                if lines.read_line(&mut line).unwrap_or(0) == 0 {
                    return;
                }
                let Ok(request) = serde_json::from_str::<Value>(&line) else {
                    continue;
                };
                let envelope = match reply(&request) {
                    Reply::Ok(result) => {
                        serde_json::json!({"id": request["id"], "ok": true, "result": result})
                    }
                    Reply::Err(code, message) => serde_json::json!({
                        "id": request["id"], "ok": false, "result": null,
                        "error": {"code": code, "message": message, "context": null}}),
                };
                kept.lock().unwrap().push(request);
                let mut out = speak.lock().unwrap();
                let _ = writeln!(out, "{envelope}");
            }
        });
        Auto { seen, out }
    }
}

impl Auto {
    /// An event or a line the server pushes.
    pub fn say(&self, value: Value) {
        let mut out = self.out.lock().unwrap();
        writeln!(out, "{value}").unwrap();
    }

    /// The server goes away, as a `serve` that exits does: the client sees its end and starts
    /// the next.
    pub fn close(&self) {
        let _ = self.out.lock().unwrap().shutdown(std::net::Shutdown::Both);
    }

    /// The requests the client has made so far.
    pub fn requests(&self) -> Vec<Value> {
        self.seen.lock().unwrap().clone()
    }

    /// The requests of one op, or of a `write` of one name.
    pub fn of(&self, op: &str) -> Vec<Value> {
        self.requests()
            .into_iter()
            .filter(|r| r["op"] == op || (r["op"] == "write" && r["name"] == op))
            .collect()
    }
}
