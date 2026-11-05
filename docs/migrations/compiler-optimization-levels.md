# Compiler optimization levels

Release note: compilation commands now accept `-O0` through `-O3` and
`--opt-level 0` through `--opt-level 3`. O2 is the default; O3 remains opt-in.
`check`, `run`, and `dump hir` use the same option. Run checkpoints record the
effective level and resume with that level.

API migration:

- `optimize_hir = false` becomes `optimization_level = OptimizationLevel::O0`.
- `optimize_hir = true` becomes `optimization_level = OptimizationLevel::O2`.
- `quality_context_token_budget` remains an independent, semantics-changing
  policy option and is not enabled by O3.
