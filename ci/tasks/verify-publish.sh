#!/bin/bash

cargo install --all-features stablesats
if [[ $(stablesats --version) != "cepler $(cat version/version)" ]]; then
  echo "Installed stablesats does not have expected version number"
  exit 1
fi
stablesats help
