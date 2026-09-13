FROM node:22-bookworm-slim AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates build-essential \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH="/usr/local/cargo/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs --output /tmp/rustup-init.sh \
    && sh /tmp/rustup-init.sh -y --profile minimal --default-toolchain 1.97.1 --target wasm32-unknown-unknown \
    && rm /tmp/rustup-init.sh

RUN cargo install trunk --locked

WORKDIR /app

COPY frontend/package.json frontend/pnpm-lock.yaml frontend/pnpm-workspace.yaml frontend/
COPY shared/Cargo.toml shared/Cargo.toml
COPY shared/Cargo.lock shared/Cargo.lock
COPY frontend/Cargo.toml frontend/Cargo.toml
COPY frontend/Cargo.lock frontend/Cargo.lock

RUN corepack enable && corepack prepare pnpm@latest --activate

WORKDIR /app/frontend
RUN pnpm install --frozen-lockfile

WORKDIR /app
RUN mkdir -p frontend/src && echo "fn main(){}" > frontend/src/main.rs \
    && mkdir -p shared/src && echo "" > shared/src/lib.rs

WORKDIR /app/frontend
RUN cargo build --release --locked --target wasm32-unknown-unknown

WORKDIR /app
COPY shared/src shared/src
COPY frontend/src frontend/src
COPY frontend/index.html frontend/
COPY frontend/style frontend/style
COPY frontend/public frontend/public

WORKDIR /app/frontend
RUN find ../shared/src src -name '*.rs' -type f -exec touch {} \; \
    && pnpm exec tailwindcss -i style/input.css -o style/output.css --minify \
    && trunk build --release

FROM nginx:alpine
COPY --from=builder /app/frontend/dist /usr/share/nginx/html
EXPOSE 80