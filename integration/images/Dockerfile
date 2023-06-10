FROM rust:1.69 AS builder

# Faster rust builds in docker
# https://www.reddit.com/r/rust/comments/126xeyx/exploring_the_problem_of_faster_cargo_docker/

# Create a new empty shell project
WORKDIR /app

# Copy over the Cargo.toml files to the shell project
COPY Cargo.toml Cargo.lock ./

# Build and cache the dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN cargo build --release
RUN rm src/main.rs

# Copy the actual code files and build the application
COPY src ./src/
# Update the file date
RUN touch src/main.rs
RUN cargo build --release

FROM ghcr.io/thin-edge/tedge-demo-main-systemd:20230517.2

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get -y --no-install-recommends install \
        git \
        jq \
        curl

RUN curl https://reubenmiller.github.io/go-c8y-cli-repo/debian/PUBLIC.KEY | gpg --dearmor | tee /usr/share/keyrings/go-c8y-cli-archive-keyring.gpg >/dev/null \
    && sh -c "echo 'deb [signed-by=/usr/share/keyrings/go-c8y-cli-archive-keyring.gpg] http://reubenmiller.github.io/go-c8y-cli-repo/debian stable main' >> /etc/apt/sources.list" \
    && apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get -y --no-install-recommends install \
        go-c8y-cli

COPY --from=builder /app/target/release/tedge-mqtt-state-machine /usr/bin/tedge-mqtt-state-machine
COPY integration/images/tedge-mqtt-state-machine.service /lib/systemd/system/

RUN mkdir -p /etc/tedge-mqtt-state-machine/operations
COPY operations /etc/tedge-mqtt-state-machine/operations/
RUN systemctl enable tedge-mqtt-state-machine mosquitto
