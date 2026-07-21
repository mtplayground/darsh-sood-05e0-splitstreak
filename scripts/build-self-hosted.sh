#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
OUT_DIR=${SELF_HOSTED_DIST_DIR:-"$ROOT_DIR/dist/self-hosted"}

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR/bin" "$OUT_DIR/frontend"

npm run build --workspace frontend
cargo build --manifest-path "$ROOT_DIR/backend/Cargo.toml" --release

cp "$ROOT_DIR/backend/target/release/splitstreak-api" "$OUT_DIR/bin/"
cp -R "$ROOT_DIR/frontend/dist/." "$OUT_DIR/frontend/"
cp "$ROOT_DIR/.env.example" "$OUT_DIR/.env.example"
cp "$ROOT_DIR/scripts/run-self-hosted.sh" "$OUT_DIR/run-self-hosted.sh"
chmod +x "$OUT_DIR/run-self-hosted.sh"

printf 'Self-hosted build written to %s\n' "$OUT_DIR"
