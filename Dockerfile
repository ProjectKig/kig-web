FROM ekidd/rust-musl-builder:stable as build

# Dependency caching
RUN cargo new --bin kig-web
WORKDIR ./kig-web
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/x86_64-unknown-linux-musl/release/deps/kig_web*
USER root 
# Fix permissions for protobuf codegen
RUN chown -R rust:rust src/
USER rust
RUN cargo build --release

FROM alpine:latest
ARG exec=/web
ARG user=kig
EXPOSE 3233
RUN addgroup -S ${user} && adduser -S -g ${user} ${user} && mkdir -p ${exec}
COPY --from=build /home/rust/src/kig-web/target/x86_64-unknown-linux-musl/release/kig-web ${exec}
RUN chown -R ${user}:${user} ${exec}

USER ${user}
WORKDIR ${exec}

VOLUME ["/web/img", "/web/static"]

CMD ["./kig-web"]