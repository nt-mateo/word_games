# Use a Rust base image
FROM rust:latest as builder

# Create a new empty shell project
RUN USER=root cargo new --bin word_games
WORKDIR /word_games

# Install LiteFS dependencies
# for alpine-based images
RUN apk add ca-certificates fuse3 sqlite
# or for debian/ubuntu-based images
RUN apt-get update -y && apt-get install -y ca-certificates fuse3 sqlite3

COPY --from=flyio/litefs:0.5 /usr/local/bin/litefs /usr/local/bin/litefs
ENTRYPOINT litefs mount

# Copy your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# This build step is to cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy your source tree
COPY ./src ./src

# Build for release
RUN rm ./target/release/deps/word_games*
RUN cargo build --release

# Final base image - align this with the builder image's OS
FROM rust:bookworm

# Copy the build artifact from the build stage
COPY --from=builder /word_games/target/release/word_games .

# Set the startup command to run your binary
ENTRYPOINT ["./word_games"]
