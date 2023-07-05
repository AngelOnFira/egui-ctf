FROM rust:1 as builder

WORKDIR /app

COPY . /app

# Install ld
RUN apt-get update && apt-get install -y \
    binutils-dev
    
RUN cd discord-bot; cargo build --release


FROM debian:buster-slim

COPY --from=builder /app/target/release/h4tt-discord-bot /app/h4tt-discord-bot

ENTRYPOINT ["/app/h4tt-discord-bot"]