//! TCP sockets from /proc/net/tcp{,6}, and which process holds each listener.

use super::{is_digits, Sys};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_ROWS: usize = 240;
const LISTEN: &str = "LISTEN";
/// How long a listener-to-pid answer is reused while the set of listeners is unchanged.
const OWNER_TTL: Duration = Duration::from_secs(30);

struct Cached {
    want: Vec<String>,
    at: Instant,
    found: HashMap<String, i64>,
}

/// Walking every `fd` directory costs tens of milliseconds on a busy desktop,
/// and a listening socket keeps its holder, so `serve` reuses the last walk.
static CACHE: Mutex<BTreeMap<PathBuf, Cached>> = Mutex::new(BTreeMap::new());

fn tcp_lines(sys: &Sys) -> Vec<String> {
    ["proc/net/tcp", "proc/net/tcp6"]
        .iter()
        .flat_map(|f| {
            sys.text(f)
                .lines()
                .skip(1)
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn read_tcp(sys: &Sys) -> Value {
    let (mut established, mut listen, mut time_wait, mut close_wait, mut total) = (0, 0, 0, 0, 0);
    for line in tcp_lines(sys) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 4 {
            continue;
        }
        total += 1;
        match cols[3].to_uppercase().as_str() {
            "01" => established += 1,
            "0A" => listen += 1,
            "06" => time_wait += 1,
            "08" => close_wait += 1,
            _ => {}
        }
    }
    json!({
        "established": established, "listen": listen, "timeWait": time_wait,
        "closeWait": close_wait, "total": total,
    })
}

struct Socket {
    inode: String,
    state: &'static str,
    addr: Option<String>,
    port: Option<u16>,
    uid: Option<i64>,
}

fn state_name(code: &str) -> Option<&'static str> {
    Some(match code.to_uppercase().as_str() {
        "01" => "ESTABLISHED",
        "02" => "SYN_SENT",
        "03" => "SYN_RECV",
        "04" => "FIN_WAIT1",
        "05" => "FIN_WAIT2",
        "06" => "TIME_WAIT",
        "07" => "CLOSE",
        "08" => "CLOSE_WAIT",
        "09" => "LAST_ACK",
        "0A" => LISTEN,
        "0B" => "CLOSING",
        _ => return None,
    })
}

fn hex_u32(text: &str) -> Option<u32> {
    u32::from_str_radix(text, 16).ok()
}

/// The kernel prints each 32-bit word of the address in host order, so on the
/// little-endian machines Omarchy runs on, `to_le_bytes` gives network order.
fn decode_addr(hex: &str) -> Option<String> {
    if !hex.is_ascii() {
        return None;
    }
    match hex.len() {
        8 => Some(Ipv4Addr::from(hex_u32(hex)?.to_le_bytes()).to_string()),
        32 => {
            let mut bytes = [0u8; 16];
            for (i, chunk) in bytes.chunks_mut(4).enumerate() {
                chunk.copy_from_slice(&hex_u32(&hex[i * 8..i * 8 + 8])?.to_le_bytes());
            }
            let ip = Ipv6Addr::from(bytes);
            Some(
                ip.to_ipv4_mapped()
                    .map_or_else(|| ip.to_string(), |v4| v4.to_string()),
            )
        }
        _ => None,
    }
}

fn parse_socket(line: &str) -> Option<Socket> {
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 10 {
        return None;
    }
    let state = state_name(cols[3])?;
    let (addr, port) = cols[1].rsplit_once(':').map_or((None, None), |(a, p)| {
        (decode_addr(a), u16::from_str_radix(p, 16).ok())
    });
    Some(Socket {
        inode: cols[9].to_string(),
        state,
        addr,
        port,
        uid: cols[7].parse().ok(),
    })
}

/// inode -> lowest pid that has `socket:[inode]` open. Other users' `fd`
/// directories are unreadable, so their listeners stay unresolved.
fn owners(sys: &Sys, want: &HashSet<&str>) -> HashMap<String, i64> {
    let mut found = HashMap::new();
    if want.is_empty() {
        return found;
    }
    let Ok(entries) = std::fs::read_dir(sys.path("proc")) else {
        return found;
    };
    let mut pids: Vec<i64> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            is_digits(&name).then(|| name.parse().ok()).flatten()
        })
        .collect();
    pids.sort_unstable();
    for pid in pids {
        let Ok(fds) = std::fs::read_dir(sys.path(&format!("proc/{pid}/fd"))) else {
            continue;
        };
        for fd in fds.flatten() {
            let Ok(link) = std::fs::read_link(fd.path()) else {
                continue;
            };
            let target = link.to_string_lossy();
            let Some(inode) = target
                .strip_prefix("socket:[")
                .and_then(|t| t.strip_suffix(']'))
            else {
                continue;
            };
            if want.contains(inode) {
                found.entry(inode.to_string()).or_insert(pid);
            }
        }
        if found.len() == want.len() {
            break;
        }
    }
    found
}

fn cached_owners(sys: &Sys, want: &HashSet<&str>) -> HashMap<String, i64> {
    let mut key: Vec<String> = want.iter().map(|s| s.to_string()).collect();
    key.sort();
    let proc_dir = sys.path("proc");
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(c) = cache.get(&proc_dir) {
        if c.want == key && c.at.elapsed() < OWNER_TTL {
            return c.found.clone();
        }
    }
    let found = owners(sys, want);
    let at = Instant::now();
    cache.insert(
        proc_dir,
        Cached {
            want: key,
            at,
            found: found.clone(),
        },
    );
    found
}

