FROM rust:1.82-alpine AS builder

WORKDIR /build

RUN apk add musl-dev

COPY src ./src
COPY Cargo.toml .

RUN arch=$(arch) && \
    cargo build --release --target $arch-unknown-linux-musl && \
    mv /build/target/$arch-unknown-linux-musl/release/trust_ip /

FROM scratch

COPY --from=builder /trust_ip .

CMD ["/trust_ip"]