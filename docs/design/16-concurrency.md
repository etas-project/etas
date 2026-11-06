# Concurrency

## 1. Goal

Etas concurrency should be structured, analyzable, and safe for agent systems.
The language should not expose unrestricted threads as the first abstraction.
Instead, concurrency is expressed through standard-library combinators and
compiled with effect, policy, limit, and trace information.

The core design is:

```text
Effect-free deterministic flow:
    may run concurrently when data-independent

Effectful flow:
    may run concurrently only when effect/action/resource conflicts are safe

Agent/tool flow:
    may run concurrently only under explicit runtime mediation, limits, policy,
    handler dispatch, trace recording, and cancellation rules
```

This gives Etas a stronger concurrency story than ordinary host-language agent
frameworks. A Python or TypeScript callback may hide arbitrary side effects;
Etas can make the effect/action surface of each branch visible before the
runtime schedules it.

## 2. No `parallel` Keyword In MVP

Etas does not need a source-level `parallel` statement in the MVP. Concurrent
composition is a library/runtime feature:

```etas
let (summary, risks, citations) = join((
    () => draft ~> Summarizer,
    () => draft ~> RiskReviewer,
    () => draft ~> CitationChecker,
), limit = [
    Concurrency(3),
    Tokens(80_000),
    WallTime(seconds(45)),
]);
```

`join` is a standard-library combinator with compiler recognition and runtime
support. The compiler can still lower independent branches to AIR regions that
the runtime may schedule concurrently.

## 3. Structured Concurrency

Etas concurrency is structured:

- a concurrent scope owns all child tasks it starts;
- child tasks cannot outlive the scope;
- cancellation, failure, limits, handlers, trace specs, runtime policies, and trace events are scoped;
- a concurrent expression returns only after its children complete, fail, or are
  cancelled;
- no background task is silently detached by default.

This matches agent-system needs better than free `spawn`: side effects, model
calls, approvals, and budget consumption must remain attributable to a source
scope and a trace region.

## 4. Standard Combinators

The MVP should provide these standard combinators:

| Combinator | Behavior | Typical use |
|---|---|---|
| `join` | Return all values, or fail fast by propagating the primary unhandled error | Fan-out/fan-in where all results are required |
| `try_join` | Return `Result<Output, E>` for a common typed error, cancelling siblings on the first error | All-or-nothing work with an explicit error value |
| `collect` | Wait for all branches and return per-branch `Result` values | Batch work where partial success is useful |
| `race` | Return the first successful branch and cancel losers | Fallback providers, redundant retrieval, latency hedging |
| `map_concurrent` | Apply a flow with bounded concurrency and fail-fast errors | Batch work where every result is required |
| `map_concurrent_collect` | Apply a flow with bounded concurrency and retain a `Result` for every input | Batch work where partial success is useful |

Example:

```etas
let reports = map_concurrent(
    papers,
    paper => paper ~> PaperSummarizer,
    limit = [Concurrency(8), Tokens(120_000)]
);
```

The exact stdlib surface can evolve, but the semantic requirement is stable:
these combinators create structured concurrent scopes visible to type checking,
effect inference, policy analysis, limit checking, runtime scheduling, and trace
recording.

The error surface is intentionally not selected by an option flag:

```text
map_concurrent(xs, f)         : Array<B> ![E, Error<X>]
map_concurrent_collect(xs, f) : Array<Result<B, X>> ![E]
```

Both preserve input order in their result arrays. `map_concurrent` cancels
unfinished items after an error; `map_concurrent_collect` waits for every item.

`join` and `try_join` are both fail-fast. Their distinction is representation,
not scheduling: `join` propagates the primary unhandled error effect, while
`try_join` handles one common typed `Error<E>` from its branches and returns
`Result<Output, E>`. Other unhandled effects still escape normally. `collect`
is the wait-all per-branch-result operation.

`join` should support two input shapes:

- tuple input for fixed heterogeneous branches, returning a tuple;
- array input for homogeneous branches, returning an array.

For example:

```etas
let (a, b) = join((
    () => LoadProfile(user),
    () => LoadHistory(user),
));

let summaries = join([
    () => Summarize(paper_a),
    () => Summarize(paper_b),
]);
```

