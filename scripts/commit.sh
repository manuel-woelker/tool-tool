#!/usr/bin/env bash

set -euo pipefail

export RUSTFLAGS='-D warnings'

cargo fmt
cargo clippy -- -D warnings
cargo test

jj desc
jj new
git push origin HEAD:refs/heads/master
git checkout master
git pull