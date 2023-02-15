#!/bin/bash

set -eu

# ----------- UPDATE REPO -----------
git config --global user.email "bot@galoy.io"
git config --global user.name "CI Bot"

pushd repo

VERSION="$(cat ../version/version)"

cat <<EOF >new_change_log.md
# [stablesats release v${VERSION}](https://github.com/GaloyMoney/stablesats-rs/releases/tag/${VERSION})

$(cat ../artifacts/gh-release-notes.md)

$(cat CHANGELOG.md)
EOF
mv new_change_log.md CHANGELOG.md

for file in $(find . -mindepth 2 -name Cargo.toml); do
    sed -i'' "0,/version/{s/version.*/version = \"${VERSION}\"/}" ${file}
    name=$(grep "name = " ${file} | sed 's/name = "\(.*\)"/\1/')
    sed -i'' "/^name = \"${name}/,/version/{s/version.*/version = \"${VERSION}\"/}" ./Cargo.lock
done

git status
git add .

if [[ "$(git status -s -uno)" != ""  ]]; then
  git commit -m "ci(release): release version $(cat ../version/version)"
fi
