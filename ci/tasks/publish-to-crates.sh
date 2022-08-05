#!/bin/bash

set -e

VERSION="$(cat version/version)"

pushd repo

git checkout "${VERSION}"

cat <<EOF | cargo login
${CRATES_API_TOKEN}
EOF

cargo publish -p shared --all-features --no-verify --locked
cargo publish -p okex-price --all-features --no-verify --locked
cargo publish -p price-server --all-features --no-verify --locked
cargo publish -p stablesats --all-features --no-verify --locked
