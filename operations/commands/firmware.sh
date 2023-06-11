#!/bin/sh

set -e

log() { echo "$*" >&2; }

log "Running command: $0 $*"

SUBCOMMAND="$1"
shift

CURRENT_STATE='{}'
if [ $# -gt 0 ]; then
    if [ -n "$1" ]; then
        if echo "$1" | jq '.' >/dev/null 2>&1; then
            CURRENT_STATE="$1"
        else
            log "Non-json input detected"
        fi
    fi
    shift
fi

#
# Helpers
#
next_state() {
    new_status="$1"
    current_status=$(echo "$CURRENT_STATE" | jq -r '.status // "unknown"')

    EVENT_MESSAGE=$(printf '{"text":"[%s] state machine: [%s] ➜ [%s]"}' "firmware" "$current_status" "${new_status:-done}")
    mosquitto_pub -t 'tedge/events/tedge_StateMachineTransition' -m "$EVENT_MESSAGE"

    OTHER_PROPS="{}"
    if [ $# -gt 1 ]; then
        OTHER_PROPS="$2"
    fi

    # Protect against template errors as they cause a infinite loop
    NEXT_STATE=$(
        c8y template execute -n --template "
            local state = $CURRENT_STATE;
            state +
            {status: '$new_status'} +
            $OTHER_PROPS +
            {i: std.get(state, 'i', 0) + 1}
        " -o json -c -M 2>/dev/null ||:
    )
    if [ -z "$NEXT_STATE" ]; then
        log "Detected state transition error. Falling back to building state without a template"
        NEXT_STATE=$(printf '{"status":"%s"}' "$new_status")
    fi
    printf '%s\n' "$NEXT_STATE"
}

#
# States
#
healthcheck() {
    IS_HEALTHY=$(echo "$CURRENT_STATE" | jq -r '. | if has("healthy") then .healthy else true end')
    if [ "$IS_HEALTHY" = "true" ]; then
        next_state "commit"
    else
        next_state "rollback"
    fi
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
case "$SUBCOMMAND" in
    init)
        next_state "scheduled"  "{startedAt: _.Now()}"
        ;;

    scheduled)
        next_state "downloading"
        ;;

    downloading)
        next_state "downloaded"
        ;;

    downloaded)
        next_state "installing"
        ;;

    verify)
        verify
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
        next_state "" "{finishedAt: _.Now()}"
        ;;

    failed)
        next_state "" "{finishedAt: _.Now()}"
        ;;

    *)
        log "Unknown command: $1"
        next_state "failed"
        ;;

esac

sleep 1
