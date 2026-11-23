# Benchmark: SysQL vs osquery

**Question:** what does it cost to record "every time a certain program runs" on a machine that is starting lots of programs, and does each tool keep the complete history?

## Setup

| | |
|---|---|
| machine | Lima VM on an M-series Mac: Ubuntu, Linux 7.0, 4 CPUs, 4 GB RAM |
| workload | `workload.sh N K [RATE]` starts N programs. Every K-th is `/bin/date` (the target), the rest are `/bin/true`. Without RATE it starts them as fast as it can (about 3,500 per second); with RATE it paces them |
| SysQL filtered | `RECORD processes WHERE name = 'date' FOR ...` |
| SysQL unfiltered | `RECORD processes FOR ...` (record everything, filter later) |
| osquery | osquery 5.24.1, audit mode, `process_events` table, with its audit queue and event cap raised so it has its best chance (`run_osquery.sh`) |
| recording CPU | CPU time of the recording processes (all threads) from just before the workload until a few seconds after it, read in nanoseconds from `/proc/<pid>/task/*/schedstat` |
| start-up CPU | CPU used before recording began. For SysQL this is bpftrace compiling and loading the filter, paid once per RECORD. For osquery it is the daemon starting, paid once |
| complete? | target runs found vs target runs started |
| runs | 3 per setting, tools interleaved, median reported. Raw data in `results.csv` |

osquery's eBPF mode (`bpf_process_events`) failed to process exec events on this kernel ("Failed to process one or more events from the following tracers: 149"), so audit mode was used. Audit mode is osquery's standard way to record processes on Linux.

## 1. Size: 1% of programs are the target, started as fast as possible

| program starts | SysQL filtered | SysQL unfiltered | osquery | osquery ÷ SysQL filtered | target found: SysQL / osquery |
|---|---|---|---|---|---|
| 100 | 0.005 s | 0.008 s | 0.021 s | 4x | 1/1 · 1/1 |
| 1,000 | 0.006 s | 0.026 s | 0.080 s | 14x | 10/10 · 10/10 |
| 10,000 | 0.011 s | 0.206 s | 0.655 s | 62x | 100/100 · 88–100/100 |
| 50,000 | 0.035 s | 1.057 s | 1.035 s | 29x | 500/500 · 152–177/500 |
| 100,000 | 0.063 s | 2.082 s | 3.798 s | 61x | 1000/1000 · 453–753/1000 |

At 50,000 and 100,000 osquery's CPU is lower than it would be, because it only processed the 30–60% of events it managed to receive.

**Counting start-up too** (one recording session, start to finish):

| program starts | SysQL filtered | osquery | cheaper |
|---|---|---|---|
| 100 | 0.16 s | 0.07 s | osquery, 2.5x |
| 1,000 | 0.13 s | 0.11 s | about even |
| 10,000 | 0.11 s | 0.69 s | SysQL, 6x |
| 100,000 | 0.19 s | 3.85 s | SysQL, 20x |

## 2. How much the filter throws away: 10,000 starts

| share that is the target | SysQL filtered | osquery | osquery ÷ SysQL |
|---|---|---|---|
| 0.1% | 0.006 s | 0.518 s | 84x |
| 1% | 0.011 s | 0.655 s | 62x |
| 10% | 0.029 s | 0.573 s | 20x |
| 50% | 0.117 s | 0.528 s | 4.5x |
| 100% (the filter keeps everything) | 0.222 s | 0.542 s | 2.4x |

At 100% the filter cannot help, so the remaining 2.4x comes from SysQL doing less work per event (4 fields vs osquery's ~20).

## 3. Steady load: 10,000 starts at about 370 per second

| | recording CPU | target found |
|---|---|---|
| SysQL filtered | 0.039 s | 100/100 |
| SysQL unfiltered | 0.566 s | 100/100 |
| osquery | 1.087 s | 100/100 |

## 4. Idle: 30 seconds, nothing happening

| | CPU |
|---|---|
| SysQL | 0.042 s |
| osquery | 0.068 s |

## 5. Memory (peak)

SysQL: 168–170 MB at every size, almost all of it bpftrace's built-in compiler, held only while a recording runs. osquery: 124 MB idle, growing to 262 MB at 100,000 starts.

## 6. When osquery loses events (`why_drops.sh`)

osquery and SysQL unfiltered recorded the same workload at the same time, so SysQL's count is the ground truth.

| workload | osquery kept |
|---|---|
| 500 / 1,000 / 2,000 / 5,000 / 10,000 starts in one burst (~3,500/s) | 99.9% |
| 10,000 starts paced at 370 / 650 / 1,200 / 1,500 per second | 99.9% |
| 20,000 in one burst | 75.9% |
| 50,000 in one burst | 66% (still 66% after waiting 60 s, so the events were gone, not late) |

- osquery keeps the complete history under normal load. It only loses events when heavy load lasts for more than a few seconds.
- It is not short of CPU: its busiest thread (`AuditdNetlinkReader`) stayed under 10% of one core while losing events.
- The kernel reported audit records it could not deliver: 188,796 of them during the 50,000 run (`kauditd_printk_skb ... callbacks suppressed` in `dmesg`). osquery takes records off the kernel's audit queue more slowly than they arrive during a long burst, the queue fills, and the kernel discards the rest.
- SysQL kept every event in every run, filtered or not. Its path out of the kernel (eBPF buffer, bpftrace, a pipe) handled the full burst rate.

## What this shows

1. **Recording something specific costs SysQL 4x to 84x less CPU than osquery**, growing with how busy the machine is and how much the filter throws away.
2. **The filter is the reason.** SysQL filtered vs unfiltered is the same code with one change, and the gap matches the share of events thrown away in the kernel.
3. **For very small recordings osquery is cheaper overall,** because SysQL pays about 0.1 s of start-up per RECORD. Above about 1,000 program starts per recording, SysQL is cheaper even counting start-up.
4. **Under sustained heavy load osquery's history becomes incomplete without any warning;** SysQL's did not.

## Caveats

- One small VM and a synthetic workload (`/bin/true` in a loop). A real workload (a large build) and a longer run on a bigger machine are next.
- osquery stores about 20 fields per event and SysQL stores 4. Section 2's 100% row shows how much of the gap comes from that.
- How much each tool slows the workload down was within the VM's run-to-run noise (about 5%), so it is not reported as a result. Baselines without any recording: 0.05 / 0.23 / 2.29 / 12.04 / 24.34 s for 100 / 1k / 10k / 50k / 100k starts.
- Query speed is not measured yet.

## Rerun it (inside the Linux VM, from the repo folder)

```bash
cargo build --release --target-dir ~/sysql-target
```

```bash
bench/run_all.sh
```

Single runs:

```bash
bench/run_sysql.sh filtered 10000 100
```

```bash
bench/run_osquery.sh 10000 100
```

```bash
bench/why_drops.sh 20000
```

`run_sysql.sh` expects the binary at `~/sysql-target/release/sysql`. The osquery scripts need osquery installed (`osquery_5.24.1-1.linux_arm64.deb` from github.com/osquery/osquery/releases).
