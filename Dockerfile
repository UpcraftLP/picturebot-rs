FROM rust:1.75.0 AS build

WORKDIR /build

RUN apt update \
    && apt install -y \
    musl-dev \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# statically link against openssl
ENV OPENSSL_STATIC=1

ARG RUST_ENVIRONMENT
ENV RUST_ENVIRONMENT=${RUST_ENVIRONMENT:-production}

ARG VERSION
ENV VERSION=${VERSION:-dev}

COPY . .

RUN cargo build --target x86_64-unknown-linux-gnu --release --bins

#FROM gcr.io/distroless/base AS runtime
FROM debian:12.4-slim AS runtime

WORKDIR /app

RUN apt update \
    && apt install -y \
    libssl-dev \
    pkg-config \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /build/target/x86_64-unknown-linux-gnu/release/picturebot .

EXPOSE 3000

ENTRYPOINT ["/app/picturebot"]
