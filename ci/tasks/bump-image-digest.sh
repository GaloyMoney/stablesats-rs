#!/bin/bash

set -eu

export digest=$(cat ./latest-image/digest)
export ref=$(cat ./repo/.git/short_ref)
export app_version=$(cat version/version)

pushd charts-repo

yq -i e '.stablesats.image.digest = strenv(digest)' ./charts/stablesats/values.yaml
sed -i "s|\(digest: \"${digest}\"\).*$|\1 # METADATA:: repository=https://github.com/GaloyMoney/stablesats-rs;commit_ref=${ref};app=stablesats;|g" "./charts/stablesats/values.yaml"

yq -i e '.appVersion = strenv(app_version)' ./charts/stablesats/Chart.yaml

if [[ -z $(git config --global user.email) ]]; then
  git config --global user.email "bot@galoy.io"
fi
if [[ -z $(git config --global user.name) ]]; then
  git config --global user.name "CI Bot"
fi

(
  cd $(git rev-parse --show-toplevel)
  git merge --no-edit ${BRANCH}
  git add -A
  git status
  git commit -m "chore(stablesats): bump stablesats image to '${digest}'"
)
