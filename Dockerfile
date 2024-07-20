FROM alpine:3.19.1 as builder

WORKDIR /build

RUN apk add gcc musl-dev libffi-dev curl openssl-dev openssl openssl-libs-static && \
    curl –proto ‘=https’ –tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -q -y && \
    mkdir -p /build/src

COPY src/main.rs ./src
COPY Cargo.toml .

RUN source $HOME/.cargo/env && \
    cargo build --release --target x86_64-unknown-linux-musl

FROM scratch

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/trust_ip .

CMD ["/trust_ip"]