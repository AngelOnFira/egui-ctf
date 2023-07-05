FROM rust:1 as builder

WORKDIR /app

COPY . /app
RUN cd backend; cargo build --release


FROM debian:buster-slim

COPY --from=builder /app/target/release/backend /app/backend

ENTRYPOINT ["/app/backend"]