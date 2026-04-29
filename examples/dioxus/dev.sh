#!/usr/bin/env bash
set -e
REPO=$(cd "$(dirname "$0")/../.." && pwd)
cd "$REPO/examples/dioxus"
~/.cargo/bin/dx build --platform web
cd "$REPO"
DIOXUS_PUBLIC_PATH="$REPO/target/dx/app/debug/web/public" \
  cargo run -p hyle-dioxus-example
