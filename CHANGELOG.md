# [stablesats release v0.1.9](https://github.com/GaloyMoney/stablesats/releases/tag/v0.1.9)


### Bug Fixes

- Fees config test
- Fee defaults
- Buy/sell naming has BTC as base currency

### Documentation

- Stablesats.yml contains tracing key

### Miscellaneous Tasks

- Bump clap from 3.2.16 to 3.2.17
- Bump futures from 0.3.21 to 0.3.23
- Bump serde_yaml from 0.9.4 to 0.9.9
- Bump chrono from 0.4.21 to 0.4.22
- Output when cli cannot connect to price server

### Refactor

- Add extract_response_data fn - simplify types

# [stablesats release v0.1.8](https://github.com/GaloyMoney/stablesats/releases/tag/v0.1.8)


### Bug Fixes

- Price calculation use cases
- Allow for 30 seconds price latency

### Features

- Tracing for e2e galoy demo
- Add fund transfer function to OkexClient
- OkexClient.get_deposit_address

### Miscellaneous Tasks

- Bump anyhow from 1.0.60 to 1.0.61
- Bump chrono from 0.4.20 to 0.4.21
- Url arg is not an arg_enum

### Refactor

- Introduce CurrencyConverter
- Rename transfer => transfer_funder_to_trading
- Move sign_okex_request into headers hepler
- Extract common header creation
- Extract signing function
- Extract OKEX_API_URL

# [stablesats release v0.1.7](https://github.com/GaloyMoney/stablesats/releases/tag/v0.1.7)


### Bug Fixes

- Undo fix in change log
- Path in Dockerfile
- Move fee calculation to domain object
- Remove conversion struct
- Typo in change log md
- Copy from debug folder in Dockerfile

### Miscellaneous Tasks

- Optionaly pass price-server-url
- Do not load fee config from env
- Add price fee calculator
- Refactor ExchangePriceCache and unit test conversion method from u64 to f64
- Bump serde from 1.0.142 to 1.0.143
- Bump rust_decimal from 1.26.0 to 1.26.1
- Bump anyhow from 1.0.59 to 1.0.60
- Move TARGET_x var to osxcross-compile.sh
- Build dev version in regular Dockerfile

### Testing

- Add mid price fn to ExchangePriceCacheInner

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
