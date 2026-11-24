# Concurrency

## 1. Scope

Concurrency uses standard combinators and scoped `Async` handlers.
There are no `parallel`, `async`, `await` or `suspend` keywords.
Optional `as k` exposes single-shot, scope-bound continuations to schedulers.
Recognized standard symbols carry these contracts; same-named user flows do not.

## 2. Structured Concurrency

Etas concurrency is structured:

- a concurrent scope owns all child tasks it starts;
- child tasks cannot outlive the scope;
- cancellation, failure, limits, handlers, trace specs, runtime policies, and trace events are scoped;
- a concurrent expression returns only after its children complete, fail, or are
  cancelled;
- no background task is silently detached by default.

The program entry and combinators provide managed execution with default async
handling. `std.task.run(scheduler => { ... }, implementation = DefaultScheduler)`
creates an explicit owned scope with the selected implementation. The default
value belongs to std; callers need not import it for the one-argument form.
The callback's `scheduler` is a fresh running `SchedulerHandle`, not that
reusable implementation value. `std.task.start` is permitted only in a
managed scope; it is not unrestricted process/thread creation. Tasks inherit
the active handler environment, trace specs, runtime policy, limits, cancellation
scope, and trace links at their start site. Owner/lifetime requirements must
survive helper calls and package interfaces.

