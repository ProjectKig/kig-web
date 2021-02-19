FROM ekidd/rust-musl-builder:stable as build

# Dependency caching
RUN USER=root cargo new --bin kig-web
WORKDIR ./kig-web
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/x86_64-unknown-linux-musl/release/deps/kig_web*
RUN cargo build --release

FROM alpine:latest
ARG exec=/usr/src/kig-web
ARG user=kig
EXPOSE 3233
RUN groupadd ${user} && useradd -g ${user} ${user} && mkdir -p ${exec}
COPY --from=build /home/rust/src/rust-docker-web/target/x86_64-unknown-linux-musl/release/kig-web ${exec}
RUN chown -R ${user}:${user} ${exec}

USER ${user}
WORKDIR ${exec}

CMD ["./kig-web"]