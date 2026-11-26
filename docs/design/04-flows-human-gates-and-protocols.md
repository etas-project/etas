# Flows, Human Gates, and Protocols

## 1. Flow

### 1.1 Calls And Values

Flows are first-class callables with inferred return/effect types. Construction
of a flow value does not execute it; calls preserve its latent behavior and
capture constraints. Declaration and inference rules are in
[General Constructs](02-general-programming-constructs.md#4-flows-as-the-ordinary-callable-unit).

### 1.2 Stage Composition

Etas also supports a stage composition operator:

```etas
let pipeline = Researcher | Writer | Publisher;
```

The operands of `|` are stages, and the operator is left-associative. A stage may be an `agent`, a `flow`, or a `tool`, as long as the output type of the left stage matches the input type of the right stage:

```text
stage(A) ~ Stage<I, M, E1>
stage(B) ~ Stage<M, O, E2>
----------------
A | B: I -> O ![E1 + E2]
```

For an empty-state agent type with an explicitly defined public `run`, stage
conversion constructs an instance per invocation; a bound instance stage reuses
its reference and scope. Stateful agents require explicit construction. Neither
form hides creation, instance-access or state-dependence metadata.

The result is a flow value, not a new nominal `agent`. Composition preserves
the selected stage methods and their behavior summaries; prompts and model
calls occur only where those methods actually request them.

Examples:

```etas
let draft_pipeline = Researcher | Writer;
// draft_pipeline: Topic -> Draft

let checked_pipeline = Researcher | sanitize_notes | Writer;
// checked_pipeline: Topic -> Draft

let publish_pipeline = Researcher | Writer | publish.write;
// publish_pipeline: Topic -> PublishedPost
```

The effect set of a composed flow is the union of the stage effects. Its determinism class is the maximum of the stage classes:

```text
Deterministic < NonDeterministic
```

Any composition containing a stage whose method/body summary is
`NonDeterministic` is itself `NonDeterministic`. An agent's nominal identity
alone does not change this classification.

`|` composes stages into a flow value. `~>` applies a value to a stage or composed flow:

```etas
let pipeline = Researcher | Writer | Publisher;
let post = topic ~> pipeline;

let same_post = topic ~> (Researcher | Writer | Publisher);
```

### 1.3 Concurrent Composition

Etas does not need a dedicated `parallel` statement. Concurrent flow or tool composition can be expressed by a standard library combinator:

```etas
let (a, b, c) = join((
    () => StepA(input),
    () => StepB(input),
    () => StepC(input),
));
```

`join` is a structured standard-library combinator, not a keyword.

The full concurrency model, including effect/resource conflicts, trace-spec/runtime-policy
ordering, limits, cancellation, and trace semantics, is specified in
[Concurrency](16-concurrency.md).

### 1.4 Resource Contracts and Budgeted Loops

Unbounded non-deterministic loops may never converge and can consume unbounded
resources. Etas requires a loop whose termination cannot be statically proven
to declare an enforceable `@limits(...)` contract. An annotation inside the
loop body alone does not bound the whole loop.

```etas
@limits([Iterations(3), Tokens(40_000), Cost(usd(2.00))])
while review.score < 8 {
    draft = (Writer {}).run({ draft, review });
    review = (Reviewer {}).run(draft);
}
```

`for` loops can also declare limits. This is useful when the collection comes from retrieval, model output, a stream, or any source whose size should be capped by runtime policy:

```etas
@limits([Iterations(20), Tokens(20_000)])
for paper in papers {
    summaries = summaries.push((Summarizer {}).run(paper));
}
```

A `for` loop over values with statically known or runtime-bounded size may omit `Iterations(...)`, but it may still need budget limits if the body performs expensive effects. If both the collection size and `Iterations(...)` are present, the loop processes at most the smaller of the two bounds.

Retry and validation patterns should use `while` with an explicit resource annotation:

```etas
@limits([Iterations(3), ContextTokens(128_000)])
while true {
    let review = (Reviewer {}).run(draft);

    if review.accepted {
        break;
    }

    draft = (Writer {}).run({ draft, review });
}
```

`@limits(bounds)` is a compiler-recognized semantic annotation; neither `limits`
nor `limit` is a keyword. Its argument is `Array<Limit>`, built from typed std
support values. Resource scopes apply to executable declarations, `for`,
`while`, `retry`, and blocks; no postfix pipeline-stage resource clause exists.

| Scope / Evaluation | Contract |
|---|---|
| Declaration | Arguments are compile-time values. Each invocation establishes a child scope enclosing the body and trailing handler, not a global counter shared by every invocation. |
| Local loop, retry, or block | Arguments may read parameters/local values and call deterministic, non-suspending, action-free helpers. Evaluate once, left to right, under enclosing budgets before scope entry; freeze the resulting bounds for this activation. |
| Loop / retry lifetime | The scope encloses iterable/condition evaluation and all iterations or attempts. Neither a back-edge nor `continue` resets its budget. `Iterations` counts entries into the annotated loop body, not nested-loop entries. |
| Block lifetime | Covers all work in the block, including calls and owned child tasks. Normal completion, `return`, `break`, and error unwinding leave the scope with ordinary structured cleanup. |
| Nested scopes | Every active bound applies. Consumptive usage is charged to each enclosing scope; a new child budget never replenishes its parent. Constraints on the same measured work use the stricter remaining allowance; iteration counters remain attached to their own loops. |
| Concurrent children | Share the enclosing aggregate budget, not one copy of it per child. Admission/accounting must prevent concurrent overspending. |
| Multiple annotations | Conjoin their bounds; duplicate dimensions use the stricter bound. `@limits([])` adds no bound and never removes an inherited one. |
| Checkpoint / resume | Restore scope identity, evaluated bounds and remaining counters; do not re-evaluate annotation arguments or restart accounting. |
| Validation | Reject wrong argument types, unsupported targets/dimensions and invalid constants statically. Validate dynamic values before entering the target; never ignore an invalid bound. `Iterations` requires a loop target. |
| Required coverage | Mere annotation presence is insufficient: the checker must establish that its dimensions bound the relevant repeated work. A context-window cap alone does not bound total iterations. |

For a parameter-dependent budget, use a local annotated block rather than a
runtime-valued item annotation:

```etas
flow BuildReport(topic: string, token_budget: u64) -> Report {
    return @limits([Tokens(token_budget)]) {
        let notes = (Researcher {}).run(topic);
        (Writer {}).run(notes)
    };
}
```

The annotated block retains the block's result type; it does not introduce a
closure or a new `return` target. In this example both calls consume one shared
token allowance. Limits are mandatory in every execution backend, including
native code; they are not advisory metadata that optimization may discard.

| Limit | Meaning |
|---|---|
| `Iterations(n)` | Maximum entries into the annotated loop body |
| `Tokens(n)` | Maximum cumulative model input and output tokens in the scope |
| `ContextTokens(n)` | Maximum context window used by any agent call in the scope |
| `Cost(money)` | Maximum cumulative monetary cost |
| `WallTime(duration)` | Maximum elapsed wall-clock time for the scope |
| `Attempts(n)` | Maximum retry attempts under the retry/agent execution contract |
| `Concurrency(n)` | Maximum active concurrent work under the structured-concurrency admission contract |

Multiple limits are conjunctive; duplicate dimensions use the stricter bound.

| Limit Outcome | Control / Error Contract |
|---|---|
| `for` reaches `Iterations(n)` | Normal bounded-prefix completion: process at most n elements. It does not claim to have exhausted the input. |
| `while` exhausts iterations while its condition remains true | End the bounded loop and report `Error<BudgetExceeded>` outside that loop's limit boundary. |
| Tokens, context, cost, wall time or attempts exhausted | Stop admission in the scope owning the exhausted bound, cancel its unfinished work and finish bounded cleanup. Then report `Error<BudgetExceeded>.raise(...)` to the surviving enclosing computation. |
| Handler placement | Handlers inside the exhausted scope cannot suppress its shutdown. An enclosing handler or `?` around the bounded computation may capture the delivered error; the ordinary single-Error rule still applies. |
| Declaration limits | Flow/tool/method bounds enclose each invocation and trailing handler. Agent annotations are method defaults, not automatically one aggregate lifetime budget. Put an enclosing bound around construction and use for that purpose; children retain applicable parent accounts. Handle exhaustion outside the failed scope. |
| Recovery | `Error.raise` returns `never`; recovery uses `finish` or Result matching, not resumption of exhausted work. Handling the error neither refunds usage nor widens an inherited limit. |
| Error data versus budget state | An application may raise BudgetExceeded as ordinary error data; doing so does not change counters or cancel a scope. Only runtime enforcement establishes actual exhaustion before reporting the boundary error. |
| Parent versus child | A child's own bound may fail while its parent remains live. Exhausting a shared parent bound ends the whole parent group, not merely one child. |
| Effect inference | A checked `@limits(...)` scope that may exhaust its budget contributes `Error<BudgetExceeded>` at its boundary; explicit rows constrain this inferred result. Successful bounded `for` prefix completion alone does not add that error. |
| Local API bounds | A read timeout or explicit collection-size cap may return its documented typed error without cancelling a live parent. It is not the same as exhausting the parent's execution budget. |
| Hard termination | Forced shutdown, process death and unrecoverable host faults need not deliver a source Error or complete cleanup. They are not ordinary `Err` results. |
| External actions | Completed or uncertain writes remain real. Budget failure is not rollback. |

Library parameters such as `join(..., limits = [...])` and memory-selection
`.limit(...)` retain their documented scope; their names are not syntax.
Suspension/checkpoint restoration preserves remaining budgets.

## 2. Human-in-the-Loop

### 2.1 Human Approval

Human approval is a typed Agent/runtime support API, not a dedicated source expression. `approve(...)` is parsed as an ordinary flow call whose signature carries `![Approval.request]`.

```etas
let approved = approve(
    title = "Send email to collaborator?",
    content = email_draft,
    risk = Medium
);

if approved {
    mail.send(email_draft);
}
```

### 2.2 Mandatory Approval Policies

Trace specs can require human approval before certain effects. `>>` is a
trace-spec temporal operator over typed guards and targets, not a general
expression operator. It includes local permission for both operands, so no
separate `+` declarations are needed for them.

```etas
spec SafeAction: trace =
    (Approval.request >> CompanyEmail.send<WorkAccount>)
    & (Approval.request >> StripePayment.charge<BillingAccount>)
    & (Approval.request >> WorkCalendar.write<WorkCalendar>);
```

If a flow attempts one of these actions without matching accepted approval
evidence before it, the compiler or runtime rejects the action. Calling
`Approval.request` alone is insufficient: denial, failure, or approval for an
unrelated request does not open the gate. The implicit permissions cannot
override an outer denial or runtime authority boundary.

The same shape can express other gates:

```etas
spec PublishingSafety: trace =
    (HumanReview >> PublicPublish)
    & (Sanitized >> PromptSystemWrite);
```

Each clause normalizes to local permissions for its operands and a typed
`Before` constraint, following the [Trace Specs](05-type-system-and-errors.md#154-trace-specs)
rules. Permissions do not remove the review or sanitization prerequisite.

## 3. Protocol

`protocol` declares role/message exchanges. `Message<T>`, conversation history
and session APIs remain ordinary support values and do not require a protocol.

```etas
protocol ReviewLoop {
    Writer -> Reviewer: Draft;
    Reviewer -> Writer: Review;
    Writer -> Chair: FinalDraft;
}
```

A conforming computation uses `~ ReviewLoop`; its declared exchanges must
respect payload types, participant roles and order. Protocol conformance does
not grant authority, transfer session ownership or replace trace constraints.
The source grammar here defines ordered exchanges, not an additional
`choice` keyword or unrestricted session-type language.
