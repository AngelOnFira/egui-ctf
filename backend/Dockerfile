FROM rust:1 as builder

WORKDIR /app

COPY . /app

# Install ld
RUN apt-get update && apt-get install -y \
    lld

RUN cd backend; cargo build


FROM debian:bullseye-slim

COPY --from=builder /app/target/release/backend /app/backend

ENTRYPOINT ["/app/backend"]