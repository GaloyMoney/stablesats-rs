# [stablesats release v0.9.7](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.9.7)


### Miscellaneous Tasks

- Rename proto file (#370)

# [stablesats release v0.9.6](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.9.6)


### Bug Fixes

- Update Cargo.lock
- Okex get position error on empty api response (#365)

# [stablesats release v0.9.5](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.9.5)


### Bug Fixes

- Okex get position error when zero position (#361)

# [stablesats release v0.9.4](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.9.4)


### Bug Fixes

- Cargo.lock

### Miscellaneous Tasks

- Bump sqlx-ledger
- Cargo update (#342)
- Bump deps and fix vulnerabilities
- Bump serde from 1.0.157 to 1.0.158 (#331)
- Bump clap from 4.1.9 to 4.1.11 (#329)
- Bump reqwest from 0.11.14 to 0.11.15 (#332)

# [stablesats release v0.9.3](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.9.3)


### Miscellaneous Tasks

- Bump async-trait from 0.1.66 to 0.1.67 (#324)
- Bump serde from 1.0.156 to 1.0.157 (#328)
- Bump thiserror from 1.0.39 to 1.0.40 (#325)
- Bump anyhow from 1.0.69 to 1.0.70 (#326)
- Bump rust_decimal_macros from 1.28.1 to 1.29.0 (#327)
- Bump rust_decimal from 1.28.1 to 1.29.0 (#322)
- Bump clap from 4.1.8 to 4.1.9 (#323)

# [stablesats release v0.9.2](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.9.2)


### Bug Fixes

- Zero exposure from api response (#321)

### Miscellaneous Tasks

- Bump futures from 0.3.26 to 0.3.27 (#317)
- Bump chrono from 0.4.23 to 0.4.24 (#319)
- Bump serde from 1.0.152 to 1.0.156 (#318)
- Bump axum from 0.6.9 to 0.6.11 (#320)
- Bump serde_with from 2.2.0 to 2.3.1 (#316)

# [stablesats release v0.9.1](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.9.1)


### Bug Fixes

- Okx transfer client id not found (#314)

# [stablesats release v0.9.0](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.9.0)


### Bug Fixes

- Handling of okx error 58129 (#313)

### Miscellaneous Tasks

- Patch vulnerability
- Bump axum-core from 0.3.2 to 0.3.3 (#305)
- Bump serde_yaml from 0.9.17 to 0.9.19 (#307)
- Bump thiserror from 1.0.38 to 1.0.39 (#308)
- Bump async-trait from 0.1.64 to 0.1.66 (#309)
- Add features to tokio in okex-client

# [stablesats release v0.8.2](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.8.2)


### Bug Fixes

- Price server should handle 0 amounts

### Miscellaneous Tasks

- Remove redis-server from dev Dockerfile

# [stablesats release v0.8.1](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.8.1)


### Documentation

- Remove old migration notice

### Miscellaneous Tasks

- Output version when starting daemon
- Bump axum from 0.6.8 to 0.6.9 (#303)
- Bump tokio from 1.25.0 to 1.26.0 (#302)
- Bump sqlx-ledger from 0.5.2 to 0.5.5 (#304)
- Bump axum from 0.6.6 to 0.6.8 (#297)
- Bump prost from 0.11.6 to 0.11.8 (#298)
- Bump clap from 4.1.6 to 4.1.8 (#300)

# [stablesats release v0.8.0](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.8.0)


### Bug Fixes

- [**breaking**] Re-import galoy-transactions (#295)

# [stablesats release v0.7.1](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.7.1)


### Bug Fixes

- New id for static POLL_OKEX job

### Miscellaneous Tasks

- Early return funding if okex is simulating

# [stablesats release v0.7.0](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.7.0)


### Miscellaneous Tasks

- Remove shared exchanges_config.rs (#292)
- [**breaking**] Pause kollider integration (#291)
- Bump axum from 0.6.4 to 0.6.6 (#286)
- Bump sqlx-ledger from 0.5.1 to 0.5.2 (#287)
- Bump clap from 4.1.4 to 4.1.6 (#290)

### Refactor

- [**breaking**] Restructure config and hedging module (#285)

# [stablesats release v0.6.6](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.6.6)


### Bug Fixes

- Span should be reference (#288)

# [stablesats release v0.6.5](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.6.5)


### Miscellaneous Tasks

- Disable bitfinex-price-feed by default
- Output sqlx_ledger spans
- Log output as json and filter out sqlx (#284)

# [stablesats release v0.6.4](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.6.4)


### Bug Fixes

- Typo
- Host name in db url

### Miscellaneous Tasks

- Bump sqlx-ledger

### Testing

- Fix DATABASE_URL in makefile
- Use same pg con as sqlx-ledger to improve local dev

# [stablesats release v0.6.3](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.6.3)


### Bug Fixes

- Use while not loop for health check
- Do not expect on health check

### Miscellaneous Tasks

- More resillient health check

# [stablesats release v0.6.2](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.6.2)


### Miscellaneous Tasks

- Improved job spawning from hedging app
- Bump serde_json from 1.0.92 to 1.0.93 (#283)

# [stablesats release v0.6.1](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.6.1)


### Bug Fixes

- Sqlx ledger initialization (#282)

### Miscellaneous Tasks

- Better account description

# [stablesats release v0.6.0](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.6.0)


### Features

- [**breaking**] Use ledger to trigger adjustment jobs (#281)

### Miscellaneous Tasks

- Bump rust_decimal from 1.28.0 to 1.28.1 (#278)

# [stablesats release v0.5.6](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.5.6)


### Bug Fixes

- Okex.check_leverage missing params

# [stablesats release v0.5.5](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.5.5)


### Bug Fixes

- Leverage check (#275)

### Features

- Add sqlx ledger (#276)

### Miscellaneous Tasks

- Bump rust_decimal_macros from 1.28.0 to 1.28.1 (#279)
- Bump serde_json from 1.0.91 to 1.0.92 (#280)
- Bump anyhow from 1.0.68 to 1.0.69 (#277)
- Bump uuid from 1.2.2 to 1.3.0 (#274)
- Bump futures from 0.3.25 to 0.3.26 (#272)
- Bump async-trait from 0.1.63 to 0.1.64 (#271)
- Bump tokio from 1.24.2 to 1.25.0 (#269)

# [stablesats release v0.5.4](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.5.4)


### Miscellaneous Tasks

- Replace mid price ratio with last price (#270)

# [stablesats release v0.5.3](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.5.3)


### Bug Fixes

- DoNothing instead of ClosePosition

### Miscellaneous Tasks

- Clippy
- Simplify listening to balance updates

# [stablesats release v0.5.2](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.5.2)


### Bug Fixes

- Pin channel_name to job runner per module

### Miscellaneous Tasks

- Clippy

# [stablesats release v0.5.1](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.5.1)


### Bug Fixes

- Remove unique constraint (#266)
- Typo in Makefile
- Remove keys from docker-compose.override.yml

# [stablesats release v0.5.0](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.5.0)


### Features

- [**breaking**] Ensure standalone price server (#264)

### Miscellaneous Tasks

- Bump axum from 0.6.2 to 0.6.4 (#258)
- Bump clap from 4.1.1 to 4.1.4 (#259)

# [stablesats release v0.4.5](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.4.5)


### Miscellaneous Tasks

- Do not expect job data to be present
- Better job execution visibility
- Remove expects in JobExecutor

# [stablesats release v0.4.4](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.4.4)


### Miscellaneous Tasks

- Reduce retries increase connections

# [stablesats release v0.4.3](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.4.3)


### Bug Fixes

- Wipe 0 attempt static jobs on startup

### Miscellaneous Tasks

- Fix adjust_hedge retries
- Improve poll_galoy_transactions retries

# [stablesats release v0.4.2](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.4.2)


### Miscellaneous Tasks

- More retries for critical jobs

# [stablesats release v0.4.1](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.4.1)


### Bug Fixes

- Remove bitfinex from hedging + more resilient health check
- Non-overlapping job ids between user-trades and hedging
- Disable bitfinex price by default
- Remove comment

### Miscellaneous Tasks

- Set correct channel_name for adjust_funding

### Testing

- Remove bitfinex from hedging test

# [stablesats release v0.4.0](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.4.0)


### Bug Fixes

- Typo in tests/price_app

### Features

- Bitfinex client  (#229)
- Bitfinex price (#247)

### Miscellaneous Tasks

- Default weight is 1.0
- Bump async-trait from 0.1.61 to 0.1.63 (#252)
- Bump serde_yaml from 0.9.16 to 0.9.17 (#254)
- Bump rust_decimal_macros from 1.27.0 to 1.28.0 (#255)
- Bump axum-core from 0.3.1 to 0.3.2 (#256)
- Comment bitfinex vars in docker-compose
- Remove redundant galoy-client query
- Delete unused graphql
- Bump reqwest from 0.11.13 to 0.11.14 (#250)
- Bump tokio from 1.24.1 to 1.24.2 (#248)
- Bump serial_test from 0.10.0 to 1.0.0 (#249)
- Bump clap from 4.0.32 to 4.1.1 (#246)
- Bump graphql_client from 0.11.0 to 0.12.0 (#245)
- Bump axum from 0.6.1 to 0.6.2 (#244)
- Bump prost from 0.11.5 to 0.11.6 (#243)
- Bump tokio from 1.24.0 to 1.24.1 (#239)
- Bump async-trait from 0.1.60 to 0.1.61 (#240)
- Bump serde_with from 2.1.0 to 2.2.0 (#241)
- Bump axum-core from 0.3.0 to 0.3.1 (#242)
- Bump async-trait from 0.1.58 to 0.1.60 (#236)
- Bump tokio from 1.23.0 to 1.24.0 (#237)
- Bump prost from 0.11.3 to 0.11.5 (#226)
- Bump clap from 4.0.29 to 4.0.32 (#231)
- Bump serde from 1.0.151 to 1.0.152 (#233)

### Refactor

- [**breaking**] Introduce single unified database (#251)
- Rename ExchangeConfigs

### Testing

- Ignore hedging - too unstable
- Ignore okex tests
- Pass withouth BITFINEX creds

# [stablesats release v0.3.27](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.27)


### Bug Fixes

- Await timestamp_sender for health check

# [stablesats release v0.3.26](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.26)


### Miscellaneous Tasks

- Dev mock price is configured directly in price_cache

# [stablesats release v0.3.25](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.25)


### Miscellaneous Tasks

- Add mock dev price
- Switch price feed to in-memory pubsub (#230)

# [stablesats release v0.3.24](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.24)


### Miscellaneous Tasks

- Nest health config for hedging

# [stablesats release v0.3.23](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.23)


### Miscellaneous Tasks

- Add aditional context to health check message

# [stablesats release v0.3.22](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.22)


### Miscellaneous Tasks

- Fix stablesats.yml

### Testing

- Attempt to avoid rate limit

# [stablesats release v0.3.21](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.21)


### Refactor

- Configure stale_after & last_msg_delay (#225)

# [stablesats release v0.3.20](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.20)


### Miscellaneous Tasks

- Bump anyhow from 1.0.66 to 1.0.68 (#221)
- Bump serde_yaml from 0.9.14 to 0.9.16 (#220)
- Bump thiserror from 1.0.37 to 1.0.38 (#222)
- Bump serial_test from 0.9.0 to 0.10.0 (#223)
- Bump serde_json from 1.0.89 to 1.0.91 (#224)

# [stablesats release v0.3.19](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.19)


### Miscellaneous Tasks

- Increase stale price duration
- Bump tokio from 1.22.0 to 1.23.0 (#215)
- Bump data-encoding from 2.3.2 to 2.3.3 (#217)
- Bump serde from 1.0.148 to 1.0.150 (#219)

# [stablesats release v0.3.18](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.18)


### Bug Fixes

- Use explicit is_empty() for clarity (#206)

### Miscellaneous Tasks

- Limit galoy tx poll to 100
- Bump derive_builder from 0.11.2 to 0.12.0 (#213)
- Bump tonic-build from 0.8.2 to 0.8.4 (#207)
- Bump clap from 4.0.27 to 4.0.29 (#208)
- Bump tokio-tungstenite from 0.17.2 to 0.18.0 (#209)
- Bump axum from 0.6.0 to 0.6.1 (#210)
- Bump governor from 0.5.0 to 0.5.1 (#211)
- Bump axum-core from 0.2.9 to 0.3.0 (#202)
- Bump axum from 0.5.17 to 0.6.0 (#201)
- Bump tonic from 0.8.2 to 0.8.3 (#204)
- Bump serde from 1.0.147 to 1.0.148 (#203)
- Bump prost from 0.11.2 to 0.11.3 (#205)

# [stablesats release v0.3.17](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.17)


### Miscellaneous Tasks

- Run cargo update
- Better okex position healthcheck and order err return
- Add tracing to okex-client
- Bump serde_json from 1.0.88 to 1.0.89 (#198)
- Bump position subscription healthy duration
- Instrument poll_okex entrypoint
- Increase poll_okex retries
- Add debug msg (#194)
- Bump serde_json from 1.0.87 to 1.0.88 (#192)
- Bump tokio from 1.21.2 to 1.22.0 (#193)
- Bump reqwest from 0.11.12 to 0.11.13 (#189)
- Bump clap from 4.0.25 to 4.0.26 (#190)
- Bump serde_with from 2.0.1 to 2.1.0 (#191)
- Bump clap from 4.0.24 to 4.0.25 (#188)
- Bump uuid from 1.2.1 to 1.2.2 (#187)
- Bump clap from 4.0.23 to 4.0.24 (#186)
- Bump chrono from 0.4.22 to 0.4.23 (#182)
- Bump clap from 4.0.22 to 4.0.23 (#183)

### Testing

- Adjust timing for hedging test to run better

# [stablesats release v0.3.16](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.16)


### Bug Fixes

- Implement ref-to-ref conversion for order book (#178)
- Typo in misconfigured account error (#175)

### Features

- Weighted price from order book cache (#181)
- Stream and publish okex order book (#176)
- Kollider-integration (#173)

### Miscellaneous Tasks

- Bump prost from 0.11.0 to 0.11.2 (#179)
- Bump clap from 4.0.18 to 4.0.22 (#180)

### Testing

- Ignore kollider price test

# [stablesats release v0.3.15](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.15)


### Bug Fixes

- Use 'delayed' fee for get_cents_from_sats_for_future_sell (#174)

### Documentation

- Update readme
- Stablesats developer documentation (#154)

### Miscellaneous Tasks

- Skip reporting config in trace
- Bump serde_yaml from 0.9.13 to 0.9.14 (#169)
- Bump axum from 0.5.16 to 0.5.17 (#170)
- Bump serde from 1.0.145 to 1.0.147 (#171)
- Bump futures from 0.3.24 to 0.3.25 (#165)
- Bump clap from 4.0.17 to 4.0.18 (#166)
- Bump axum-core from 0.2.8 to 0.2.9 (#167)
- Bump anyhow from 1.0.65 to 1.0.66 (#168)
- Bump serde_json from 1.0.86 to 1.0.87 (#164)
- Bump clap from 4.0.16 to 4.0.17 (#163)
- Bump clap from 4.0.15 to 4.0.16 (#162)
- Bump clap from 4.0.14 to 4.0.15 (#160)

# [stablesats release v0.3.14](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.14)


### Bug Fixes

- No need for optimistic concurrency when fetching galoy txs

### Miscellaneous Tasks

- Bump clap from 4.0.13 to 4.0.14 (#158)

# [stablesats release v0.3.13](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.13)


### Bug Fixes

- Only complete job when it did not error

# [stablesats release v0.3.12](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.12)


### Bug Fixes

- Okex client balance data fields (#157)
- Set min retry to 5s after polling galoy txs

### Miscellaneous Tasks

- Bump clap from 4.0.12 to 4.0.13 (#156)
- Bump uuid from 1.1.2 to 1.2.1 (#152)
- Bump clap from 4.0.11 to 4.0.12 (#155)
- Use recomended distroless base image
- Bump clap from 4.0.10 to 4.0.11 (#151)
- Bump serde_json from 1.0.85 to 1.0.86 (#153)

# [stablesats release v0.3.11](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.11)


### Bug Fixes

- Galoy client error return (#142)

### Miscellaneous Tasks

- Bump tracing from 0.1.36 to 0.1.37 (#149)
- Bump tracing-subscriber from 0.3.15 to 0.3.16 (#150)
- Bump clap from 4.0.8 to 4.0.10 (#148)

# [stablesats release v0.3.10](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.10)


### Bug Fixes

- Inject trace to all request & fix double request client (#145)
- Poll_galoy_transactions job was not retrying

# [stablesats release v0.3.9](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.9)


### Features

- Inject tracing headers (#138)

### Miscellaneous Tasks

- Bump clap from 4.0.4 to 4.0.8 (#141)

# [stablesats release v0.3.8](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.8)


### Miscellaneous Tasks

- Breaking clap upgrade
- Bump patches
- Set output formatting to json (#140)

# [stablesats release v0.3.7](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.7)


### Bug Fixes

- Declare checkpoint_json in execute_job trace

# [stablesats release v0.3.6](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.6)


### Miscellaneous Tasks

- Add position msg to health check
- Better health check

# [stablesats release v0.3.5](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.5)


### Bug Fixes

- Clippy
- Identify okex service unavailable

### Features

- Throttle price tick publishing (#129)

### Miscellaneous Tasks

- Bump thiserror from 1.0.36 to 1.0.37 (#133)
- Bump tokio from 1.21.1 to 1.21.2 (#132)
- Shared JobExecutor (#131)
- Persist okex orders (#130)
- Bump governor from 0.4.2 to 0.5.0 (#120)
- Bump thiserror from 1.0.35 to 1.0.36 (#128)

# [stablesats release v0.3.4](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.4)


### Bug Fixes

- Clippy

### Miscellaneous Tasks

- Wire price + hedging health check
- Report Level::ERROR after 4 retries
- Record error.level in error traces
- Some health check boilerplate
- Record last_msg_timestamp in subscriber
- Bump OTEL libraries
- Bump serde from 1.0.144 to 1.0.145
- Bump reqwest from 0.11.11 to 0.11.12

# [stablesats release v0.3.3](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.3)


### Bug Fixes

- Pairing of ln user-trades

### Miscellaneous Tasks

- Bump clap from 3.2.21 to 3.2.22

# [stablesats release v0.3.2](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.2)


### Miscellaneous Tasks

- Deduplicate adjust_hedge job
- Dedup adjust_hedge job

# [stablesats release v0.3.1](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.1)


### Bug Fixes

- Clippy
- Handle bootstrapping large no of txs

### Miscellaneous Tasks

- Remove redundant code
- Use push_tuples

# [stablesats release v0.3.0](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.3.0)


### Miscellaneous Tasks

- [**breaking**] Update sqlx-data and deps
- Remove redundant code
- Add update_paired_ids
- Add galoy_transactions table

# [stablesats release v0.2.4](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.2.4)


### Bug Fixes

- Contract size was in usd (instead of cents)
- Minimum liability threshold to dealer v1

### Miscellaneous Tasks

- No comments
- Newline eof, discard test data
- Removing duplicate test
- Add error recording to all publish calls

### Testing

- Use the original hedging integration test
- Add dealer v1 scenario based integration test

# [stablesats release v0.2.3](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.2.3)


### Bug Fixes

- More robust listen to user trades notification
- Ensure poll jobs do not run out of retries

### Miscellaneous Tasks

- Track poll attempts per job
- Default to 20 retries (~1 week)
- Add .gitignore to hedging
- Bump sqlx from 0.6.1 to 0.6.2
- Bump serde_yaml from 0.9.11 to 0.9.13

# [stablesats release v0.2.2](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.2.2)


### Bug Fixes

- Tests passing
- Some issues from initial run

# [stablesats release v0.2.1](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.2.1)


### Miscellaneous Tasks

- Whitespace for release

# [stablesats release v0.2.0](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.2.0)


### Bug Fixes

- Sqlx for release compiling
- Typo in stablesats.yml
- Use rust-tls in galoy-client login
- Force use_rustls on clients
- Error test in price_app
- Deserialize Duration with serde_with
- Security upgrade
- Stablesats.yml defaults doc
- User_trade_balances bug
- Clippy
- Galoy-client handles cursor
- Clippy in galoy-client and user-trades
- Typo
- Cursos is optional in transactions_list
- Check-code
- Cleanup okex-client for hedging use case
- Serialize insert-if-new
- Add SQLX_OFFLINE=true to Dockerfile
- Correct position typo and deposit_status test
- Construct okex client if position mode is set to "net_mode"

### Documentation

- Typo in example stablesats.yml

### Features

- Adding hysteresis around hedging actions
- Add hedging to cli
- Retrieve onchain transaction fee
- Send onchain payment
- Create onchain deposit address
- Impl try_from for transactions_list and wallets
- Retrieve btc and usd wallet balances
- Return stream of transactions
- Get btc and usd transactions list
- Retrieve default btc and usd wallets
- Get transactions list for stablesats account
- Login to wallet account
- Send auth code to wallet phone number
- Scaffold galoy wallet library
- Add hedging_adjustments table to record actions
- Hedging boilerplate
- User-trades crate

### Miscellaneous Tasks

- Patch upgrades
- Bump anyhow from 1.0.64 to 1.0.65
- Bump thiserror from 1.0.34 to 1.0.35
- Remove default-features for reqwest
- Fix cert validation for reqwest calls
- Improve error outupt
- Patch bump deps
- Bump url from 2.3.0 to 2.3.1
- Bump tonic from 0.8.0 to 0.8.1
- Refactor test to use helper functions
- Use sig exposure to handle neg feedback loop
- Some tracing in hedging
- Use record_error in price-server
- More user trade tracing
- Improve user_trades insert order
- Fixes for tests
- Record error and make is_latest optional
- Add some tracing to user-trades
- Wire config for user-trades to cli
- Implement transaction unification
- Explicit translation of GaloyTransaction
- Unify WIP
- Bump url from 2.2.2 to 2.3.0
- Extend user_trades repo
- Remove GaloyTransactions table
- Some boilerplate for user_trade/galoy_transactions
- Clean up transactions list
- Ignore onchain payment and tx fee tests
- Sqlxmq setup in user-trades
- Cleanup some galoy-client types
- Bump thiserror from 1.0.33 to 1.0.34
- Bump protobuf-src from 1.0.5+3.19.3 to 1.1.0+21.5
- Bump anyhow from 1.0.63 to 1.0.64
- Bump serde_yaml from 0.9.10 to 0.9.11
- Decouple graphql url from environment
- Jwt, transaction list variables, tokio macros
- Decouple environment from environment variable names
- Remove reqwest-blocking and toggle tokio test-util
- Remove sensitive environment variables
- Rename galoy-wallet to galoy-client
- Pass correlation_id to adjust_hedge / extract shared::tracing
- Add okex polling to hedge
- Create job in transaction
- Adjustment_action (and lots more)
- Rename exposure -> liability
- Bump fred from 5.1.0 to 5.2.0
- Bump clap from 3.2.19 to 3.2.20
- Bump clap from 3.2.18 to 3.2.19
- Bump thiserror from 1.0.32 to 1.0.33
- Bump anyhow from 1.0.62 to 1.0.63
- Bump futures from 0.3.23 to 0.3.24
- Bump clap from 3.2.17 to 3.2.18
- Bump serde_yaml from 0.9.9 to 0.9.10
- Add order type and margin mode to config
- Bump serde_json from 1.0.83 to 1.0.85
- Bump serde from 1.0.143 to 1.0.144

### Refactor

- Simplify UserTrade structs and fields
- Remove wallet id from 'onchain-x' methods + cleanup
- Rename client_configuration
- Deserialize timestamp integer to chrono type
- Replace f64 and u32 with Decimal
- Remove redundant UnknownResponse error
- Return vector of transactions
- Move error to src/error
- Type aliases for long query-struct names
- Reimplement galoy client constructor
- Add Stablesats prefix to gql names
- Remove redundant btc_price query
- Extract wallet configuration into a function
- Complete job in job execution
- Use dec! in adjustment_action test
- UserTradeUnit in pg table + single error file
- Make new-balance.sql more efficient
- Extract balance update into single query
- Enumerate trade currency & remove from parameters
- Remove margin_mode, position_side, & order_type from client config
- Rename create to new & rename misconfiguration error

### Testing

- Fix tx list and hedging
- Remove tests against dev galoy backend
- Refactor hedging to assert on OkexPosition msg
- External position change is acted on
- Hedging test working in ci
- Hedging e2e working locally

# [stablesats release v0.1.11](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.1.11)


### Bug Fixes

- Configure position mode and side in client constructor

### Miscellaneous Tasks

- Move shared/currency to price-server
- Remove more unused
- More unused
- Remove unused
- Derive Eq alongside PartialEq

### Refactor

- Replace type casting with explicit type conversion

### Testing

- Verify rounding of tiny amounts

# [stablesats release v0.1.10](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.1.10)


### Bug Fixes

- Fee-calculator test
- Fmt
- When to increase / decrease price calc
- Fee-calculation is direction dependent
- Run position tests if environment variables are loaded
- Open a position to close
- Allow clippy issues in generated proto code
- Remove accidentally committed notes
- Use Decimal in deserialization and optionally run tests
- Use cach_bal in available_balance
- Fix flagged typo on 'controling'
- Fetch onchain btc address from environment variables
- Clippy update

### Features

- Close all positions of an instrument
- Open and get position on "btc-usd-swap" instrument
- Deposit status, floating point, & btc address
- Place order and get positions
- Withdraw to onchain BTC address
- Fetch available trading account balance
- State of transfer from funding to trading account
- Fetch available balance of funding account
- Transfer from trading to funding account

### Miscellaneous Tasks

- Create Decimal values with dec!()
- Use rate-limit in deposit-history
- Add rate-limit to okex-client
- Bump anyhow from 1.0.61 to 1.0.62
- Merge branch 'main' into feat-okex-client-library

### Refactor

- Extract client/primitives
- Hardcode demo client api keys
- Change config simulated property to bool
- Implement Display for enums
- Remove superfluous enum implementations
- Enumerate order side
- Enumerate position side
- Enumerate margin mode
- Enumerate instrument id and rate limit api calls
- Deduplicate client test
- Extract get/post header creation
- Extract 'withdraw to btc onchain address' response data
- Extract 'transfer state' response data
- Extract 'trading account balance' response data
- Extract 'funding account balance' response data
- Extract 'trading-to-funding' response data
- Use generics

# [stablesats release v0.1.9](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.1.9)


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

# [stablesats release v0.1.8](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.1.8)


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

# [stablesats release v0.1.7](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.1.7)


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

# [stablesats release v0.1.6](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.1.6)


### Bug Fixes

- Default wrapper config to true
- Use rustls-tls-webpki-roots for wss
- Price_app asserts

### Miscellaneous Tasks

- Better config dump output
- Option to report confg on crash
- Switch edge image to ubuntu
- Better error output

# [stablesats release v0.1.5](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.1.5)


### Bug Fixes

- Dockerfile.release ca not mkdir

# [stablesats release v0.1.4](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.1.4)


### Miscellaneous Tasks

- Add support for sentinal + redis pw
- Put stablesats under /bin in release image

# [stablesats release v0.1.3](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.1.3)


### Bug Fixes

- Shared name is taken on crates.io

### Miscellaneous Tasks

- Remove author, about from cli
- Add Dockerfile.release
- Add BUILDTIME + COMMITHASH to docker image

# [stablesats release v0.1.2](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.1.2)


### Miscellaneous Tasks

- Complete manifest for Cargo.toml files

# [stablesats release v0.1.1](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.1.1)



# [stablesats release v0.1.1](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.1.1)



# [stablesats release v0.1.0](https://github.com/GaloyMoney/stablesats-rs/releases/tag/0.1.0)


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
