#!/bin/bash

set -eu

# ----------- UPDATE REPO -----------
git config --global user.email "bot@galoy.io"
git config --global user.name "CI Bot"

pushd repo

version="$(cat ../version/version)"

for file in $(find . -name Cargo.toml -d 2); do
    sed -i'' "0,/version/{s/version.*/version = \"${VERSION}\"/}" ${file}
    name=$(grep "name = " ${file} | sed 's/name = "\(.*\)"/\1/')
    sed -i'' "/^name = \"${name}/,/version/{s/version.*/version = \"${VERSION}\"/}" ./Cargo.lock
done

git status
git add .

if [[ "$(git status -s -uno)" != ""  ]]; then
  git commit -m "ci(release): release version $(cat ../version/version)"
fi
