#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
if [ -x "$SCRIPT_DIR/bin/splitstreak-api" ]; then
  ROOT_DIR=$SCRIPT_DIR
else
  ROOT_DIR=${SELF_HOSTED_DIST_DIR:-"$SCRIPT_DIR/../dist/self-hosted"}
fi

: "${DATABASE_URL:?DATABASE_URL is required}"

export HOST=${HOST:-0.0.0.0}
export PORT=${PORT:-8080}
export STATIC_DIR=${STATIC_DIR:-"$ROOT_DIR/frontend"}

exec "$ROOT_DIR/bin/splitstreak-api" "$@"
