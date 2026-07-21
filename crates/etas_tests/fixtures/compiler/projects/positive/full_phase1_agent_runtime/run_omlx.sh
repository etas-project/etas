#!/bin/sh
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ETAS_BIN="${ETAS_BIN:-etas}"
MODE="${1:-run}"

if [ "$#" -gt 0 ]; then
  shift
fi

CHECKPOINT_DIR="${ETAS_CHECKPOINT_DIR:-$SCRIPT_DIR/target/checkpoints}"

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
  ./run_omlx.sh run    [extra etas run args...]
  ./run_omlx.sh resume [checkpoint-id] [extra etas resume args...]

Environment overrides:
  ETAS_BIN                  etas binary to run, default: etas
  ETAS_HOST_OMLX_API_KEY    local OMLX API key, required
  ETAS_CHECKPOINT_DIR       checkpoint dir, default: ./target/checkpoints

Modes:
  run     Execute the full Phase 1 Agent runtime path with model, tool, memory,
          policy, approval, and checkpoint support.
  resume  Resume a checkpoint from the same checkpoint dir. Defaults to id 1.
EOF
}

case "$MODE" in
  run)
    require_omlx_api_key
    mkdir -p "$CHECKPOINT_DIR"
    exec "$ETAS_BIN" run "$SCRIPT_DIR" \
      --profile local-omlx \
      --budget-tokens "${ETAS_BUDGET_TOKENS:-256}" \
      --checkpoint-dir "$CHECKPOINT_DIR" \
      "$@"
    ;;
  resume)
    require_omlx_api_key
    CHECKPOINT_ID="${1:-1}"
    if [ "$#" -gt 0 ]; then
      shift
    fi
    exec "$ETAS_BIN" resume "$CHECKPOINT_ID" \
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
