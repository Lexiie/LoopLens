FROM rust:1-bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release -p looplens-service

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/looplens-service /usr/local/bin/looplens-service
COPY .looplens ./.looplens

ENV LOOPLENS_ROOT=/app
EXPOSE 8787

CMD ["looplens-service"]

