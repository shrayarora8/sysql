#!/bin/bash
# usage: workload.sh N K [RATE]
# starts N programs, every K-th is /bin/date and the rest are /bin/true
# with RATE, starts about RATE programs per second instead of as fast as possible
# N=0 means stay idle for 30 seconds
N=$1; K=$2; RATE=$3
if ((N == 0)); then sleep 30; exit; fi
for ((i = 1; i <= N; i++)); do
    if ((i % K == 0)); then
        /bin/date > /dev/null
    else
        /bin/true
    fi
    if [[ -n $RATE ]] && ((i % 100 == 0)); then
        sleep "$(echo "scale=3; 100 / $RATE" | bc)"
    fi
done
