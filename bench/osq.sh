#!/bin/bash
# osquery helper for demos
#   osq.sh start          start osquery recording every program start
#   osq.sh query "<sql>"  ask osquery a question
#   osq.sh stop           stop osquery
O=/tmp/osq
case $1 in
start)
    sudo pkill -x osqueryd; sudo rm -rf $O; mkdir -p $O; echo "{}" > $O/conf.json
    sudo osqueryd --ephemeral --database_path=$O/db --extensions_socket=$O/sock \
        --config_path=$O/conf.json --logger_plugin=filesystem --logger_path=$O --disable_events=false \
        --disable_audit=false --audit_allow_config=true --audit_allow_process_events=true \
        --audit_backlog_limit=65536 --events_max=1000000 --events_expiry=86400 --verbose > $O/out.txt 2>&1 &
    until sudo grep -q "Enabling audit rules for the process_events" $O/out.txt 2>/dev/null; do sleep 0.1; done
    sleep 2
    echo "osquery is recording every program start"
    ;;
query)
    sudo osqueryi --connect $O/sock "$2" 2>/dev/null | grep -v "^Connected"
    ;;
stop)
    sudo pkill -x osqueryd && echo "osquery stopped"
    ;;
esac
