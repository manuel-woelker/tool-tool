#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(dirname $0)/..
cd $ROOT_DIR

export TOOL_TOOL_REVISION=$(./scripts/revision.sh)

cargo build --release

ls -lah target/release/tool-tool.exe