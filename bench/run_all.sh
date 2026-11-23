#!/bin/bash
# runs every experiment 3 times, with the tools interleaved, and appends to results.csv
B=$(cd "$(dirname "$0")" && pwd)
OUT=$B/results.csv
echo "experiment,tool,mode,n,k,rate,startup_s,record_cpu_s,mem_mb,workload_s,events,target_found,target_expected,query_s" > "$OUT"
run() {
    local exp=$1; shift
    local line
    if [[ $1 == osquery ]]; then line=$("$B/run_osquery.sh" "${@:2}"); else line=$("$B/run_sysql.sh" "${@:2}"); fi
    echo "$exp,$line" | tee -a "$OUT"
}
for rep in 1 2 3; do
    echo "--- repetition $rep"
    for n in 0; do
        run idle sysql filtered 0 100; run idle osquery 0 100
    done
    for n in 100 1000 10000 50000 100000; do
        run size sysql filtered $n 100; run size sysql all $n 100; run size osquery $n 100
    done
    for k in 1000 10 2 1; do
        run match sysql filtered 10000 $k; run match osquery 10000 $k
    done
    run steady sysql filtered 10000 100 500; run steady sysql all 10000 100 500; run steady osquery 10000 100 500
done
echo "done"
