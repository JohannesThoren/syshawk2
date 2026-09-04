# --- build stage ---
FROM rust:1-bookworm AS builder
WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    libclang-dev clang libpam0g-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates/common ./crates/common
COPY crates/probe ./crates/probe
COPY crates/server ./crates/server

RUN cargo build --release -p shawk-server

# --- runtime stage ---
FROM debian:bookworm-slim
WORKDIR /app

# libpam0g: talks to the HOST's PAM stack (mounted in at runtime) to
# authenticate dashboard logins against real Linux accounts.
# ca-certificates: for any outbound TLS (none by default, but cheap to have).
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpam0g ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/shawk-server /app/shawk-server
COPY crates/server/migrations /app/migrations

EXPOSE 8080
ENTRYPOINT ["/app/shawk-server"]
