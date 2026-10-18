# The Palimpseste server and its front end, in one image.
# Dependencies are cached with cargo-chef (https://docs.fly.io/rust/the-basics/cargo-chef).

# Rust matches `mise.toml`; wasm-bindgen-cli matches the wasm-bindgen locked in Cargo.lock.
FROM lukemathwalker/cargo-chef:latest-rust-1.99-bookworm AS chef
WORKDIR /app
RUN rustup target add wasm32-unknown-unknown \
    && cargo install --locked wasm-bindgen-cli --version 0.2.129

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Dependencies only: the cached layers.
RUN cargo chef cook --release --locked -p palimpseste-backend --recipe-path recipe.json \
    && cargo chef cook --release --locked -p palimpseste-wasm --target wasm32-unknown-unknown --recipe-path recipe.json
COPY . .
# The same commands as `mise run wasm:build`.
RUN cargo build --release --locked -p palimpseste-backend \
    && cargo build --release --locked -p palimpseste-wasm --target wasm32-unknown-unknown \
    && wasm-bindgen --target web --out-dir app/frontend/src/wasm \
        target/wasm32-unknown-unknown/release/palimpseste_wasm.wasm

FROM node:24-bookworm-slim AS frontend
WORKDIR /app/frontend
RUN corepack enable && corepack prepare pnpm@10.33.0 --activate
COPY app/frontend/package.json app/frontend/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY app/frontend/ ./
COPY --from=builder /app/app/frontend/src/wasm src/wasm
RUN pnpm run build

FROM debian:bookworm-slim AS runtime
# CA certificates for the HTTPS calls to the OIDC providers.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home --shell /usr/sbin/nologin palimpseste \
    && mkdir /data \
    && chown palimpseste:palimpseste /data
COPY --from=builder /app/target/release/palimpseste-backend /usr/local/bin/palimpseste-backend
COPY --from=frontend /app/frontend/dist /app/frontend
ENV PALIMPSESTE_ADDR=0.0.0.0:3000 \
    PALIMPSESTE_FRONTEND_DIR=/app/frontend \
    PALIMPSESTE_DATABASE=/data/palimpseste.sqlite3
USER palimpseste
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/palimpseste-backend"]
