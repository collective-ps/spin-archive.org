# ------------------------------------------------------------------------------
# Rust Stage
# ------------------------------------------------------------------------------

FROM lukemathwalker/cargo-chef as planner
WORKDIR app
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM lukemathwalker/cargo-chef as cacher
WORKDIR app
COPY --from=planner /app/recipe.json recipe.json
COPY rust-toolchain .
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust as builder
WORKDIR app
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release --bin spin-archive

# ------------------------------------------------------------------------------
# Front-end Assets Stage
# ------------------------------------------------------------------------------

FROM node:14-alpine as node-build

WORKDIR /usr/src/spin-archive

COPY package.json package.json
COPY package-lock.json package-lock.json
COPY webpack.config.js webpack.config.js
COPY postcss.config.js postcss.config.js

RUN npm install

COPY assets assets

RUN npm run build

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM rust as runtime

WORKDIR /home/spin-archive/bin/

COPY --from=builder /app/target/release/spin-archive .
COPY --from=node-build /usr/src/spin-archive/build ./build

CMD ["./spin-archive"]