FROM ghcr.io/angelonfira/egui-ctf/ctf-backend-base:latest as builder

WORKDIR /app

COPY . /app

RUN cd backend; cargo build --release


FROM debian:bullseye-slim

COPY --from=builder /app/target/release/backend /app/backend
COPY backend/run.sh /app/run.sh

# Install git with no frontend
RUN DEBIAN_FRONTEND=noninteractive apt-get update \
    && apt-get install -y \
        git

ENTRYPOINT ["/app/run.sh"]
