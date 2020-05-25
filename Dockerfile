# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:latest as cargo-build

RUN apt-get update

WORKDIR /usr/src/spin-archive

COPY Cargo.toml Cargo.toml

RUN mkdir src/

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

COPY rust-toolchain .

RUN cargo build --release

RUN rm -f target/release/deps/spin-archive*

COPY src .
COPY templates .
COPY migrations .

RUN cargo build --release

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

FROM cargo-build

WORKDIR /home/spin-archive/bin/

COPY --from=cargo-build /usr/src/spin-archive/target/release/spin-archive .
COPY --from=cargo-build /usr/src/spin-archive/templates ./templates
COPY --from=node-build /usr/src/spin-archive/build ./build

CMD ["./spin-archive"]