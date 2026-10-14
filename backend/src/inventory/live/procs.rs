//! User-visible processes from /proc/<pid>/{stat,status,cmdline,io,fd}.

use super::{is_digits, Sys};
use serde_json::{json, Value};

const PF_KTHREAD: i64 = 0x0020_0000;

struct Stat {
    comm: String,
    flags: i64,
    ticks: i64,
    state: String,
    ppid: i64,
    nice: i64,
    threads: i64,
}

fn parse_stat(text: &str) -> Option<Stat> {
    let start = text.find('(')?;
    let end = text.rfind(')')?;
    if end <= start {
        return None;
    }
    let rest: Vec<&str> = text[end + 1..].split_whitespace().collect();
    // state, ppid, flags, utime, stime, nice, threads in /proc/pid/stat.
    if rest.len() < 13 {
        return None;
    }
    let int = |i: usize| rest[i].parse::<i64>().ok();
    let nice = if rest.len() > 16 { int(16)? } else { 0 };
    let threads = if rest.len() > 17 { int(17)? } else { 1 };
    Some(Stat {
        comm: text[start + 1..end].to_string(),
        flags: int(6)?,
        ticks: int(11)? + int(12)?,
        state: rest[0].chars().take(1).collect(),
        ppid: int(1)?,
        nice,
        threads: threads.max(0),
    })
}

fn parse_status(text: &str) -> (Option<i64>, Option<i64>) {
    let (mut uid, mut rss) = (None, None);
    for line in text.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if line.starts_with("Uid:") {
            uid = cols.get(2).and_then(|c| c.parse().ok());
        } else if line.starts_with("VmRSS:") {
            rss = cols.get(1).and_then(|c| c.parse().ok());
        }
    }
    (uid, rss)
}

fn read_io(sys: &Sys, pid: &str) -> (Option<i64>, Option<i64>) {
    let (mut read, mut write) = (None, None);
    for line in sys.text_limit(&format!("proc/{pid}/io"), 4096).lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        let value = cols
            .get(1)
            .filter(|c| is_digits(c))
            .and_then(|c| c.parse().ok());
        if line.starts_with("read_bytes:") && cols.len() >= 2 {
            read = value;
        } else if line.starts_with("write_bytes:") && cols.len() >= 2 {
            write = value;
        }
    }
    (read, write)
}

pub fn processes(sys: &Sys, want_uid: i64) -> Vec<Value> {
    let mut rows = Vec::new();
    let Ok(entries) = std::fs::read_dir(sys.path("proc")) else {
        return rows;
    };
    for entry in entries.flatten() {
        let pid_s = entry.file_name().to_string_lossy().into_owned();
        if !is_digits(&pid_s) {
            continue;
        }
        let Some(stat) = parse_stat(&sys.text(&format!("proc/{pid_s}/stat"))) else {
            continue;
        };
        let raw = sys.text_limit(&format!("proc/{pid_s}/cmdline"), 4096);
        let cmdline = raw
            .split('\0')
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let (uid, rss) = parse_status(&sys.text(&format!("proc/{pid_s}/status")));
        let (Some(uid), Ok(pid)) = (uid, pid_s.parse::<i64>()) else {
            continue;
        };
        let (read_b, write_b) = read_io(sys, &pid_s);
        let fds = std::fs::read_dir(sys.path(&format!("proc/{pid_s}/fd")))
            .ok()
            .map(|d| d.count());
        rows.push(json!({
            "pid": pid,
            "ppid": stat.ppid,
            "comm": stat.comm,
            "cmdline": cmdline,
            "uid": uid,
            "rssKb": rss,
            "ticks": stat.ticks,
            "state": stat.state,
            "threads": stat.threads,
            "nice": stat.nice,
            "kthread": stat.flags & PF_KTHREAD != 0,
            "readBytes": read_b,
            "writeBytes": write_b,
            "fds": fds,
            "mine": uid == want_uid,
        }));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_lines_with_odd_command_names_parse() {
        let line = "4242 (tmux: server (x)) S 1 4242 4242 0 -1 4194560 100 0 0 0 7 3 0 0 20 0 4 0 99 100 200";
        let stat = parse_stat(line).unwrap();
        assert_eq!(stat.comm, "tmux: server (x)");
        assert_eq!(
            (stat.ppid, stat.ticks, stat.nice, stat.threads),
            (1, 10, 0, 4)
        );
        assert_eq!(stat.state, "S");
        assert!(parse_stat("1 (x) S 1").is_none());
    }

    #[test]
    fn status_gives_the_effective_uid_and_rss() {
        let (uid, rss) = parse_status("Name:\tx\nUid:\t1000\t1001\t1000\t1000\nVmRSS:\t  512 kB\n");
        assert_eq!((uid, rss), (Some(1001), Some(512)));
    }
}
