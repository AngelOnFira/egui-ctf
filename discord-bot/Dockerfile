FROM ghcr.io/angelonfira/egui-ctf/ctf-discord-bot-base:latest as builder

WORKDIR /app

COPY . /app
    
RUN cd discord-bot; cargo build --release


FROM debian:bullseye-slim

COPY --from=builder /app/target/release/h4tt-discord-bot /app/h4tt-discord-bot

ENTRYPOINT ["/app/h4tt-discord-bot"]