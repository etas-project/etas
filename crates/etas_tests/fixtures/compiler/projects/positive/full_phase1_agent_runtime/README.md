# Full Phase 1 Agent Runtime

This fixture exercises a project-level sequential multi-agent runtime path:
`RuntimePlanner.run(...)` feeds a typed `RuntimeReviewer.run(...)`, whose model
request uses a source-bodied tool call loop before decoding a typed record. The
same flow also covers real typed memory conflict handling through
`Error[MemoryConflict]`, retry, checkpoint/resume metadata, handler completion,
console output, and a source policy (`RuntimeGate`) that is forwarded to the
runtime policy provider as an active policy.

Manual live run:

```bash
ETAS_HOST_OMLX_API_KEY=<local-omlx-key> \
etas run . --profile local-omlx --budget-tokens 256 --checkpoint-dir target/checkpoints
```

The checked-in `local-omlx` profile uses the rule-based local policy provider for
dev/test execution. The `unsafe-*` local policy modes are only for narrow
diagnostics.
