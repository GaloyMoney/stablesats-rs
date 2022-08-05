FROM clux/muslrust:stable AS build
  COPY . /src
  WORKDIR /src
  RUN cargo build --release --locked

FROM alpine:latest
  COPY --from=build /src/target/x86_64-unknown-linux-musl/release/stablesats /bin/
  ARG BUILDTIME
  ARG COMMITHASH
  ENV BUILDTIME ${BUILDTIME}
  ENV COMMITHASH ${COMMITHASH}
  USER 1000
  CMD ["stablesats"]
