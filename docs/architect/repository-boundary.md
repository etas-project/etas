# Etas Repository Boundary

## Responsibility

`etas` owns:

- user-facing CLI commands;
- release packaging and distribution entry points;
- cross-repository integration tests;
- orchestration across component repository facades.

Phase 1 CLI design is recorded in `docs/architect/phase1-cli-design.md`.

## Forbidden

This repository must not own:

- parser, HIR, type, or effect semantics;
- interpreter internals;
- FIR/AIR builder or verifier internals;
- runtime authority internals;
- LSP/editor protocol internals.

## Dependency Rule

Allowed dependencies:

```text
etas -> etas-core
etas -> etas-frontend
etas -> etas-interpreter
etas -> etas-optimizing
etas -> etas-runtime
etas -> etas-ide
```

The CLI should route to public component APIs instead of becoming a monolithic
implementation owner.
