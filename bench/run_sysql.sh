#!/bin/bash
# usage: run_sysql.sh MODE N K [RATE]   MODE is filtered or all
# prints: tool,mode,n,k,rate,startup_s,record_cpu_s,mem_mb,workload_s,events,target_found,target_expected,query_s
B=$(cd "$(dirname "$0")" && pwd)
source "$B/common.sh"
MODE=$1; N=$2; K=$3; RATE=$4
W=$(drain_wait "$N")
if [[ -n $RATE ]]; then EST=$((N / RATE)); else EST=$((N / 4000 + N / K / 1000)); fi
((N == 0)) && EST=30
FOR=$((EST * 2 + W + 6))
if [[ $MODE == filtered ]]; then
    STMT="RECORD processes WHERE name = 'date' FOR $FOR"
else
    STMT="RECORD processes FOR $FOR"
fi
D=$(mktemp -d); cd "$D"
~/sysql-target/release/sysql "$STMT" 2> rec.err &
SYSQL=$!
until grep -q Attached rec.err 2>/dev/null; do sleep 0.02; done
BPF=$(pgrep -n -x bpftrace)
START=$(cpu_ns $BPF $SYSQL)
/usr/bin/time -f "%e" -o work.time "$B/workload.sh" "$N" "$K" $RATE
sleep "$W"
if [[ ! -d /proc/$BPF ]]; then echo "error: recording ended before measurement"; exit 1; fi
END=$(cpu_ns $BPF $SYSQL)
MEM=$(mem_mb $BPF $SYSQL)
wait
EXPECTED=0; ((N > 0)) && EXPECTED=$((N / K))
echo "sysql,$MODE,$N,$K,${RATE:-burst},$(sec $START),$(sec $((END - START))),$MEM,$(cat work.time),$(wc -l < events.log),$(grep -c ' date$' events.log),$EXPECTED,"
rm -rf "$D"
