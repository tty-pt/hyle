#!/usr/bin/env bash
set -e

REPO=$(cd "$(dirname "$0")/.." && pwd)

wait_for_url() {
  local url="$1"
  local timeout="${2:-60}"
  local elapsed=0
  echo "Waiting for $url..."
  until curl -sf "$url" > /dev/null 2>&1; do
    if [ "$elapsed" -ge "$timeout" ]; then
      echo "Timed out waiting for $url" >&2
      return 1
    fi
    sleep 1
    elapsed=$((elapsed + 1))
  done
  echo "$url is ready."
}

# ── Environment ────────────────────────────────────────────────────────────────

# ── Unit tests ────────────────────────────────────────────────────────────────

echo "==> Running Rust unit tests..."
cargo test -p hyle --manifest-path "$REPO/Cargo.toml"
cargo test -p hyle-dioxus --manifest-path "$REPO/Cargo.toml"
cargo test -p hyle-source-qmap --manifest-path "$REPO/Cargo.toml"

echo "==> Running hyle-react unit tests..."
npm test --prefix "$REPO/packages/hyle-react"

echo "==> Running hyle-react-dom unit tests..."
npm test --prefix "$REPO/packages/hyle-react-dom"

# ── Dioxus ────────────────────────────────────────────────────────────────────

echo "==> Starting dioxus server..."
DIOXUS_PUBLIC_PATH="$REPO/target/dx/hyle-dioxus-example/debug/web/public" \
  cargo run -p hyle-dioxus-example &
DIOXUS_PID=$!

cleanup_dioxus() {
  echo "==> Stopping dioxus server (PID $DIOXUS_PID)..."
  kill "$DIOXUS_PID" 2>/dev/null || true
}
trap cleanup_dioxus EXIT

wait_for_url "http://localhost:8080" 120

echo "==> Running dioxus e2e tests..."
npx playwright test --config "$REPO/examples/dioxus/e2e/playwright.config.ts"

cleanup_dioxus
trap - EXIT

# ── React ─────────────────────────────────────────────────────────────────────

echo "==> Starting react servers..."
# Kill any stale processes holding the React ports before starting
fuser -k 3001/tcp 2>/dev/null || true
fuser -k 4173/tcp 2>/dev/null || true
node "$REPO/examples/react/server.js" &
REACT_API_PID=$!

# Build the SSR bundle if not already built
if [ ! -f "$REPO/examples/react/dist/server/entry-server.js" ]; then
  echo "==> Building SSR bundle..."
  npm run build:ssr --prefix "$REPO/examples/react"
fi

PORT=4173 node "$REPO/examples/react/ssr-server.js" &
REACT_SSR_PID=$!

cleanup_react() {
  echo "==> Stopping react servers..."
  kill "$REACT_API_PID" 2>/dev/null || true
  kill "$REACT_SSR_PID" 2>/dev/null || true
  pkill -P "$REACT_API_PID" 2>/dev/null || true
  pkill -P "$REACT_SSR_PID" 2>/dev/null || true
  wait "$REACT_API_PID" "$REACT_SSR_PID" 2>/dev/null || true
}
trap cleanup_react EXIT

wait_for_url "http://localhost:3001/api/source" 30
wait_for_url "http://localhost:4173" 30

echo "==> Running react e2e tests..."
npx playwright test --config "$REPO/examples/react/e2e/playwright.config.ts"

cleanup_react
trap - EXIT
