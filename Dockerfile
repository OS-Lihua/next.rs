FROM rust:1.83-slim AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/next /usr/local/bin/next
COPY --from=builder /app/.next .next/

EXPOSE 3000
CMD ["next", "start", "--port", "3000"]
