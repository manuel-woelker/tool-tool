#!/usr/bin/env bash

set -euo pipefail
LEVEL=${1:-minor}

ROOT_DIR=$(dirname $0)/..
cd $ROOT_DIR

#cargo release -v --workspace --tag-prefix v --all-features $LEVEL
#cargo publish --workspace --all-features
export LEVEL=${1:-minor}
cargo set-version --bump $LEVEL
VERSION=$(cargo pkgid --manifest-path cli/Cargo.toml | cut -d "@" -f2)
echo VERSION: $VERSION
git commit -a -m "chore(release): Release v$VERSION"
git push
git tag -a v$VERSION -m "Release v$VERSION"
git push origin v$VERSION
#cargo publish --workspace --all-features