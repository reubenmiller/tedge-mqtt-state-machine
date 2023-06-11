set dotenv-load

# Build
build:
    cargo build

# Release
release:
    cargo build --release

# Run debug
run:
    RUST_LOG=info cargo run

# ------------------------------------
# Demo
# ------------------------------------

# Start demo
up:
    cd integration && docker compose up --build -d

# Bootstrap demo
bootstrap *ARGS="":
    cd integration && docker compose exec tedge env C8Y_BASEURL=${C8Y_BASEURL:-} C8Y_USER=${C8Y_USER:-} C8Y_PASSWORD=${C8Y_PASSWORD:-} DEVICE_ID=${DEVICE_ID:-} bootstrap.sh {{ARGS}}

# Stop demo
down:
    cd integration && docker compose down -v

# Start shell inside demo container
shell:
    cd integration && docker compose exec tedge bash

# View state machine logs in demo
view-state-machine:
    cd integration && docker compose exec tedge journalctl -fu tedge-mqtt-state-machine -n 100

# View MQTT operations topic in demo
view-mqtt:
    cd integration && docker compose exec tedge mosquitto_sub -t 'tedge/operations/+/+/+/+' -F '%t\t%p'

# View external updater
view-external-updater:
    cd integration && docker compose exec tedge journalctl -fu external-updater -n 100

# Publish firmware operation which will pass
publish-firmware-operation-successful:
    cd integration && docker compose exec tedge mosquitto_pub -t tedge/operations/main-device/firmware/update/123 -m '{"status":"init", "target":"mosquito"}'

# Publish firmware operation which will fail
publish-firmware-operation-failed:
    cd integration && docker compose exec tedge mosquitto_pub -t tedge/operations/main-device/firmware/update/123 -m '{"status":"init", "target":"mosquito", "healthy":false}'

# Publish external operation which will pass
publish-external-operation-successful:
    cd integration && docker compose exec tedge mosquitto_pub -t tedge/operations/main-device/external/update/123 -m '{"status":"init", "target":"mosquito","children":["child01"]}'

# Publish external operation which will fail
publish-external-operation-failed:
    cd integration && docker compose exec tedge mosquitto_pub -t tedge/operations/main-device/external/update/123 -m '{"status":"init", "target":"mosquito","children":["child01","child02"]}'
