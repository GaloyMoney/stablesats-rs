#!/bin/bash

set -eu

export digest=$(cat ./latest-image/digest)
export ref=$(cat ./repo/.git/short_ref)

pushd charts-repo

git checkout ${BRANCH}

old_digest=$(yq e '.stablesats.image.digest' "./charts/stablesats/values.yaml")
old_ref=$(grep "digest: \"${old_digest}\"" "./charts/stablesats/values.yaml" \
  | sed -n 's/.*commit_ref=\([^;]*\);.*/\1/p' | tr -d ' \n')

cat <<EOF >> ../body.md
# Bump ${CHARTS_SUBDIR} image

The ${CHARTS_SUBDIR} image will be bumped to digest:
\`\`\`
${digest}
\`\`\`

Code diff contained in this image:

https://github.com/GaloyMoney/stablesats-rs/compare/${old_ref}...${ref}
EOF

gh pr close ${BOT_BRANCH} || true
gh pr create \
  --title "chore(deps) bump-stablesats-image-${ref}" \
  --body-file ../body.md \
  --base ${BRANCH} \
  --head ${BOT_BRANCH} \
  --label galoybot
