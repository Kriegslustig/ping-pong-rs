FROM rust:1.30

EXPOSE 1234/udp

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

CMD ./target/release/ping-pong server
