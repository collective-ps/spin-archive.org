# ------------------------------------------------------------------------------
# Rust Stage
# ------------------------------------------------------------------------------

FROM rust:1-bookworm AS builder
WORKDIR /app

# Install the pinned nightly toolchain in its own layer.
COPY rust-toolchain .
RUN rustup show

# Build dependencies against a stub main so they're cached between deploys.
# build.rs compiles the ructe templates, so those come along too.
COPY Cargo.toml Cargo.lock ./
COPY src/build.rs src/build.rs
COPY templates templates
RUN echo "fn main() {}" > src/main.rs \
    && cargo build --release --bin spin-archive \
    && rm -rf target/release/spin-archive target/release/deps/spin_archive-* target/release/.fingerprint/spin-archive-*

COPY . .
RUN touch src/main.rs && cargo build --release --bin spin-archive

# ------------------------------------------------------------------------------
# Front-end Assets Stage
# ------------------------------------------------------------------------------

FROM node:14-alpine AS node-build

WORKDIR /usr/src/spin-archive

COPY package.json package-lock.json webpack.config.js postcss.config.js ./

RUN npm install

COPY assets assets

RUN npm run build

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/spin-archive .
COPY --from=node-build /usr/src/spin-archive/build ./build

ENV ROCKET_ENV=production \
    ROCKET_ADDRESS=0.0.0.0 \
    ROCKET_PORT=8080 \
    DATABASE_PATH=/data/spin-archive.db

EXPOSE 8080

CMD ["./spin-archive"]
