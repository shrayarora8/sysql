#!/bin/bash
# usage: why_drops.sh N [RATE]
# osquery and SysQL (unfiltered, the ground truth) record the same workload together
B=$(cd "$(dirname "$0")" && pwd)
N=$1; RATE=$2
supp() { sudo dmesg | grep -o "kauditd_printk_skb: [0-9]* callbacks suppressed" | awk '{s+=$2} END {print s+0}'; }
threads() { for t in /proc/$1/task/*; do echo "$(sudo cat $t/comm) $(sudo cut -d' ' -f1 $t/schedstat)"; done; }
D=$(mktemp -d); cd "$D"
"$B/osq.sh" start > /dev/null
OSQ=$(pgrep -n -x osqueryd)
if [[ -n $RATE ]]; then FOR=$((N / RATE + 25)); else FOR=$((N / 3000 + 25)); fi
~/sysql-target/release/sysql "RECORD processes FOR $FOR" > /dev/null 2>&1 &
sleep 3
S0=$(supp); threads $OSQ > before.txt
T0=$(date +%s%N); "$B/workload.sh" "$N" 100 $RATE; T1=$(date +%s%N)
threads $OSQ > after.txt
sleep 8
OSQ_EVENTS=$("$B/osq.sh" query "select count(*) from process_events;" | sed -n 4p | tr -d '| ')
LOST=$(( $(supp) - S0 ))
wait
TRUTH=$(wc -l < events.log)
WALL=$((T1 - T0))
BUSIEST=$(join <(sort before.txt) <(sort after.txt) 2>/dev/null | awk -v w=$WALL '{d=$3-$2; if (d>m) {m=d; n=$1}} END {printf "%s %.0f%%", n, 100*m/w}')
"$B/osq.sh" stop > /dev/null
printf "%-7s rate=%-6s %6.0f starts/s | truth=%-6s osquery=%-6s kept=%5.1f%% | kernel_undelivered=%-7s | busiest osquery thread: %s\n" \
    "$N" "${RATE:-burst}" "$(echo "$N * 1000000000 / $WALL" | bc)" "$TRUTH" "$OSQ_EVENTS" \
    "$(echo "scale=1; 100 * $OSQ_EVENTS / $TRUTH" | bc)" "$LOST" "$BUSIEST"
rm -rf "$D"
