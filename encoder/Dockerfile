# One-shot video encoder, run as a single-use Fly Machine per upload.
# See encode.sh for the inputs it expects.
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ffmpeg curl ca-certificates jq \
    && rm -rf /var/lib/apt/lists/*

COPY encode.sh /usr/local/bin/encode
RUN chmod +x /usr/local/bin/encode

CMD ["encode"]
