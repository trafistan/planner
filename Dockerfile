# 1) Build aşaması: musl target’ı ekleyip statik derliyoruz3
FROM rust:1.78-slim AS builder
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app
COPY Cargo.toml ./
COPY src/ ./src/

# cache için önce manifest’i derle
RUN cargo build --release --target x86_64-unknown-linux-musl

# 2) Runtime aşaması: scratch, hiçbir sistem kütüphanesine bağlı değil
FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/planner /planner
ENTRYPOINT ["/planner"]