# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.91.0
ARG DEBIAN_SUITE=trixie

########################################
# Frontend build (Yew + Trunk)
########################################

FROM rust:${RUST_VERSION}-slim-${DEBIAN_SUITE} AS frontend-builder
WORKDIR /app/frontend
RUN rustup target add wasm32-unknown-unknown && cargo install trunk && \
    apt-get update && apt-get install -y --no-install-recommends binaryen && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_DIR=/app/target
COPY frontend/ .
RUN trunk build --dist /app/dist && \
    set -eux; wasm_file=$(find /app/dist -maxdepth 1 -name '*_bg.wasm' | head -n 1 || true); \
    if [ -n "$wasm_file" ]; then wasm-opt -O --enable-bulk-memory -o "$wasm_file" "$wasm_file"; fi

########################################
# Backend build (Axum)
########################################

FROM rust:${RUST_VERSION}-slim-${DEBIAN_SUITE} AS backend-builder
WORKDIR /app/backend

COPY backend/ .

RUN cargo build --release && \
    cp /app/backend/target/release/pokedex_rncp_backend /app/pokedex_rncp_backend

########################################
# Runtime image
########################################

FROM debian:${DEBIAN_SUITE}-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -u 10001 -r -s /usr/sbin/nologin appuser

WORKDIR /app

ENV BACKEND_URL=0.0.0.0:8080
ENV FRONTEND_ORIGIN=http://localhost:8080

COPY --from=backend-builder /app/pokedex_rncp_backend /app/pokedex_rncp_backend
COPY --from=backend-builder /app/backend/migrations /app/migrations
COPY --from=backend-builder /app/backend/data /app/data
COPY --from=frontend-builder /app/dist /app/static

RUN chown -R appuser:appuser /app
USER appuser
EXPOSE 8080

CMD ["/app/pokedex_rncp_backend"]