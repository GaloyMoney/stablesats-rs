#!/bin/bash

set -e

VERSION="$(cat version/version)"

pushd repo

git checkout "${VERSION}"

cat <<EOF | cargo login
${CRATES_API_TOKEN}
EOF

cargo publish --all-features --no-verify --locked
