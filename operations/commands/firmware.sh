#!/bin/sh

set -e

log() {
    echo "$*" >&2
}

log "Running command: $0 $*"

CURRENT_STATE='{}'
if [ $# -gt 1 ]; then
    CURRENT_STATE="$2"
fi

next_state() {
    new_status="$1"
    current_status=$(echo "$CURRENT_STATE" | jq -r '.status')

    EVENT_MESSAGE=$(printf '{"text":"[%s] state machine: [%s] ➜ [%s]"}' "firmware" "$current_status" "$new_status")
    mosquitto_pub -t 'tedge/events/tedge_StateMachineTransition' -m "$EVENT_MESSAGE"
    echo "$CURRENT_STATE" | jq '.status = "'"$new_status"'"' -c -M
}

#
# States
#
healthcheck() {
    next_state "commit"
}

verify() {
    next_state "installing"
}

rollback() {
    next_state "failed"
}

commit() {
    next_state "successful"
}

handle_reboot() {
    OP_ID=$(echo "$CURRENT_STATE" | jq -r '.id // 123')
    REBOOT_FLAG=".reboot-$OP_ID"

    if [ -f "$REBOOT_FLAG" ]; then
        log "Already rebooted. Removing flag"
        rm -f "$REBOOT_FLAG"
        next_state "healthcheck"
    else
        touch "$REBOOT_FLAG"
        log "Shutting down...this will kill the script at any time"
        sleep 2

        # Restart via tedge interface
        MESSAGE=$(printf '{"id":"%s"}' "$OP_ID")
        mosquitto_pub -t 'tedge/commands/req/control/restart' -m "$MESSAGE"
        log "Waiting for system to shutdown..."
        sleep 120
    fi
}

#
# main
#
case "$1" in
    verify)
        verify
        ;;

    scheduled)
        next_state "downloading"
        ;;

    downloading)
        next_state "downloaded"
        ;;

    installing)
        next_state "reboot"
        ;;

    reboot)
        handle_reboot
        ;;

    healthcheck)
        healthcheck
        ;;

    commit)
        commit
        ;;

    rollback)
        rollback
        ;;

    successful)
        next_state "done"
        ;;

    failed)
        next_state "done"
        ;;

    *)
        log "Unknown command: $1"
        next_state "failed"
        ;;

esac

sleep 2
