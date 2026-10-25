//! A scripted server for the tests that need the backend's end of a connection: a socket pair per
//! connection, so what the client does on a restart is exact, and a way to wait on a future.

#![allow(dead_code)]

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