/// Listeners first, so the row cap never drops one in favour of a connection.
pub fn read_tcp_sockets(sys: &Sys) -> Vec<Value> {
    let mut socks: Vec<Socket> = tcp_lines(sys)
        .iter()
        .filter_map(|l| parse_socket(l))
        .collect();
    socks.sort_by_key(|s| s.state != LISTEN);
    socks.truncate(MAX_ROWS);
    let want: HashSet<&str> = socks
        .iter()
        .filter(|s| s.state == LISTEN)
        .map(|s| s.inode.as_str())
        .collect();
    let pids = cached_owners(sys, &want);
    socks
        .iter()
        .map(|s| {
            json!({
                "inode": s.inode, "state": s.state, "addr": s.addr,
                "port": s.port, "uid": s.uid, "pid": pids.get(&s.inode),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const V4: &str = "   0: 0100007F:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 5001 1 0000000000000000 100 0 0 10 0";
    const V6: &str = "   1: 00000000000000000000000001000000:0016 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 5002 1 0000000000000000 100 0 0 10 0";
    const EST: &str = "   2: 0100007F:D000 0100007F:1F90 01 00000000:00000000 00:00000000 00000000  1000        0 5003 1 0000000000000000 20 4 30 10 -1";

    #[test]
    fn addresses_decode_to_network_order() {
        assert_eq!(decode_addr("0100007F").as_deref(), Some("127.0.0.1"));
        assert_eq!(decode_addr("00000000").as_deref(), Some("0.0.0.0"));
        assert_eq!(
            decode_addr("00000000000000000000000001000000").as_deref(),
            Some("::1")
        );
        assert_eq!(
            decode_addr("0000000000000000FFFF00000100007F").as_deref(),
            Some("127.0.0.1")
        );
        assert_eq!(decode_addr("xyz"), None);
        assert_eq!(decode_addr("é0000000"), None);
    }

    #[test]
    fn socket_lines_carry_port_uid_and_inode() {
        let s = parse_socket(V4).unwrap();
        assert_eq!(
            (s.state, s.addr.as_deref(), s.port, s.uid, s.inode.as_str()),
            (LISTEN, Some("127.0.0.1"), Some(8080), Some(1000), "5001")
        );
        let s = parse_socket(V6).unwrap();
        assert_eq!(
            (s.addr.as_deref(), s.port, s.uid),
            (Some("::1"), Some(22), Some(0))
        );
        assert!(parse_socket("0: short").is_none());
        assert!(parse_socket(&V4.replace(" 0A ", " FF ")).is_none());
    }

    fn fixture(name: &str) -> (Sys, PathBuf) {
        let dir = std::env::temp_dir().join(format!("atmos-sockets-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("proc/net")).unwrap();
        (Sys(dir.clone()), dir)
    }

    fn holds(dir: &std::path::Path, pid: u32, inode: u32) {
        let fd = dir.join(format!("proc/{pid}/fd"));
        std::fs::create_dir_all(&fd).unwrap();
        std::os::unix::fs::symlink(format!("socket:[{inode}]"), fd.join("3")).unwrap();
    }

    #[test]
    fn listeners_come_first_and_resolve_to_the_lowest_holder() {
        let (sys, dir) = fixture("owners");
        let header = "  sl  local_address rem_address   st\n";
        std::fs::write(dir.join("proc/net/tcp"), format!("{header}{EST}\n{V4}\n")).unwrap();
        std::fs::write(dir.join("proc/net/tcp6"), format!("{header}{V6}\n")).unwrap();
        holds(&dir, 900, 5001);
        holds(&dir, 300, 5001);
        // A pid whose fd directory cannot be listed stays unresolved.
        std::fs::create_dir_all(dir.join("proc/400")).unwrap();

        let rows = read_tcp_sockets(&sys);
        let states: Vec<&str> = rows.iter().map(|r| r["state"].as_str().unwrap()).collect();
        assert_eq!(states, ["LISTEN", "LISTEN", "ESTABLISHED"]);
        assert_eq!(rows[0]["port"], 8080);
        assert_eq!(rows[0]["pid"], 300);
        assert_eq!(rows[1]["port"], 22);
        assert_eq!(rows[1]["uid"], 0);
        assert!(rows[1]["pid"].is_null());
        assert!(rows[2]["pid"].is_null());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn an_unchanged_listener_set_reuses_the_last_walk() {
        let (sys, dir) = fixture("cache");
        let header = "  sl  local_address rem_address   st\n";
        std::fs::write(dir.join("proc/net/tcp"), format!("{header}{V4}\n")).unwrap();
        holds(&dir, 300, 5001);
        assert_eq!(read_tcp_sockets(&sys)[0]["pid"], 300);
        std::fs::remove_dir_all(dir.join("proc/300")).unwrap();
        holds(&dir, 301, 5001);
        assert_eq!(read_tcp_sockets(&sys)[0]["pid"], 300);
        // A new listener changes the set, so the walk runs again.
        std::fs::write(dir.join("proc/net/tcp"), format!("{header}{V4}\n{V6}\n")).unwrap();
        assert_eq!(read_tcp_sockets(&sys)[0]["pid"], 301);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_proc_gives_no_rows() {
        let (sys, dir) = fixture("empty");
        assert!(read_tcp_sockets(&sys).is_empty());
        assert!(owners(&sys, &HashSet::from(["1"])).is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }
}
