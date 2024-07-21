FROM alpine:3.19.1 as builder

WORKDIR /build

RUN apk add gcc musl-dev libffi-dev curl openssl-dev openssl openssl-libs-static && \
    curl –proto ‘=https’ –tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -q -y && \
    mkdir -p /build/src

COPY src/main.rs ./src
COPY Cargo.toml .

RUN source $HOME/.cargo/env && \
    arch=$(arch) && \
    cargo build --release --target $arch-unknown-linux-musl && \
    mv /build/target/$arch-unknown-linux-musl/release/trust_ip /build/target

FROM scratch

COPY --from=builder /build/target/trust_ip .

CMD ["/trust_ip"]