# SysQL

A small SQL-like language for recording and querying what programs run on a Linux machine.

`RECORD` says what to watch for. It is compiled into an eBPF filter that runs **inside the kernel**, so events you didn't ask for are thrown away before they cost anything. `SELECT` asks questions about what was recorded.

```
sysql> RECORD processes WHERE name = 'curl' FOR 30
● recording RECORD processes WHERE name = 'curl' FOR 30
  + 2026-11-23 03:20:05  curl             pid 2422102  ubuntu
  + 2026-11-23 03:20:05  curl             pid 2422107  root
✓ recorded 2 events in 30s

sysql> SELECT time, pid, user FROM processes WHERE name = 'curl'
┌─────────────────────┬─────────┬────────┐
│ time                │ pid     │ user   │
├─────────────────────┼─────────┼────────┤
│ 2026-11-23 03:20:05 │ 2422102 │ ubuntu │
│ 2026-11-23 03:20:05 │ 2422107 │ root   │
└─────────────────────┴─────────┴────────┘
(2 rows)
```

## Install

**Linux** (Ubuntu or Debian), one line:

```bash
cd ~ && git clone https://github.com/shrayarora8/sysql && cd sysql && ./install.sh
```

`install.sh` installs bpftrace and Rust if they're missing, builds SysQL, and puts `sysql` on your PATH.

**Mac**, start a Linux VM with [Lima](https://lima-vm.io) first, then run the Linux line inside it:

```bash
brew install lima && limactl start --name=sysql --tty=false template://ubuntu-lts && limactl shell sysql
```

**Windows**: run the Linux line inside WSL2 (Ubuntu).

## Try it

Open two terminals on the Linux machine.

1. In the first one, start SysQL and record:

```bash
sysql
```

```
RECORD processes FOR 20
```

2. In the second one, within those 20 seconds, do anything. For example, compile a tiny C program:

```bash
echo 'int main(){return 0;}' > /tmp/a.c && cc /tmp/a.c -o /tmp/a
```

3. Back in the first one, ask what happened:

```
SELECT time, pid, name FROM processes
```

You'll see every program that started, including the ones `cc` ran behind the scenes (`cc1`, `as`, `collect2`, `ld`).

## Change what it does

| to change | write | example |
|---|---|---|
| which events to keep | `WHERE column = value` | `WHERE name = 'ssh'`, `WHERE user = 'root'`, `WHERE pid = 1234` |
| several conditions at once | `AND` | `WHERE name = 'curl' AND user = 'root'` |
| keep everything | leave out WHERE | `RECORD processes FOR 60` |
| how long to record | `FOR seconds` | `FOR 3600` is an hour |
| which columns to show | list them, or `*` | `SELECT name, user FROM processes` |

Columns: `time`, `pid`, `name` (first 15 characters, a kernel limit), `user`.

In the shell: `\help`, `\d` (columns), `\history` (past recordings), `\q` (quit). Up-arrow brings back earlier lines, and keywords work in any case.

One statement without the shell:

```bash
sysql "SELECT * FROM processes WHERE name = 'curl'"
```

Recordings are saved in `events.log` and `history.txt` in the folder you run `sysql` from.

## How it works

```
"RECORD processes WHERE name = 'curl' FOR 30"
     │ parser      text → syntax tree
     │ compiler    syntax tree → bpftrace program
     ▼
tracepoint:sched:sched_process_exec /comm == "curl"/ { printf(...); }
interval:s:30 { exit(); }
     │ bpftrace loads this into the kernel; the filter runs on every program start
     │ recorder    reads the matching events and appends them to events.log
     ▼
"SELECT ..."  → parser → interpreter reads events.log → table
```

The code is in `src/`: `parser.rs`, `compiler.rs`, `recorder.rs`, `interpreter.rs`, `analyzer.rs`, plus `repl.rs` and `display.rs` for the shell.

## Compared with osquery

Recording one specific program while a machine started 100 to 100,000 programs, SysQL used **4x to 84x less CPU** than osquery while recording, and kept every event. Under sustained heavy load, osquery silently lost 25–40% of events. For very small recordings osquery is cheaper overall, because SysQL spends about 0.1 s starting bpftrace. Full numbers, method and caveats: [bench/RESULTS.md](bench/RESULTS.md).

## Limits

- One table (`processes`), four columns.
- Only `=` and `AND` (no `<`, `>`, OR, COUNT or time ranges yet).
- Needs Linux, bpftrace and sudo, because loading code into the kernel needs root.

## Tests

```bash
cargo test
```

ECS 240 (UC Davis) final project.