## 5. Type And Effect Rules

For a fixed tuple of branches:

```text
branch_i : () -> T_i ![E_i]
----------------------------------------
join((branch_1, ..., branch_n))
  : (T_1, ..., T_n) ![E_1 union ... union E_n]
```

For homogeneous collection branches:

```text
branch_i : () -> T ![E_i]
----------------------------------------
join([branch_1, ..., branch_n])
  : Array<T> ![E_1 union ... union E_n]
```

For concurrent mapping:

```text
f  : A -> B ![E]
xs : Array<A>
----------------------------------------
map_concurrent(xs, f) : Array<B> ![E]
```

`join` and `map_concurrent` do not hide effects. The resulting expression keeps
the union of branch effects unless a handler in the concurrent scope eliminates
some of them. This preserves normal effect-boundary and policy checking.

## 6. Effect-Free Concurrency

An effect-free deterministic flow has no observable interaction with the
outside world:

```etas
flow normalize(title: string) -> string {
    title.trim().lowercase()
}
```

If two calls are data-independent, the compiler or runtime may run them in
parallel, reorder them, memoize them, cancel and recompute them, or speculatively
execute them.

```etas
let a = normalize(left);
let b = normalize(right);
let merged = combine(a, b);
```

`normalize(left)` and `normalize(right)` can be scheduled concurrently because
neither call has effects and neither depends on the other's result.

This is safe because Etas ordinary values have value semantics. Implementations
may use sharing internally, but accidental shared mutation is not observable.

## 7. Data Dependencies

Effect freedom does not remove data dependencies:

```etas
let a = parse(input);
let b = analyze(a);
```

`analyze(a)` depends on `parse(input)`, so the two calls cannot be reordered into
parallel execution even if both are effect-free.

The compiler should use ordinary data-dependency analysis before scheduling
branches concurrently.

## 8. Effect And Resource Conflicts

Effectful branches may still be safe to run concurrently when their actions and
resource parameters do not conflict.

| Branch A | Branch B | Default judgment |
|---|---|---|
| `Memory.read<R>` | `Memory.read<R>` | Safe |
| `Memory.read<R>` | `Memory.write<R>` | Conflict |
| `Memory.write<R>` | `Memory.write<R>` | Conflict |
| `Memory.write<ProjectA>` | `Memory.write<ProjectB>` | Safe when regions are distinct |
| `ProjectWorkspace.write<"a.md">` | `ProjectWorkspace.write<"b.md">` | Safe when paths are proven disjoint |
| `ProjectWorkspace.write<_>` | `ProjectWorkspace.write<_>` | Runtime check or serialize when disjointness is unknown |
| `AcademicSearch.search` | `AcademicSearch.search` | Safe, but shared budgets apply |
| `CompanyEmail.send<WorkAccount>` | `StripePayment.charge<Account>` | Policy-sensitive |

The conflict check is based on:

```text
effect tag + action name + resource parameters
```

not only on the coarse effect tag.

## 9. Policy Happens-Before

Policies can impose ordering constraints. For example:

```etas
spec SafeEmail: trace =
    +Approval.request
    & +CompanyEmail.send<WorkAccount>
    & (Approval.request >> CompanyEmail.send<WorkAccount>);
```

This concurrent expression is invalid:

```etas
join((
    () => approve("Send email?", draft),
    () => perform CompanyEmail.send(WorkAccount, draft),
));
```

The `join` scope does not establish that approval happens before sending.
Correct code must express the order:

```etas
let ok = approve("Send email?", draft);
if ok {
    perform CompanyEmail.send(WorkAccount, draft);
}
```

Concurrency analysis therefore reasons over a partial order of actions, not only
over a set of effects.

## 10. Limits And Budgets

Concurrent scopes share parent limits. Child branches must not receive fresh
budgets merely because they run in parallel.

```etas
let result = join((
    () => topic ~> Researcher,
    () => topic ~> CompetitorResearcher,
), limit = [
    Concurrency(2),
    Tokens(50_000),
    Cost(usd(2.00)),
    WallTime(seconds(30)),
]);
```

Rules:

