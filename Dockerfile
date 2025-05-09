FROM rust:1.70 as builder
WORKDIR /usr/src/app
COPY . .
# Install build dependencies for ffmpeg and Rust crates
RUN apt-get update && apt-get install -y pkg-config libavcodec-dev libavformat-dev libavutil-dev libavdevice-dev libavfilter-dev libswresample-dev libswscale-dev ffmpeg && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

FROM debian:bullseye-slim
# Install runtime ffmpeg for audio processing
RUN apt-get update && apt-get install -y ffmpeg && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/audio_agent /usr/local/bin/
COPY --from=builder /usr/src/app/.env /usr/local/bin/.env
EXPOSE 3000
CMD ["audio_agent"] 