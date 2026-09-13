FROM rust:1-slim AS builder
WORKDIR /app

COPY shared/Cargo.toml shared/Cargo.toml
COPY shared/Cargo.lock shared/Cargo.lock
COPY shared/src shared/src
COPY backend/Cargo.toml backend/Cargo.toml
COPY backend/Cargo.lock backend/Cargo.lock

RUN mkdir -p backend/src && echo "fn main(){}" > backend/src/main.rs \
    && cargo build --release --manifest-path backend/Cargo.toml \
    && rm -rf backend/src backend/target/release/backend backend/target/release/deps/backend-*

COPY backend/src backend/src
RUN touch backend/src/main.rs && cargo build --release --manifest-path backend/Cargo.toml

FROM debian:bookworm-slim
COPY --from=builder /app/backend/target/release/backend /usr/local/bin/backend
EXPOSE 3000
CMD ["backend"]