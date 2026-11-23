use crate::ast::*;
use crate::display::{self, clock, dim, green};
use crate::parser::parse;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, IsTerminal, Write};
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
    let live = std::io::stderr().is_terminal();
    display::info(&format!("recording {}", text));
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
    let mut count = 0;
    for line in BufReader::new(stdout).lines() {
        let line = line.expect("could not read from bpftrace");
        if let Some(row) = parse_event(&line) {
            writeln!(log, "{}", line).expect("could not write events.log");
            count += 1;
            if live {
                eprintln!(
                    "  {} {}  {:<16} pid {:<8} {}",
                    green("+"),
                    dim(&clock(row.time)),
                    row.name,
                    row.pid,
                    user_name(&row.user)
                );
            }
        }
    }
    let status = child.wait().expect("bpftrace did not finish");
    if !status.success() {
        display::error("bpftrace failed, nothing was recorded");
        return;
    }
    let mut history = OpenOptions::new()
        .create(true)
        .append(true)
        .open(HISTORY_FILE)
        .expect("could not open history.txt");
    let end = now();
    writeln!(history, "{} {} {}", start, end, text).expect("could not write history.txt");
    display::ok(&format!(
        "recorded {} event{} in {}s",
        count,
        if count == 1 { "" } else { "s" },
        end - start
    ));
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

fn user_name(uid: &str) -> String {
    let passwd = std::fs::read_to_string("/etc/passwd").unwrap_or_default();
    for line in passwd.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() > 2 && fields[2] == uid {
            return fields[0].to_string();
        }
    }
    uid.to_string()
}

fn parse_history_line(line: &str) -> Option<RecordRun> {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() != 3 {
        return None;
    }
    let start = parts[0].parse().ok()?;
    let end = parts[1].parse().ok()?;
    match parse(parts[2])? {
        Stmt::Record(rule) => Some(RecordRun { rule, start, end }),
        _ => None,
    }
}

pub fn read_log() -> Vec<Row> {
    let text = std::fs::read_to_string(LOG_FILE).unwrap_or_default();
    let mut rows = Vec::new();
    for line in text.lines() {
        if let Some(mut row) = parse_event(line) {
            row.user = user_name(&row.user);
            rows.push(row);
        }
    }
    rows
}

pub fn read_history() -> Vec<RecordRun> {
    let text = std::fs::read_to_string(HISTORY_FILE).unwrap_or_default();
    let mut runs = Vec::new();
    for line in text.lines() {
        if let Some(run) = parse_history_line(line) {
            runs.push(run);
        }
    }
    runs
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

    // uid 0 is always root, and an unknown uid stays a number
    #[test]
    fn test_user_name() {
        assert_eq!(user_name("0"), "root");
        assert_eq!(user_name("4294967295"), "4294967295");
    }

    // a history line comes back as a Record through the parser
    #[test]
    fn test_parse_history_line() {
        let run =
            parse_history_line("100 130 RECORD processes WHERE name = 'curl' FOR 30").unwrap();
        assert_eq!(run.start, 100);
        assert_eq!(run.end, 130);
        assert_eq!(run.rule.seconds, 30);
        assert_eq!(run.rule.conds[0].value, "curl");
        assert_eq!(
            parse_history_line("100 130 SELECT pid FROM processes"),
            None
        );
        assert_eq!(parse_history_line("garbage"), None);
    }
}
