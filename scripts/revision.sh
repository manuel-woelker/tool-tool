#!/usr/bin/env bash

set -euo pipefail

LAST_COMMIT_DATE=$(git log -1 --format="%cd" --date=format:"%Y%m%d")
LAST_COMMIT_HASH=$(git log -1 --format="%h")
DIRTY_BIT=$(git diff-index --quiet HEAD || echo "-dev")

echo "${LAST_COMMIT_DATE}-${LAST_COMMIT_HASH}${DIRTY_BIT}"