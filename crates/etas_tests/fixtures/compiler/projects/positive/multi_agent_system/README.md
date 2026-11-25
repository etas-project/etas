# Multi Agent System

This fixture exercises a project-level multi-agent workflow with typed agent declarations, sequential agent calls, cross-module imports, and console output.

`app.agent.runtime_variants` adds five heavier Phase 1 runtime variants:

- `run_three_agent_version`: Planner, Researcher, and Reviewer agent calls.
- `run_retry_checkpoint_version`: agent retry orchestration plus checkpoints.
- `run_memory_conflict_version`: typed memory write conflict handled with `finish`.
- `run_cached_review_version`: typed memory read/write/upsert around a reviewed result.
- `run_abort_guard_version`: guarded `abort(...)` plus the memory conflict variant.

Researcher calls the Etas tool `EvidenceLookup` and sends its result back to the model. The memory conflict handler matches `Error<StorageError>` with code `condition_conflict`. The cached review flow reads the stored review, saves a checkpoint, and returns the value it read.

The `runtime_main` entry is guarded by `MultiAgentRuntimeGate`, which covers the
Agentic, Memory, and Console actions used by the runtime variants and is
forwarded to the runtime policy provider as an active source policy. It runs the
retry/checkpoint, memory conflict, cached review, and abort-guard variants in one
runtime path so CLI tests exercise the heavy flows rather than effects output
alone.

Manual live run:

```bash
ETAS_HOST_OMLX_API_KEY=<local-omlx-key> \
etas run . --profile local-omlx --flow runtime_main --allow-effects --budget-tokens 256
```