- `Concurrency(n)` bounds the number of simultaneously active child tasks;
- `n` must be a positive `usize` within the runtime implementation ceiling;
  the compiler rejects invalid constants and the runtime validates dynamic or
  decoded values again;
- a parked task does not occupy an active concurrency slot;
- a runnable descendant counts against every enclosing concurrency ceiling, so
  a nested scope has its own bound but cannot widen an ancestor bound;
- a parent waiting for nested children is parked and releases its active slot,
  preventing a bounded parent scope from deadlocking its descendants;
- token, context, cost, and wall-time limits are shared by the whole concurrent
  scope;
- child branches may have stricter nested limits;
- a child cannot widen a parent limit;
- if the scope exhausts a required limit, the runtime cancels unfinished branches
  and raises the appropriate error, such as `Error<BudgetExceeded>`.

## 11. Error And Cancellation Semantics

Recommended semantics:

| Combinator | Error behavior | Cancellation behavior |
|---|---|---|
| `join` | Propagate the deterministic primary unhandled error | Cancel unfinished siblings and wait for cleanup |
| `try_join` | Return `Err(primary)` for its common typed error | Cancel unfinished siblings and wait for cleanup |
| `collect` | Convert each branch to `Result<T, E>` | Does not cancel siblings just because one branch fails |
| `race` | Return first success; if all fail, propagate the deterministic primary error | Cancel losing branches after success |
| `map_concurrent` | Propagate the deterministic primary error | Cancel unfinished items and wait for cleanup |
| `map_concurrent_collect` | Return one input-ordered `Result` per item | Does not cancel siblings just because one item fails |

`std.task.run` uses the same fail-fast rule. Inherited handlers may recover
within a child; an error that remains unhandled at the child boundary fails the
owning scope. `join` and `try_join` have the same fail-fast behavior here;
`collect`, not a silent `join` variation, provides per-branch results. Result
collection requires a declared common error type or explicit error mapping;
it does not change the single-Error rule of `?` or invent implicit error unions.
The original typed failure is propagated after cancellation and cleanup, not
fabricated as a normal task result. `try_join` is the explicit exception that
converts its declared common error to `Err`. Competing failures and the selected
primary failure are trace-visible.

Fail-fast scopes choose the primary error by the lowest source branch index
among failures that reach a terminal state before cancellation settles. Array
operations use input index; tuple operations use tuple position. A scheduler
turn drains ready terminal notifications before making this choice and orders
same-turn notifications by task id. Secondary failures remain trace events and
do not replace the primary error. This makes the selection independent of host
completion polling order for failures already observed at the cancellation
barrier.

Cancellation is cooperative at runtime mediation points: model calls, tool calls,
effect actions, memory APIs, waits, and loop boundaries. A purely deterministic
CPU-bound branch may be cancelled at implementation-defined safe points.

Cancellation must be trace-visible when it affects non-deterministic work or
external actions.

The default scheduler uses a FIFO ready queue. Starting or waking a task appends
it to the queue, and each dispatch runs one cooperative quantum. A task that
repeatedly yields therefore cannot starve an older ready task. External host
completion timing remains nondeterministic and is captured by the structured
trace for replay.

Completion, resumption, cancellation, and scope exit coordinate task and
continuation state atomically. Each task has one terminal completion, and each
captured continuation has one terminal resumption or disposal. A task may wait
many times, capturing a fresh continuation at each suspension. A completion
racing cancellation cannot resume the same continuation twice or revive a
cancelled scope. Late completions are reconciled or discarded with trace
evidence. Cancellation is not rollback: already executed external actions
remain real and may require application-level reconciliation.

On cooperative failure, cancellation, or `finish`, the runtime unwinds the
affected managed frames and runs their registered resource cleanup before the
scope releases ownership. No `finally` keyword is added by this design; the
cleanup guarantee belongs to resource/task execution contracts. Forced process
termination is not a guarantee that arbitrary cleanup code can run. Catching a
local cancellation error cannot clear the parent's cancellation state or reset
its budgets.

## 12. Handlers In Concurrent Scopes

Handlers are scoped dynamically. A handler around a concurrent expression applies
to all child branches in that scope:

```etas
handle {
    join((
        () => SaveDraft(a),
        () => SaveDraft(b),
    ))
} with DryRunWorkspace;
```

