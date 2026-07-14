# ---- builder ----
FROM rust:1-bookworm AS builder

RUN rustup target add wasm32-unknown-unknown \
    && cargo install trunk wasm-bindgen-cli

WORKDIR /app
COPY . .

RUN cargo build -p backend --release
RUN cd frontend && trunk build --release

# ---- runtime ----
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/backend ./backend
COPY --from=builder /app/dist ./dist

ENV PORT=8080
ENV DIST_DIR=/app/dist
EXPOSE 8080

CMD ["./backend"]
