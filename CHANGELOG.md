# [stablesats release v0.1.6](https://github.com/GaloyMoney/stablesats/releases/tag/v0.1.6)


### Bug Fixes

- Default wrapper config to true
- Use rustls-tls-webpki-roots for wss
- Price_app asserts

### Miscellaneous Tasks

- Better config dump output
- Option to report confg on crash
- Switch edge image to ubuntu
- Better error output

# [stablesats release v0.1.5](https://github.com/GaloyMoney/stablesats/releases/tag/v0.1.5)


### Bug Fixes

- Dockerfile.release ca not mkdir

# [stablesats release v0.1.4](https://github.com/GaloyMoney/stablesats/releases/tag/v0.1.4)


### Miscellaneous Tasks

- Add support for sentinal + redis pw
- Put stablesats under /bin in release image

# [stablesats release v0.1.3](https://github.com/GaloyMoney/stablesats/releases/tag/v0.1.3)


### Bug Fixes

- Shared name is taken on crates.io

### Miscellaneous Tasks

- Remove author, about from cli
- Add Dockerfile.release
- Add BUILDTIME + COMMITHASH to docker image

# [stablesats release v0.1.2](https://github.com/GaloyMoney/stablesats/releases/tag/v0.1.2)


### Miscellaneous Tasks

- Complete manifest for Cargo.toml files

# [stablesats release v0.1.1](https://github.com/GaloyMoney/stablesats/releases/tag/v0.1.1)



# [stablesats release v0.1.1](https://github.com/GaloyMoney/stablesats/releases/tag/v0.1.1)



# [stablesats release v0.1.0](https://github.com/GaloyMoney/stablesats/releases/tag/v0.1.0)


### Bug Fixes

- Clippy
- Fmt + clippy
- Move anyhow to dev-deps
- Improve price_feed test
- Remove unwrap() calls
- Snake_case
- Clippy
- Typos
- Style
- Clippy
- Clippy
- More price-server scafolding
- Return pined stream from subscribe
- Clippy
- Commit Cargo.lock as we are buiding a binary
- Missing -
- Add check-code
- Remove --all-features from watch commands
- Clippy

### Features

- Add sell logic
- Initial price cli command
- Cli runs okex-feed
- Cli can run price server
- Okex-price publishes to redis
- Okex exchange pricefeed
- Initial pubsub

### Miscellaneous Tasks

- Bump rust_decimal from 1.25.0 to 1.26.0
- Bump chrono
- PubSubConfig default
- Add bid_price to BTC_USD_TICK
- Remove redundant types
- Cli boilerplate
- Bump anyhow from 1.0.59 to 1.0.60
- No need for _test postfix
- Test err states in price_app
- Price_app_test passing
- Some price-server scaffolding
- Forgott Cargo.lock
- Rename some pubsub stuff
- Bump price-server deps
- More shared types
- Bump docker-compose.yml version
- Rename pubsub channel from dealerv2 to stablesats
- Impl MessagePayload in macro

### Testing

- Add use test cases
