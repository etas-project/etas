# Local Recovery And Integration Review

Date: 2026-11-21. This is a recovery/validation record, not a declaration that
the current toolchain is fully accepted.

## Merge Decisions

All eight original committed checkout heads were already ancestors of the
fetched remote `main`. The original 499 dirty paths were not 499 unmerged
features. No old branch or whole-source snapshot was merged back into `main`.

Of the differing tracked files, a read-only three-way comparison found 105
with no remaining textual local delta, 23 clean textual merge candidates,
and 207 conflicts requiring semantic review. A clean textual merge was not
treated as permission to apply it: some candidates duplicated an existing
`RegistrySource` declaration or an existing `begin_match_arm` hook. Those
duplicate definitions and the unused handler-completion helper were not ported.
All 499 original paths now have a per-file decision in the
[disposition ledger](local-recovery-disposition-2026-11-21.json). Exact content
and file mode, not merely branch ancestry, prove that 276 local file versions
were already integrated into remote main history; 107 equal current main and
two are deleted in both. The remaining 114 paths were reviewed independently.
Of the 207 initially conflicting tracked paths, 166 have exact historical proof.
The other 41 were resolved semantically, not by taking an entire old file.

Keep the current handler scope evidence and checked resumptive trace analysis;
do not restore the older `enclosing_completion_preserved` boolean. Keep current
closed-generic/runtime-layout checks, index inference progress, and typed
checkpoint fields that older local edits removed. Current Flow reflection,
metadata behavior, checked overload selection, match-arm hooks, and revision
restoration already cover the valuable older changes without duplicating APIs.

The old `solver/method.rs` and `conditional_callback_rejected.es` were deleted
upstream intentionally. They remain deleted. Python bytecode was not restored.

## Accepted Changes

- Core Secret requests now validate the current action grant and time budget.
  The CLI environment adapter validates before accessing its provider; the
  interpreter test adapter follows the same boundary contract.
- Negative tests cover missing/wrong Secret grants, expired requests, and the
  fact that a sealed secret handle does not grant authority. An unindexed host
  request cannot manufacture a static type selector from a runtime key string.
- Three standard-type regressions were recovered. The old assumption that
  `JsonValue` is opaque was rejected: current SPEC chapter 19 defines its six
  public enum variants, which the recovered test now checks explicitly.
- Std registry source-role compatibility now checks generic parameter kinds.
  This closes a real path that accepted a type parameter in place of the effect
  row in `Task<T, effect E>`. The existing negative regression is retained.
- Stale Rust test fixtures were adapted to current model protocol fields,
  descriptor defaults, optional HIR initializers/source bodies, external
  metadata fields, parameter interfaces, and analysis/trace enum variants.
  Assertions were not removed, and production validators were not weakened.
- The SDK inventory example now calls the lazy public/image accessors and
  propagates loading errors instead of accessing private fields.
- Live oMLX tests retain their explicit opt-in. If explicitly run without the
  required enable flag, they now fail instead of returning early as a false pass.
- The parser's checked `Option` unwrap was removed without changing receiver
  validation semantics.

Thirteen unique design/audit documents were recovered under each owning
repository's `docs/archive/local-recovery-2026-11-21/`. Every recovered body was
verified against its immutable backup. Historical designs and worklists are
not reinstated as current specifications or unfinished tasks.

Older local edits to existing architecture documents are preserved separately
as reversible `tracked-design-deltas.patch.json` historical data against their
recorded Git bases. Active documents are not replaced with those older drafts.
EDK's Session Context Processing ownership section is restored to its active
architecture: it agrees with current SPEC 03 section 3.3 and Core storage design.
Selection, summarization, tokenizer/provider choice and retry decisions belong
to applications; bounded history, conditional publication and receipts belong
to the public Etas substrate.

The current README was retained. The separate
[local development guide](../development/local-checkouts.md) records the local
Cargo override/lockfile behavior and the SDK prerequisite for CLI tests.

## Verification Status

The full-suite results below are the initial recovery audit, using sibling
integration where noted. Subsequent published-pin checks passed for Core,
Frontend all test targets, the 110 Syntax tests, three standard-type regressions,
four default-handler regressions, Interpreter all targets, and six Interpreter
Secret regressions. The CLI PR separately runs its installed-source-SDK CI;
that workflow result is evidence on the PR, not a replacement for the known
full-suite failures recorded here.

