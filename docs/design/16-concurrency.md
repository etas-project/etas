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
- cancellation, failure, limits, handlers, policies, and trace events are scoped;
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
| `join` | Wait for all branches and return their values | Fan-out/fan-in where all results are required |
| `try_join` | If any branch raises an error, cancel remaining branches and raise the error | All-or-nothing parallel work |
| `collect` | Wait for all branches and return per-branch `Result` values | Batch work where partial success is useful |
| `race` | Return the first successful branch and cancel losers | Fallback providers, redundant retrieval, latency hedging |
| `map_concurrent` | Apply a flow to a collection with bounded concurrency | Batch summarization, validation, retrieval, tests |

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
| `join` | Propagate first unhandled error unless the stdlib variant documents result collection | Cancel unfinished siblings |
| `try_join` | Propagate first unhandled error | Cancel unfinished siblings immediately |
| `collect` | Convert each branch to `Result<T, E>` | Does not cancel siblings just because one branch fails |
| `race` | Return first success; if all fail, raise or return collected failure depending on variant | Cancel losing branches after success |
| `map_concurrent` | Depends on selected variant: fail-fast or collect | Bounded by `Concurrency(n)` |

Cancellation is cooperative at runtime mediation points: model calls, tool calls,
effect actions, memory APIs, waits, and loop boundaries. A purely deterministic
CPU-bound branch may be cancelled at implementation-defined safe points.

Cancellation must be trace-visible when it affects non-deterministic work or
external actions.

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
- `map_concurrent`;
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
