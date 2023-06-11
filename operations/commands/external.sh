#!/bin/sh

set -e
SUBCOMMAND="$1"

case "$SUBCOMMAND" in
    start)
        systemctl enable external-updater
        systemctl start external-updater
        python3 -m ext_updater start "$2"
        ;;

    stop)
        systemctl disable external-updater
        systemctl stop external-updater
        python3 -m ext_updater stop "$2"
        ;;
    
    successful)
        printf '{"status":""}\n'
        ;;
    
    failed)
        printf '{"status":""}\n'
        ;;

    *)
        exit 1
        ;;
esac