Combinators inside a run inherit its selected policy while retaining their own
child-group lifetime and cancellation boundaries. An explicit nested `run`
selects its own implementation; omitting its argument selects the std default,
not an ambient mutable global. The root managed execution uses that default
when there is no explicit selection. The behavioral `Scheduler` spec, default
value, and protected execution contract are specified in
[Scheduler Spec And Default Value](19-standard-library.md#24-scheduler-spec-and-default-value).

Active instances are scope-owned lightweight concurrent entities, not OS
processes or dedicated threads. Construction registers a mailbox; methods
execute as coroutine requests. Scope exit stops admission, cancels/joins active
requests and autonomous work, then destroys instances and owned descendants.
An idle mailbox does not make ordinary scope completion wait forever.
A scoped reference may be shared, but copying it does not reparent the instance.
See [Agent Ownership](03-agents-tools-prompts-memory.md#15-ownership-budgets-and-supervision).

## 3. Standard Combinators

`std.task` owns these APIs; the prelude re-exports the six combinator names.
They schedule closures, not already-completed values. Callback effects,
requested actions, captures and trace summaries are preserved. `limits` is an
ordinary parameter name, not a keyword or annotation. It establishes the
combinator's checked budget scope; an enclosing `@limits(...)` scope still
applies, and concurrent branches share its aggregate allowance.

```etas
public enum TaskError {
    EmptyRace,
    InvalidConcurrency,
}
```

| `std.task` API | Result / Escaping Row | Completion Contract |
|---|---|---|
| `join<T, effect E>(jobs: Array<() -> T ![E]>, limits: Array<Limit> = [])` | `Array<T> ![E, Error<TaskError>, Error<BudgetExceeded>]` | Input-ordered results; first unhandled failure initiates sibling cancellation. |
| `try_join<T, X, effect E>(jobs: Array<() -> T ![E, Error<X>]>, limits: Array<Limit> = [])` | `Result<Array<T>, X> ![E, Error<TaskError>, Error<BudgetExceeded>]` | Same fail-fast scheduling, but capture the common `Error<X>`. |
| `collect<T, X, effect E>(jobs: Array<() -> T ![E, Error<X>]>, limits: Array<Limit> = [])` | `Array<Result<T, X>> ![E, Error<TaskError>, Error<BudgetExceeded>]` | Wait for every branch; capture each selected `Error<X>`; input-ordered output. |
| `race<T, effect E>(jobs: Array<() -> T ![E]>, limits: Array<Limit> = [])` | `T ![E, Error<TaskError>, Error<BudgetExceeded>]` | First success; cancel/clean up losers. If all fail recoverably, propagate the primary original typed error. |
| `map_concurrent<A, B, effect E>(values: Array<A>, f: A -> B ![E], limits: Array<Limit> = [])` | `Array<B> ![E, Error<TaskError>, Error<BudgetExceeded>]` | Bounded fan-out, input-ordered results, fail-fast errors. |
| `map_concurrent_collect<A, B, X, effect E>(values: Array<A>, f: A -> B ![E, Error<X>], limits: Array<Limit> = [])` | `Array<Result<B, X>> ![E, Error<TaskError>, Error<BudgetExceeded>]` | Bounded fan-out, one result per input; selected typed errors do not cancel other items. |

| Parameter / Boundary | Rule |
|---|---|
| `limits = []` | Inherit the active scope's limits and scheduler; it does not remove parent budgets or create an unbounded detached scope. |
| `Concurrency(n)` | Positive local admission cap; duplicate/nested caps combine by minimum. Invalid constants are rejected statically; a dynamic zero/out-of-range cap raises `InvalidConcurrency` before starting jobs, including on empty input. |
| Other dimensions | Existing `Limit` composition/accounting rules; siblings share budgets, including tokens and wall time. Execution-limit exhaustion retains its normal termination semantics. |
| Admission | Do not start all work before applying the cap; enqueue pending closures and start at most the admitted number. Started descendants remain owned until cleanup. |
| Empty input | `join`, `collect` and mapping return `[]`; `try_join` returns `Ok([])`; `race` raises `EmptyRace`. Validate options first. |
| Error type `X` | Select one concrete error family from context/annotation or unique inference; factor `Error<X>` out of `E`. Multiple candidates without a selection are ambiguous, not an inferred union. |
| Other errors | Other `Error<Y>` in `E` escape and fail the scope. A surviving parent may collect a child's boundary-delivered `Error<BudgetExceeded>` when X selects that family. Cancellation, hard faults and termination of the collecting scope itself are not per-item `Err` results. |
| Race timing | Defer recoverable failed branches while a success remains possible. Same-dispatch-turn successes use lowest source index; all-failed selection follows section 10. |
| Retry / result reuse | No automatic retry/resample. Each admitted closure is invoked once; completion waits for loser/sibling cleanup before returning. |
| Scope | Install the group completion policy before starting children; wait-all/race must not inherit the parent's fail-fast handling for the errors they explicitly collect/defer. |

Fixed heterogeneous tuples are also accepted by `join`, `try_join` and
`collect`. This is a compiler-known tuple signature scheme, not new variadic
generic syntax. For branches `job_i: () -> T_i ![E_i]`:

| Tuple Operation | Output Shape |
|---|---|
| `join((job_1, ..., job_n))` | `(T_1, ..., T_n)`; union of branch rows plus `Error<TaskError>` and `Error<BudgetExceeded>`. |
| `try_join((job_1, ..., job_n))` | `Result<(T_1, ..., T_n), X>`; one selected common error family, as above. |
| `collect((job_1, ..., job_n))` | `(Result<T_1, X>, ..., Result<T_n, X>)`; capture the same selected error family in every branch. |
| Empty tuple | Corresponding `()`, `Ok(())`, `()` after option validation. |
| `race` / mapping | Homogeneous array surface only; adapt heterogeneous values into an ordinary enum explicitly. |

```etas
let reports = std.task.map_concurrent(
    papers,
    paper => paper ~> PaperSummarizer,
    limits = [Concurrency(8), Tokens(120_000)]
);

let (profile, history) = std.task.join((
    () => LoadProfile(user),
    () => LoadHistory(user),
));
```

## 4. Type And Effect Rules

For a fixed tuple of branches:

```text
branch_i : () -> T_i ![E_i]
----------------------------------------
join((branch_1, ..., branch_n))
  : (T_1, ..., T_n) ![E_1 union ... union E_n, Error<TaskError>, Error<BudgetExceeded>]
```

For homogeneous collection branches:

```text
branch_i : () -> T ![E_i]
----------------------------------------
join([branch_1, ..., branch_n])
  : Array<T> ![E_1 union ... union E_n, Error<TaskError>, Error<BudgetExceeded>]
```

For concurrent mapping:

```text
f  : A -> B ![E]
xs : Array<A>
----------------------------------------
map_concurrent(xs, f) : Array<B> ![E, Error<TaskError>, Error<BudgetExceeded>]
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

## 5. Effect-Free Concurrency

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

## 6. Data Dependencies

Effect freedom does not remove data dependencies:

```etas
let a = parse(input);
let b = analyze(a);
```

`analyze(a)` depends on `parse(input)`, so the two calls cannot be reordered into
parallel execution even if both are effect-free.

The compiler should use ordinary data-dependency analysis before scheduling
branches concurrently.

## 7. Effect And Resource Conflicts

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

## 8. Policy Happens-Before

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

## 9. Limits And Budgets

Concurrent scopes share parent limits. Child branches must not receive fresh
budgets merely because they run in parallel.

```etas
let result = join((
    () => topic ~> Researcher,
    () => topic ~> CompetitorResearcher,
), limits = [
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
- exhaustion stops the owning scope and its unfinished branches, completes
  bounded cleanup, then delivers `Error<BudgetExceeded>` outside that scope.
  An inner handler cannot keep the exhausted scope alive. See the canonical
  [limit outcome table](04-flows-human-gates-and-protocols.md#1-flow).

## 10. Error And Cancellation Semantics

The default task scope and fail-fast combinators use these semantics:

| Combinator | Error behavior | Cancellation behavior |
|---|---|---|
| `join` | Propagate the deterministic primary unhandled error | Cancel unfinished siblings and wait for cleanup |
| `try_join` | Return `Err(primary)` for its common typed error | Cancel unfinished siblings and wait for cleanup |
| `collect` | Convert each branch to `Result<T, E>` | Does not cancel siblings just because one branch fails |
| `race` | Return first success; if all fail, propagate the deterministic primary error; empty input raises `TaskError.EmptyRace` | Cancel losing branches after success |
| `map_concurrent` | Propagate the deterministic primary error | Cancel unfinished items and wait for cleanup |
| `map_concurrent_collect` | Return one input-ordered `Result` per item | Does not cancel siblings just because one item fails |

`std.task.run` uses the same fail-fast rule. Inherited handlers may recover
within a child; an error that remains unhandled at the child boundary fails the
owning scope. `join` and `try_join` have the same fail-fast behavior here;
`collect`, not a silent `join` variation, provides per-branch results. Result
collection requires a declared common error type or explicit error mapping;
it does not change the single-Error rule of `?` or invent implicit error unions.
The original typed failure is propagated after cancellation and cleanup, not
fabricated as a normal task result. `try_join`, `collect` and
`map_concurrent_collect` explicitly capture their selected error family as
specified in section 3; they cannot catch shutdown from inside their own
exhausted scope. Competing failures and the selected
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

`DefaultScheduler` uses a FIFO ready queue. Starting, yielding, or waking
appends a task to the queue, and each dispatch runs one cooperative quantum.
A yielding task cannot starve older ready work. External host completion timing
remains nondeterministic and is captured by the structured trace for replay.

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

## 11. Handlers In Concurrent Scopes

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

Constructor `@inject` requests use these same scoped handlers in each task;
they are not thread-local/global registry lookups. Concurrent constructions
request independently and obey normal capture, cancellation and lifetime
checks. Saved fields retain their values across later calls; injecting a shared
agent reference neither reparents it nor shares one caller's grants.

### 11.1 Handler-Based Scheduling

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

### 11.2 Ownership And Transfer

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

### 11.3 Waiting And Wakeup

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
The standard handler adapter transfers work to the scope's selected policy.
`DefaultScheduler` chooses FIFO ready order; a custom `Scheduler` implementation
can choose another order without replacing wait-registration or result checks.
A custom handler can also change control behavior through the protected
transfer APIs, not the validity of task results or authorization of host work.

### 11.4 Handler Lifetime And Early Finish

Resuming a task reinstalls the captured handler environment (deep resumption).
Handler arms themselves follow ordinary enclosing-handler lookup rules. A
handler frame cannot disappear while inherited child work or a parked
continuation still depends on it. Normal managed scope exit settles that work
before committing the answer; task handles cannot escape a shorter-lived
handler environment they still depend on.

Each action invocation has its own arm/continuation. Arms may interleave;
there is no implicit global lock held across an arm's wait. Same-task lambda
and Handler captures preserve their mutable lexical bindings. A callable
retaining a local `var` is task-local, including when stored in another
callable or value container. Child-task inheritance and concurrent invocation
must enforce that ownership. Independent task input can capture an immutable
snapshot. Shared resources use explicit APIs and effect/resource conflict
checks. The full capture rules are in
[Lambda And Handler Capture](20-lexical-capture.md).

`finish value;` targets its enclosing handled expression, not whichever child
happens to be executing the arm. If that expression encloses a concurrent
scope, finishing it cancels and cleans up the unfinished child work it owns
before delivering its answer. A handler local to one child finishes only that
child's handled region. Competing finish/error/cancellation decisions cannot
complete the same region twice. No explicit continuation can resume a region
after it has finished.

### 11.5 Context And Analysis Preservation

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

### 11.6 Complete Example

This design example uses the standard task protocol and optional `as k`. It is
not a claim that the current interpreter implements these APIs. The two jobs
are started before either is awaited. It explicitly selects a custom ready
policy and writes the handler adapter; business code normally omits both and
uses the standard default.

```etas
import std.task;
import std.task.Async;
import std.task.scheduler.{Scheduler, ReadyTask};

type NewestFirst = {};

impl NewestFirst ~ Scheduler {
    flow select(var self, ready: Array<ReadyTask>) -> usize ![] {
        // The scheduler contract supplies a non-empty snapshot.
        return ready.len() - 1;
    }
}

@model("gpt-5.5")
agent Reviewer = {};
impl Reviewer {
    public flow run(self, request: string) -> string {
        perform Async.yield();

        let prompt = Prompt.new()
            .system(Trusted("Review the request and report concrete issues."))
            .data(request);

        return perform infer(prompt);
    }
}

flow main(args: Array<string>) -> i32 {
    return std.task.run(scheduler => {
        handle {
            let correctness = std.task.start(
                () => (Reviewer {}).run("Check correctness")
            );
            let security = std.task.start(
                () => (Reviewer {}).run("Check security")
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
    }, NewestFirst {});
}
```

The unit value queued by `requeue` matches `Async.yield`'s result type. `park`
preserves the relationship between a task result and the captured await
continuation. Failure is propagated by the task protocol, not passed to that
continuation as a fake string. Unhandled task or output errors fail execution
instead of returning the normal exit code `0`; application-specific recovery
can use ordinary outer handlers. The example does not add an aborting `?` or
change the existing Error-to-Result semantics.

Omitting `NewestFirst {}` uses `DefaultScheduler` without an extra import.
Omitting the explicit `handle ... with ...` layer uses the standard handler
adapter automatically. Ready-order selection is independent of that adapter;
neither choice grants authority or permits a task/continuation to escape.

### 11.7 Active Agent Requests

| Boundary | Contract |
|---|---|
| Call | `a.method(x)` submits a typed request and suspends the caller without blocking a host thread. `start(() => a.method(x))` permits other work before awaiting the reply. |
| Admission | Preserve request identity, caller handlers/restrictions, accounts, cancellation and causal links. Do not reuse another client's approvals. |
| Updating request | One pending state per instance; exclusive update right survives suspension. Only normal completion commits, once before reply. |
| Read request | Committed snapshot at admission; may overlap updates but never observes pending state. |
| Upward action | Dispatch to the nearest applicable request handler, then its caller environment; not automatically to the lifecycle owner. |
| Handler safety | Cross-request handlers cannot capture caller-local `var` or a live updating receiver. Handler frames remain live until associated requests are cleaned up. |
| Resume / finish | Resume exactly the suspended request. Finish unwinds the handled subtree and discards its pending updates; unrelated requests and prior commits survive. |
| Reentrancy | No automatic updating reentrancy. Direct same-turn helper calls share pending state; external-reference callbacks are separate requests. |
| Deadlock | Reject proven unavoidable waits statically; detect concrete closed request-wait cycles at runtime and fail the affected structured execution. Arbitrary communication graphs need not be acyclic. |
| Scheduler/budgets | A blocked request releases its runnable slot, not its update right. Descendants share applicable accounts; execution stacks do not reset quotas. |
| Lifecycle | Autonomous work is owned and cancelled; replacement/retry is explicit. A shared reference is not supervision ownership. |

The runtime traces instance, request, parent-request and committed-state-version
identities. Replay must preserve request order, admission snapshots, commit and
cancel outcomes; recording only model replies is insufficient. Pure snapshots
do not make effects through stored resource references conflict-free.

## 12. Trace And Replay

Concurrent traces retain task/scope identity, scheduler selection and decisions,
action/handler events, continuation transfers, wakeup winners, budget usage and
cancellation/failure outcomes. Replay preserves the recorded logical ordering;
it need not reproduce wall-clock timing.

`Task`, `SchedulerHandle` and continuation handles cannot be serialized,
hashed, compared, persisted or supplied to models/hosts as ordinary values.
Checkpoint restoration must preserve their scope/ownership and single-shot
state, not expose them through source-level decoding. Reusable values satisfying
`Scheduler` are distinct from live handles.

Any scheduling transformation must preserve dependencies, action ordering,
handler behavior, limits and replay obligations. Missing proof does not permit
reordering observable effects.
