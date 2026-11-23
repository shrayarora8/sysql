use crate::ast::*;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub const LOG_FILE: &str = "events.log";
pub const HISTORY_FILE: &str = "history.txt";

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn record(text: &str, script: &str) {
    let start = now();
    let mut child = Command::new("sudo")
        .arg("bpftrace")
        .arg("-e")
        .arg(script)
        .stdout(Stdio::piped())
        .spawn()
        .expect("could not start bpftrace");
    let stdout = child.stdout.take().unwrap();
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)
        .expect("could not open events.log");
    for line in BufReader::new(stdout).lines() {
        let line = line.expect("could not read from bpftrace");
        if parse_event(&line).is_some() {
            writeln!(log, "{}", line).expect("could not write events.log");
        }
    }
    let status = child.wait().expect("bpftrace did not finish");
    if !status.success() {
        eprintln!("bpftrace failed, nothing was recorded");
        return;
    }
    let mut history = OpenOptions::new()
        .create(true)
        .append(true)
        .open(HISTORY_FILE)
        .expect("could not open history.txt");
    writeln!(history, "{} {} {}", start, now(), text).expect("could not write history.txt");
}

fn parse_event(line: &str) -> Option<Row> {
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    if parts.len() != 4 {
        return None;
    }
    let time = parts[0].parse().ok()?;
    let pid: u32 = parts[1].parse().ok()?;
    let uid: u32 = parts[2].parse().ok()?;
    Some(Row {
        time,
        pid,
        name: parts[3].to_string(),
        user: uid.to_string(),
    })
}

pub fn read_log() -> Vec<Row> {
    todo!()
}

pub fn read_history() -> Vec<RecordRun> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_event() {
        assert_eq!(
            parse_event("1795390635 219653 501 curl"),
            Some(Row {
                time: 1795390635,
                pid: 219653,
                name: "curl".to_string(),
                user: "501".to_string(),
            })
        );
        assert_eq!(parse_event("1795 219 501 my app").unwrap().name, "my app");
        assert_eq!(parse_event(""), None);
        assert_eq!(parse_event("Attached 2 probes"), None);
        assert_eq!(parse_event("a b c d"), None);
    }
}