| Area | Result |
| --- | --- |
| Core full workspace | 901 passed, 4 ignored; no failures |
| Core changed-crate strict Clippy | Passed for Host, Std, Code, all targets |
| Interpreter Secret regressions | 6 passed |
| IDE full workspace | 80 passed, 1 ignored, using its committed dependency pins |
| Runtime workspace | Command succeeded; zero tests, not a runtime capability claim |
| Frontend full workspace | 1857 passed, 26 failed, 1 ignored; also one crashed library-test target |
| CLI environment Secret regression | 1 passed |
| CLI rebuilt-binary/source-SDK smoke | Hello check/run and isolated CLI check regression passed |
| CLI all-features check | Passed; existing dead-code warnings remain |
| CLI workspace | Initial SDK-missing full run was aborted; not an acceptance pass |
| Interpreter full workspace | Blocked by the reproducible stack overflow below |
| Interpreter strict Clippy | Failed on existing runtime/layout/style diagnostics; not waived |
| Frontend / CLI strict Clippy | Failed on existing Types / CLI diagnostics; not waived |

All Frontend test targets now compile. Its full run used `--no-fail-fast` and
reported ten failing targets. The 26 assertion failures do not include tests
whose process aborted before a summary could be produced. Core, Frontend,
Interpreter, CLI, IDE and Runtime formatting checks and the changed repositories'
tracked-diff whitespace checks passed.

The matching source SDK was installed under the local CLI workspace's `target`
prefix. A previously failing isolated `check_success_exits_zero` then passed.
The initial SDK-missing run is not reported as a valid final CLI acceptance run.
Its remaining SDK subprocesses were terminated and that termination was recorded
explicitly. Partial completed-target counts from that aborted run are not a full
CLI result. A fresh CLI build, matching SDK installation and targeted smoke
commands are recorded separately in the private verification receipts.

The final debug-build smoke measured 3.45 seconds for `hello` checking and
39.87 seconds for execution with project caching off. Execution printed
`hello from Etas` and exited successfully. These single debug runs are not a
release performance benchmark or evidence that startup cost is acceptable.

Loopback-dependent tests were rerun with the required permissions after the
sandbox rejected listener binding. Live model tests and Linux isolation
acceptance were not enabled. No assertions were removed, stack limits raised,
new ignores added, or production fallbacks introduced to obtain passing results.

## Blocking Interpreter Finding

The existing test below reproduces the crash in isolation, without business
recursion and without this recovery's Secret paths:

```sh
cargo test -p etas_interpreter --offline --lib \
  testing::cases::concurrency::host_waiting_parent_handles_a_child_while_its_arm_also_waits_on_host \
  -- --exact --nocapture
```

LLDB stops at BTree insertion of a `ScheduledTask`, reached from:

```text
driver::task::drive_one_with_root
  -> EvalMachine::run_until_yield / run_until_boundary
  -> EvalContext::execute_task_start / start_checked_task
  -> TaskScheduler::spawn / spawn_in_scope
  -> BTreeMap::insert
```

However, the BTree insertion is the failure location, not sufficient evidence
that the BTree algorithm itself is the root cause. Frame-SP deltas in the same
debug build measured approximately:

| Active layer | Bytes |
| --- | ---: |
| `driver::task::drive_one_with_root` poll | 552384 |
| `driver::drive::drive_scope` poll | 461232 |
| `drive_eval_signal` poll | 133264 |
| `EvalMachine::run_until_boundary` | 119904 |
| `drive_invocation` poll | 102480 |
| `control_from_root` poll | 100528 |
| `resume_root_wait` poll | 97824 |

These are debug-build frame observations, not cross-platform ABI guarantees.
`ScheduledTask` itself measured 4464 bytes. The public invocation future
measured 26752 bytes; that measurement alone does not establish it as the cause.
A temporary measurement assertion was removed, not converted into an arbitrary
new public API size requirement. No speculative runtime layout fix was merged.

### Required Engineering Follow-Up

1. Start in `src/driver/task.rs::drive_one_with_root` and
   `src/driver/drive.rs::drive_scope`, not the parser or language call-depth limit.
   Inventory inline owned machine/task/continuation values and awaited branch
   temporaries responsible for the large poll frames.
2. Split scheduling/dispatch/settling operations into small state transitions
   with explicit owned suspension records. Keep heavyweight suspended state
   heap-owned and avoid copying it through nested async poll layers. Preserve
   the current task ownership, cancellation, Host operation and handler rules.
