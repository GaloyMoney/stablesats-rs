# About `stablesats`
Stablesats is a part of the galoy OSS banking stack.
It enables users that deposit Bitcoin to hold a USD denominated value in their wallets.
It achieves this by identifying transactions that involve a hard-coded `dealer` ledger account in the Galoy ledger and calculating a target liability.
This liability is subsequently hedged via shorting perpetual swap contracts on the okex exchange.

## Design

The code is organized into multiple crates.
Some of the crates represent heplers or client libraries for the APIs we depend on and some of them represent logical units that can be run either in isolated processes or together with other units within the same process depending on config settings.

Communication between the (potentially distributed) processes happens via a pubsub system (currently Redis).
Like this we can run multiple copies of the processes to achieve high-availability, fault tolerance and scalability.

The main modules that can be run via the cli are:
- `okex-price`: Module that streams price information from okex onto the pubsub
- `price-server`: Module that exposes a grpc endpoint for clients to get up-to-date price information (cached from the pubsub messages coming from `okex-price`).
- `user_trades`: Module that identifies how much the total usd liability exists in the galoy accounting ledger. It publishes the `SynthUsdLiabilityPayload` message for downstream trading modules to pick up.
- `hedging`: Module that executes trades on okex to match the target liability received from the pubsub.

## Migration to 0.4.0

Pre 0.4.0 there were 2 database connections required.
One for user-trades and one for hedging.
From 0.4 onwards we are only requiring a single postgres database to operate stablesats.
If you are already running with 2 databases you can set the environment variable `MIGRATE_TO_UNIFIED_DB=true` to migrate the 2 existing dbs to a fresh database that will then be unified.
Be sure to remove the flag again on the next startup or else the migration will be attempted again (and fail) due to data already being present.