Effects handled by the scoped handler are removed from the escaping effect row
only if the handler covers those branch actions. Effects performed by handler
arms still escape normally.

Handler arms must not hide audit evidence: even when a branch action is handled,
the trace records the attempted action and the handler decision.

## 13. Trace And Replay

A concurrent scope produces structured trace events:

```text
JoinEnter(scope_id)
  TaskStart(task_id = 1)
  TaskStart(task_id = 2)
  ActionAttempt(AcademicSearch.search, task_id = 1)
  AgentCall(Reviewer, task_id = 2)
  TaskEnd(task_id = 1)
  TaskEnd(task_id = 2)
JoinExit(scope_id)
```

Trace events must preserve:

- source span and stable AIR node id;
- branch/task id;
- parent concurrent scope id;
- action attempts, handler dispatch decisions, and default-action execution decisions;
- concrete budget usage;
- cancellation and failure events;
- replay/recompute/resample decisions.

This allows replay to restore the logical behavior of a concurrent run without
depending on the original wall-clock interleaving.

`Task`, `Scheduler`, and continuation handles are runtime capabilities, not user
data. They cannot be serialized, hashed, compared, persisted, sent to a host or
model payload, or decoded as ordinary Etas values. Runtime checkpoints use a
separate internal scheduler snapshot that may encode task ids, owner scopes,
ready/parked state, waiters, and continuation-arena state. That encoding is
versioned runtime state, is valid only with its checkpoint, and is never exposed
through source-level serialization APIs.

## 14. Automatic Scheduling

The compiler/runtime may automatically schedule independent work concurrently
when all of these are true:

1. data dependencies permit it;
2. inferred effects are absent or conflict-free;
3. policy ordering is preserved;
4. limits allow concurrency;
5. trace and replay semantics are preserved.

This permits optimization such as:

- running independent deterministic preprocessing in parallel;
- parallelizing independent retrievals;
- scheduling independent reviewers concurrently;
- serializing conflicting writes automatically;
- falling back to sequential execution when resource conflicts are unclear.

Automatic scheduling must be semantics-preserving. If the compiler cannot prove
safety, it should keep the source order or require an explicit combinator plus
runtime checks.

## 15. Advantages Over Ordinary Agent Frameworks

| Dimension | Etas | Typical Python/TypeScript agent framework |
|---|---|---|
| Side-effect visibility | Effects/actions are inferred and attached to branches | Callback may hide arbitrary SDK calls or global mutation |
| Race safety | Value semantics plus effect/resource conflict checks | User must reason about shared objects, closures, and clients |
| Policy preservation | `before`/`after` requirements become ordering constraints | Usually runtime guardrails or manual hooks |
| Budget control | Concurrent scope shares typed limits | Separate timeouts or per-call options can accidentally reset budgets |
| Replay | Structured task trace with action metadata | Often logs concrete events, but lacks language-level replay semantics |
| Optimization | Compiler can schedule, hoist, deduplicate, and serialize safely | Optimizations are manual and local |
| Agent/tool mediation | Model/tool calls remain under effect, policy, handler, and trace checks | Tool calls are often ordinary functions exposed to a model runtime |

The main benefit is not that Etas can run things in parallel. Existing
frameworks can do that. The benefit is that Etas can decide when parallelism is
safe, what authority it requires, what budget it consumes, how it is traced, and
how recovery should behave.

## 16. MVP Scope

MVP should include:

- `join`;
- `try_join`;
- `collect`;
- `race`;
- `map_concurrent`;
- `map_concurrent_collect`;
- `Concurrency(n)` limit;
- effect/action/resource conflict checks for standard effects;
- structured trace events for concurrent scopes;
- fail-fast and collect-style error semantics;
- runtime cancellation at mediation points.

MVP should not include:

- unrestricted `spawn`;
- `async` / `await` keywords;
- channels and `select`;
- shared mutable state;
- preemptive thread interruption as a source-language guarantee;
- actor syntax;
- detached background tasks.

Those features can be added later if they earn their complexity. The first
version should make common fan-out/fan-in agent workflows safe, bounded,
traceable, and optimizable.
