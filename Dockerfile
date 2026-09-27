# ------------------------------------------------------------------------------
# Rust Stage
# ------------------------------------------------------------------------------
#
# Fly builds on Depot, which persists BuildKit cache mounts between builds. The
# cargo registry (including the slow git crates.io index that this 2021 cargo
# requires) and the whole target/ directory live in cache mounts, so:
#   - a src/ or templates/ change recompiles only the spin-archive crate;
#   - a Cargo.toml/Cargo.lock change recompiles only the crates that changed;
#   - if the cache is ever evicted, cargo simply rebuilds from scratch.
# Because target/ is a cache mount (not part of the image), the binary is copied
# out of it at the end of the RUN.

FROM rust:1-bookworm AS builder
WORKDIR /app

# Fetch the git index with the git CLI rather than cargo's built-in libgit2,
# which is much slower on the huge crates.io index.
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

# Install the pinned nightly toolchain in its own layer.
COPY rust-toolchain .
RUN rustup show

# Only the inputs the Rust build reads. embed_migrations! reads migrations/ and
# build.rs (ructe) compiles templates/ at build time.
COPY Cargo.toml Cargo.lock ./
COPY src src
COPY templates templates
COPY migrations migrations

# Touching main.rs and build.rs forces cargo to rebuild the spin-archive crate and
# rerun its build script every time this step runs, so a stale mtime can never
# leave old templates or migrations in the binary. Dependencies stay cached.
RUN --mount=type=cache,id=spin-archive-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=spin-archive-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=spin-archive-target,target=/app/target,sharing=locked \
    touch src/main.rs src/build.rs \
    && cargo build --release --bin spin-archive \
    && cp target/release/spin-archive /usr/local/bin/spin-archive

# ------------------------------------------------------------------------------
# Front-end Assets Stage (independent of the Rust stage, so BuildKit runs it in
# parallel)
# ------------------------------------------------------------------------------

FROM node:14-alpine AS node-build

WORKDIR /usr/src/spin-archive

COPY package.json package-lock.json ./
RUN --mount=type=cache,id=spin-archive-npm,target=/root/.npm \
    npm install --no-audit

COPY webpack.config.js postcss.config.js ./
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

COPY --from=builder /usr/local/bin/spin-archive .
COPY --from=node-build /usr/src/spin-archive/build ./build

ENV ROCKET_ENV=production \
    ROCKET_ADDRESS=0.0.0.0 \
    ROCKET_PORT=8080 \
    DATABASE_PATH=/data/spin-archive.db

EXPOSE 8080

CMD ["./spin-archive"]
