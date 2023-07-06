FROM rust:1 as builder

WORKDIR /app

COPY . /app

# Install ld
RUN apt-get update && apt-get install -y \
    lld

RUN cd backend; cargo build --release


FROM debian:bullseye-slim

COPY --from=builder /app/target/release/backend /app/backend
COPY backend/run.sh /app/run.sh

# Install openssl
RUN apt-get update && apt-get install -y \
    git-all

ENTRYPOINT ["/app/run.sh"]
