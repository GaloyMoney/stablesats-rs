FROM clux/muslrust:stable AS build
  COPY . /src
  WORKDIR /src
  RUN SQLX_OFFLINE=true cargo build --locked

FROM ubuntu
  RUN apt-get update && apt-get install -y redis-server
  COPY --from=build /src/target/x86_64-unknown-linux-musl/debug/stablesats /usr/local/bin
  USER 1000
  CMD ["stablesats"]
