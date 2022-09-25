# [stablesats release v0.3.4](https://github.com/GaloyMoney/stablesats/releases/tag/v0.3.4)


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

# [stablesats release v0.3.3](https://github.com/GaloyMoney/stablesats/releases/tag/v0.3.3)


### Bug Fixes

- Pairing of ln user-trades

### Miscellaneous Tasks

- Bump clap from 3.2.21 to 3.2.22

# [stablesats release v0.3.2](https://github.com/GaloyMoney/stablesats/releases/tag/v0.3.2)


### Miscellaneous Tasks

- Deduplicate adjust_hedge job
- Dedup adjust_hedge job

# [stablesats release v0.3.1](https://github.com/GaloyMoney/stablesats/releases/tag/v0.3.1)


### Bug Fixes

- Clippy
- Handle bootstrapping large no of txs

### Miscellaneous Tasks

- Remove redundant code
- Use push_tuples

# [stablesats release v0.3.0](https://github.com/GaloyMoney/stablesats/releases/tag/v0.3.0)


### Miscellaneous Tasks

- [**breaking**] Update sqlx-data and deps
- Remove redundant code
- Add update_paired_ids
- Add galoy_transactions table

# [stablesats release v0.2.4](https://github.com/GaloyMoney/stablesats/releases/tag/v0.2.4)


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

# [stablesats release v0.2.3](https://github.com/GaloyMoney/stablesats/releases/tag/v0.2.3)


### Bug Fixes

- More robust listen to user trades notification
- Ensure poll jobs do not run out of retries

### Miscellaneous Tasks

- Track poll attempts per job
- Default to 20 retries (~1 week)
- Add .gitignore to hedging
- Bump sqlx from 0.6.1 to 0.6.2
- Bump serde_yaml from 0.9.11 to 0.9.13

# [stablesats release v0.2.2](https://github.com/GaloyMoney/stablesats/releases/tag/v0.2.2)


### Bug Fixes

- Tests passing
- Some issues from initial run

# [stablesats release v0.2.1](https://github.com/GaloyMoney/stablesats/releases/tag/v0.2.1)


### Miscellaneous Tasks

- Whitespace for release

# [stablesats release v0.2.0](https://github.com/GaloyMoney/stablesats/releases/tag/v0.2.0)


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

# [stablesats release v0.1.11](https://github.com/GaloyMoney/stablesats/releases/tag/v0.1.11)


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

# [stablesats release v0.1.10](https://github.com/GaloyMoney/stablesats/releases/tag/v0.1.10)


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
