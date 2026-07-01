# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY web ./web
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates curl ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# yt-dlp standalone binary (arch-aware; self-contained, no python needed)
ARG TARGETARCH
RUN case "$TARGETARCH" in \
        amd64) ytdlp=yt-dlp_linux ;; \
        arm64) ytdlp=yt-dlp_linux_aarch64 ;; \
        *)     ytdlp=yt-dlp_linux ;; \
    esac && \
    curl -fsSL "https://github.com/yt-dlp/yt-dlp/releases/latest/download/${ytdlp}" \
        -o /usr/local/bin/yt-dlp && \
    chmod +x /usr/local/bin/yt-dlp

# deno: yt-dlp's default JS runtime for YouTube extraction
COPY --from=denoland/deno:bin /deno /usr/local/bin/deno

COPY --from=builder /app/target/release/radiopal-rs /usr/local/bin/radiopal-rs

WORKDIR /app
ENV RADIOPAL_CONFIG=/config/config.yaml \
    RADIOPAL_STATE_FILE=/data/selector_state.json \
    RADIOPAL_LIQUIDSOAP_ADDR=liquidsoap:1234

ENTRYPOINT ["radiopal-rs"]
