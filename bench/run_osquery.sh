#!/bin/bash
# usage: run_osquery.sh N K [RATE]   osquery audit mode, tuned so its queues do not overflow
# prints the same columns as run_sysql.sh
B=$(cd "$(dirname "$0")" && pwd)
source "$B/common.sh"
N=$1; K=$2; RATE=$3; O=/tmp/osq
W=$(drain_wait "$N")
sudo rm -rf $O; mkdir -p $O; echo "{}" > $O/conf.json
sudo osqueryd --ephemeral --database_path=$O/db --extensions_socket=$O/sock --pidfile=$O/pid \
    --config_path=$O/conf.json --logger_plugin=filesystem --logger_path=$O --disable_events=false \
    --disable_audit=false --audit_allow_config=true --audit_allow_process_events=true \
    --audit_backlog_limit=65536 --events_max=1000000 --events_expiry=3600 --verbose > $O/out.txt 2>&1 &
until sudo grep -q "Enabling audit rules for the process_events" $O/out.txt 2>/dev/null; do sleep 0.1; done
sleep 3
PIDS=$(pgrep -x osqueryd | tr '\n' ' ')
START=$(cpu_ns $PIDS)
/usr/bin/time -f "%e" -o $O/work.time "$B/workload.sh" "$N" "$K" $RATE
sleep "$W"
END=$(cpu_ns $PIDS)
MEM=$(mem_mb $PIDS)
q() { sudo osqueryi --connect $O/sock --csv "$1" 2> /dev/null | tail -1; }
T0=$(date +%s%N)
FOUND=$(q "select count(*) from process_events where path like '%/date';")
T1=$(date +%s%N)
ALL=$(q "select count(*) from process_events;")
sudo pkill -x osqueryd; while pgrep -x osqueryd > /dev/null; do sleep 0.2; done
EXPECTED=0; ((N > 0)) && EXPECTED=$((N / K))
echo "osquery,tuned,$N,$K,${RATE:-burst},$(sec $START),$(sec $((END - START))),$MEM,$(cat $O/work.time),$ALL,$FOUND,$EXPECTED,$(sec $((T1 - T0)))"
