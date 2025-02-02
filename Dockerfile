FROM rust:1.84-alpine AS builder

RUN apk add --no-cache libgcc openssl-dev musl-dev
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release 

FROM alpine:latest AS runner
RUN apk add --no-cache libgcc openssl
COPY --from=builder /usr/src/app/target/release/api /usr/local/bin/api

EXPOSE 8000
CMD ["api"]
