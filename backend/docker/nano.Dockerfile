# syntax=docker/dockerfile:1
# Build context: the backend/ directory.

FROM rust:1-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY . .
# Statically linked musl binary (x86_64-unknown-linux-musl on production hosts).
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    TARGET="$(uname -m)-unknown-linux-musl" \
 && cargo build --release --locked -p nano --target "$TARGET" \
 && cp "target/$TARGET/release/nano" /nano
# The data directory must exist in the image so that a fresh named volume
# inherits its ownership by the unprivileged user.
RUN mkdir /data

FROM scratch
COPY --from=builder /nano /nano
COPY --from=builder --chown=65534:65534 /data /data
ENV DATA_DIR=/data PORT=8080 LOG_FORMAT=json
USER 65534:65534
EXPOSE 8080
# There is no curl in a scratch image: the binary probes its own /health.
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 CMD ["/nano", "healthcheck"]
ENTRYPOINT ["/nano"]