3. Review task-table and extraction APIs together. Boxed task ownership can
   reduce BTree node movement, but merely boxing the table, public future, or
   failing test is not proof that the two largest driver frames were fixed.
4. Retain the original regression and verify it with the normal test-thread
   configuration. Cover child Host waits, nearest handler selection, checkpoint
   serialization and resume, cancellation during both waits, and repeated task
   admission. Do not use a raised `RUST_MIN_STACK` as acceptance evidence.
5. Repeat stack measurements and the full interpreter suite. Address the
   strict-Clippy large-variant/large-error and ownership findings in their
   owning layers rather than adding blanket `allow` attributes.

## Remaining Frontend Review

The second definite process-level blocker is
`package_metadata::projection::type_projection::tests::deep_acyclic_types_reach_the_explicit_metadata_budget`.
It aborts with stack overflow in the Frontend library test target. The test is
in `src/package_metadata/projection/type_projection/tests.rs`; its owning
implementation is `src/package_metadata/projection/type_projection.rs`.
Projection and cleanup must reach the configured depth/node budget without
native-stack exhaustion. Audit recursive projection, retained intermediate
values and recursive destruction; use bounded explicit traversal where needed.
Keep the original budget regression and normal stack configuration.

The remaining assertion failures need contract-aware triage, not a blanket
expectation rewrite:

- Effects: selected Agentic requested-action facts, missing callable-source
  evidence, rebinding through an unknown callback, and trace expansion limits.
  Inspect retained parameter/projection placeholders before deciding that absence
  of a diagnostic is unsound; do not reject a valid abstract parameter template
  merely to satisfy an older negative fixture.
- The trace-limit fixture intersects the same two alternatives fourteen times.
  Check normalization/idempotence before treating its missing size-limit error
  as a bug. A genuine expansion-budget regression should exercise distinct
  alternatives that survive normalization.
- Frontend: duplicate-name versus entry-selection diagnostics, effect-row
  metadata replay, suspension/body artifact ownership, reflection descriptors,
  and runtime type-role materialization.
- Source-stdlib fixtures: several partial source projects no longer provide
  `Array`, `Iterations`, or the selected `std.collections.Iter` dependency. Verify
  their bootstrap/environment completeness; do not restore implicit private
  standard-library injection to make an incomplete fixture pass.
- Spec method fixtures: compare the latest receiver/method rules against
  declarations that use `input: T` and calls that implicitly supply a receiver.
  Resolve that contract distinction before changing either the solver or tests.

Strict-Clippy work is also outstanding: seven current Types diagnostics, two
CLI diagnostics, and runtime/layout/ownership diagnostics in Interpreter. They
are recorded verbatim in the private logs. Avoid unrelated bulk refactors or
warning suppression as part of this recovery.

## Local State And Publication

The complete pre-sync snapshots, bundles, binary patches, detailed path
comparison and test receipts are retained privately under the workspace's
`_backups/20261121T113929Z-before-main-sync/` directory.

Three ignored `.cargo/config.toml` files previously selected sibling path
dependencies automatically. They were backed up and removed from automatic
discovery; explicit profiles with absolute paths remain in the private recovery
backup. Published manifests and lockfiles are updated producer-first to exact
merged Git revisions, without upgrading unrelated registry dependencies. Normal
Cargo/editor metadata queries therefore no longer dirty the published lockfiles
merely by selecting sibling paths.

Only ignored Rust incremental build caches were removed to recover disk space.
Source, backups, installed SDK data, and test executables were retained. The
aborted run's PID-qualified temporary SDK workspaces were removed separately.
The workspace-level `.ignore` excludes private backup snapshots from ordinary
code searches; it does not remove or change their contents.
The initial backup/synchronization audit performed no publication. The user
subsequently authorized necessary commits, pushes and PR merges for this goal.
Recovery publication proceeds through normal PRs, without overriding required
checks. [Core #70](https://github.com/etas-project/etas-core/pull/70) and
[Frontend #95](https://github.com/etas-project/etas-frontend/pull/95) retain the
audited fixes; [EDK #4](https://github.com/etas-project/etas-edk/pull/4) restores
only current-contract documentation. [Interpreter #116](https://github.com/etas-project/etas-interpreter/pull/116)
retains the Secret regression and shared descriptor adaptation. CLI publishes
the matching adapter fix and the same precise dependency set. This is repository
synchronization, not acceptance of the unresolved full-suite failures above.
