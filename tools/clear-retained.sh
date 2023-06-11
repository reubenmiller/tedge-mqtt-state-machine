#!/bin/sh
TOPICS=$(mosquitto_sub -t 'tedge/operations/main-device/+/+/+' -W 1 -F "%t" 2>/dev/null)

if [ -z "$TOPICS" ]; then
    echo "No retained topics found"
    exit 0
fi

printf '%s\n' "$TOPICS" | while IFS= read -r TOPIC
do
   echo "Deleteing retained message on topic: $TOPIC"
   mosquitto_pub -t "$TOPIC" -n -d -r
done
