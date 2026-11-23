# total cpu time in nanoseconds used by the given processes, summed over all their threads
cpu_ns() {
    local total=0 p t v
    for p in "$@"; do
        for t in /proc/$p/task/*/schedstat; do
            v=$(sudo cut -d' ' -f1 "$t" 2>/dev/null) && total=$((total + v))
        done
    done
    echo $total
}

# peak memory of the given processes in MB, added together
mem_mb() {
    local total=0 p
    for p in "$@"; do
        total=$((total + $(sudo awk '/VmHWM/ {print $2}' /proc/$p/status)))
    done
    echo $((total / 1024))
}

# seconds to wait after the workload so each tool can finish handling what it got
drain_wait() { echo $((4 + $1 / 20000)); }

sec() { echo "scale=4; $1 / 1000000000" | bc; }
