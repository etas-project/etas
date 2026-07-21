#!/bin/sh
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ETAS_BIN="${ETAS_BIN:-etas}"
MODE="${1:-smoke}"

if [ "$#" -gt 0 ]; then
  shift
fi

require_omlx_api_key() {
  if [ -z "${ETAS_HOST_OMLX_API_KEY:-}" ]; then
    echo "ETAS_HOST_OMLX_API_KEY must be set" >&2
    exit 2
  fi
  export ETAS_HOST_OMLX_API_KEY
}

usage() {
  cat <<'EOF'
Usage:
  ./run_omlx.sh smoke [extra etas args...]
  ./run_omlx.sh full  [extra etas args...]

Environment overrides:
  ETAS_BIN                  etas binary to run, default: etas
  ETAS_HOST_OMLX_API_KEY    local OMLX API key, required
  ETAS_CHECKPOINT_DIR       full-mode checkpoint dir, default: ./target/checkpoints

Modes:
  smoke  Planner -> Reviewer, two model calls. Runtime readiness is entry-reachable,
         so unrelated runtime-variant services do not block this flow.
  full   Runtime variants with retry, checkpoint, memory conflict, cache, and guard path.
EOF
}

case "$MODE" in
  smoke)
    require_omlx_api_key
    exec "$ETAS_BIN" run "$SCRIPT_DIR" \
      --profile local-omlx \
      --flow smoke_main \
      --budget-tokens "${ETAS_BUDGET_TOKENS:-128}" \
      "$@"
    ;;
  full)
    require_omlx_api_key
    CHECKPOINT_DIR="${ETAS_CHECKPOINT_DIR:-$SCRIPT_DIR/target/checkpoints}"
    mkdir -p "$CHECKPOINT_DIR"
    exec "$ETAS_BIN" run "$SCRIPT_DIR" \
      --profile local-omlx \
      --flow runtime_main \
      --budget-tokens "${ETAS_BUDGET_TOKENS:-256}" \
      --checkpoint-dir "$CHECKPOINT_DIR" \
      "$@"
    ;;
  -h|--help|help)
    usage
    ;;
  *)
    echo "unknown mode: $MODE" >&2
    usage >&2
    exit 2
    ;;
esac
