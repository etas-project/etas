# Concurrency

## 1. Goal

Etas concurrency should be structured, analyzable, and safe for agent systems.
The language should not expose unrestricted threads as the first abstraction.
Instead, concurrency is expressed through standard-library combinators and
compiled with effect, policy, limit, and trace information.

This chapter specifies a language and standard-library design, not current
implementation coverage. Ordinary programs use default scheduling through
combinators. Scheduler authors can use optional `as k` handler bindings and
the scoped task APIs without adding `async`, `await`, or `suspend` keywords.

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

These are distinct responsibilities in the [five-layer model](01-overview.md#53-feature-map-by-layer):
Etas can implement combinator logic, the compiler recognizes the resolved
standard contract, and the runtime provides structured task execution. A
user-defined same-named `join` does not acquire scheduling or analysis rules
merely from its spelling.

## 3. Structured Concurrency

Etas concurrency is structured:

- a concurrent scope owns all child tasks it starts;
- child tasks cannot outlive the scope;
- cancellation, failure, limits, handlers, trace specs, runtime policies, and trace events are scoped;
- a concurrent expression returns only after its children complete, fail, or are
  cancelled;
- no background task is silently detached by default.

The program entry and combinators provide managed execution with default async
handling. `std.task.run(scheduler => { ... })` creates an explicit owned scope
when code needs the scheduler handle. `std.task.start` is permitted only in a
managed scope; it is not unrestricted process/thread creation. Tasks inherit
the active handler environment, trace specs, runtime policy, limits, cancellation
scope, and trace links at their start site. Owner/lifetime requirements must
survive helper calls and package interfaces.

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

The same rule applies to `std.task.start(job)`: the enclosing scope includes
the job's effects, requested actions, trace behavior, and determinism when work
is started, not only when `await` is reached. Dropping a task handle neither
detaches the job nor removes its behavior from analysis. `Task<T, E>` preserves
the result and effect-row relationship plus its checked owner/behavior metadata.
Handling scheduling actions cannot discharge unrelated child effects. An empty
escaping row after default handling is not proof of determinism or absence of
external requests.

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
    Approval.request >> CompanyEmail.send<WorkAccount>;
```

This concurrent expression is invalid:

```etas
join((
    () => approve("Send email?", draft),
    () => perform CompanyEmail.send(WorkAccount, draft),
));
```

The implicit permissions in `>>` do not establish ordering. The `join` scope
does not establish that matching accepted approval happens before sending.
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

The default task scope and fail-fast combinators use these semantics:

| Combinator | Error behavior | Cancellation behavior |
|---|---|---|
| `join` | Propagate first unhandled error | Cancel unfinished siblings and wait for cleanup |
| `try_join` | Propagate first unhandled error | Cancel unfinished siblings immediately |
| `collect` | Convert each branch to `Result<T, E>` | Does not cancel siblings just because one branch fails |
| `race` | Return first success; if all fail, raise or return collected failure depending on variant | Cancel losing branches after success |
| `map_concurrent` | Depends on selected variant: fail-fast or collect | Bounded by `Concurrency(n)` |

`std.task.run` uses the same fail-fast rule. Inherited handlers may recover
within a child; an error that remains unhandled at the child boundary fails the
owning scope. `join` and `try_join` have the same fail-fast behavior here;
`collect`, not a silent `join` variation, provides per-branch results. Result
collection requires a declared common error type or explicit error mapping;
it does not change the single-Error rule of `?` or invent implicit error unions.
The original typed failure is propagated after cancellation and cleanup, not
fabricated as a normal task result. Competing failures and the selected primary
failure are trace-visible.

Cancellation is cooperative at runtime mediation points: model calls, tool calls,
effect actions, memory APIs, waits, and loop boundaries. A purely deterministic
CPU-bound branch may be cancelled at implementation-defined safe points.

Cancellation must be trace-visible when it affects non-deterministic work or
external actions.

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

### 12.1 Handler-Based Scheduling

The standard `std.task.Async` effect has `yield() -> unit` and
`await<T, effect E>(task: Task<T, E>) -> T` actions. `yield` keeps the current
task runnable; `await` waits for a particular task result. Both have default
handlers in managed execution. Their public contracts are specified in
[Task And Scheduling APIs](19-standard-library.md#2-task-and-scheduling-apis).

A scheduler author may bind the current continuation with `as k`:

```etas
Async.yield() as k => {
    scheduler.requeue(k, ());
}

Async.await(task) as k => {
    scheduler.park(task, k);
}
```

This extends the existing handler arm syntax; it is not a new expression for
arbitrary continuation capture. `k` is inferred from the action result, handled
answer, remaining effects, and owner. A `never`-returning action cannot bind it.
Without `as k`, existing immediate `resume value;` remains unchanged. With
`as k`, immediate resume and deferred transfer use the same one-shot handle.

### 12.2 Ownership And Transfer

Continuations are affine, scope-bound handles, not ordinary copyable values.
Passing one to an owner-preserving helper or scheduler queue consumes/transfers
ownership. They cannot be cloned, stored globally or persistently, returned
beyond their owner, sent to a model, or placed in a copying container. Closures
and aggregates cannot hide an escape or copy. A scheduler may hold them only
while their task and captured handler environment remain live. Runtime checks
also reject stale, wrong-owner, or already-consumed handles before resumption.

The ownership transitions are conceptually:

```text
arm-owned -> scheduler-owned (ready or waiting) -> resumed or cancelled
arm-owned -> immediate resume or unwound by finish/termination
```

Transfer changes the owner; it does not clone the continuation. Resumption may
later reach another `perform`, which captures a different one-shot handle.

`scheduler.requeue(k, value)` and `scheduler.park(task, k)` return `never` to the
current arm. A successful transfer stops that arm and gives control to the
scheduler; it does not abort the program. It cannot be followed by `resume`,
another transfer, or ordinary work on the consumed handle. All reachable arm
paths must still resume, finish, transfer, or take another non-returning path.
A plain enqueue returning `unit` does not satisfy that control-flow rule.

Controlled transfer may cross a child-task boundary inside the same owned
structured group. It always restores the original task's identity, frames,
handler environment, and constraints, rather than executing the continuation
as the scheduler or as another task. Arbitrary cross-thread transfer and
detachment are not source-language facilities. The owner cannot return while
an accepted continuation is still live: it must resume or cancel and unwind it.

### 12.3 Waiting And Wakeup

`std.task.start(job)` schedules work without waiting for its result. Starting
two jobs and then awaiting the first allows both to make progress; starting
and awaiting one before starting the other does not express that concurrency.

`park` requires the continuation of the matching `Async.await(task)` request,
with the task result type and owner preserved. It handles an already-completed
task without entering an indefinite wait. Completion registration and terminal
state observation must be atomic enough to prevent lost wakeups. Pending
waiters become runnable only on completion or a failure/cancellation decision;
they must not be busy-polled by repeatedly yielding.

Waiting suspends the current task, not the whole scheduler thread. Models,
streams, and other host operations need non-blocking adapters or managed
blocking offload. Yielding cannot turn a blocking host function into async I/O.
Default handlers perform ordinary scheduling; a custom handler changes policy,
not the validity of task results or the authorization of protected host work.

### 12.4 Handler Lifetime And Early Finish

Resuming a task reinstalls the captured handler environment (deep resumption).
Handler arms themselves follow ordinary enclosing-handler lookup rules. A
handler frame cannot disappear while inherited child work or a parked
continuation still depends on it. Normal managed scope exit settles that work
before committing the answer; task handles cannot escape a shorter-lived
handler environment they still depend on.

Each action invocation has its own arm/continuation. Arms may interleave;
there is no implicit global lock held across an arm's wait. Capturing a handler
value does not create shared mutable local variables. Shared resources still
use explicit APIs and effect/resource conflict checks; a single-threaded
scheduler does not make arbitrary reentrant mutation safe.

`finish value;` targets its enclosing handled expression, not whichever child
happens to be executing the arm. If that expression encloses a concurrent
scope, finishing it cancels and cleans up the unfinished child work it owns
before delivering its answer. A handler local to one child finishes only that
child's handled region. Competing finish/error/cancellation decisions cannot
complete the same region twice. No explicit continuation can resume a region
after it has finished.

### 12.5 Context And Analysis Preservation

Parking, requeueing, and wakeup retain task/parent scope ids, handler frames,
trace-spec state, current authorization checks, cancellation, remaining budgets,
and source/code-revision identity. They neither grant new authority nor restore
revoked permission from an old snapshot. Waiting time counts toward active
wall-time limits, and restarting a scheduler or creating a child cannot reset
parent budgets.

`Async` is scheduling control, not another external-authority root. Handling
its actions does not hide model/network/file requests, child errors, or trace
ordering. Standard task operations are compiler-recognized contracts: type and
effect analysis must follow the jobs they start, even through helper flows and
package interfaces. The usual union of effects is still insufficient for
approval happens-before; task dependencies and action order remain necessary.
No unrestricted multi-shot continuation or general shared-reference system is
introduced by this extension.

### 12.6 Complete Example

This design example uses the standard task protocol and optional `as k`. It is
not a claim that the current interpreter implements these APIs. The two jobs
are started before either is awaited. The handler demonstrates explicit
scheduling; business code normally uses `join` with default handling instead.

```etas
import std.task;
import std.task.Async;

@model("gpt-5.5")
agent Reviewer(request: string) -> string {
    perform Async.yield();

    let prompt = Prompt.new()
        .system(Trusted("Review the request and report concrete issues."))
        .data(request);

    return perform infer(prompt);
}

flow main(args: Array<string>) -> i32 {
    return std.task.run(scheduler => {
        handle {
            let correctness = std.task.start(
                () => Reviewer.run("Check correctness")
            );
            let security = std.task.start(
                () => Reviewer.run("Check security")
            );

            let a = std.task.await(correctness);
            let b = std.task.await(security);

            std.io.println(std.text.join([a, b], "\n"));
            0
        } with {
            Async.yield() as k => {
                scheduler.requeue(k, ());
            }

            Async.await(task) as k => {
                scheduler.park(task, k);
            }
        }
    });
}
```

The unit value queued by `requeue` matches `Async.yield`'s result type. `park`
preserves the relationship between a task result and the captured await
continuation. Failure is propagated by the task protocol, not passed to that
continuation as a fake string. Unhandled task or output errors fail execution
instead of returning the normal exit code `0`; application-specific recovery
can use ordinary outer handlers. The example does not add an aborting `?` or
change the existing Error-to-Result semantics.

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
- park, runnable-queue, wakeup, and continuation transfer/consumption events;
- the completion or cancellation decision that won a wakeup race;
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
| Policy preservation | Trace-spec `>>`/`<<` requirements become ordering constraints | Usually runtime guardrails or manual hooks |
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
- runtime cancellation at mediation points;
- `std.task.run`, scoped `start`/`await`, and default `Async` handlers;
- optional `as k` with single-shot ownership and scope-preserving transfer;
- scheduler `requeue`/`park` contracts, wakeup races, and unwind/cleanup rules.

MVP should not include:

- unrestricted `spawn`;
- `async` / `await` keywords;
- a `suspend` keyword, continuation cloning, or multi-shot resume;
- channels and `select`;
- shared mutable state;
- preemptive thread interruption as a source-language guarantee;
- actor syntax;
- detached background tasks.

Those features can be added later if they earn their complexity. The first
version should make common fan-out/fan-in agent workflows safe, bounded,
traceable, and optimizable.
