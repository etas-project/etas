# Flows, Human Gates, and Protocols

## 1. Flow

### 1.1 Flow as Callable Control Flow

A flow is Etas's user-defined callable unit. It may be deterministic local computation, non-deterministic computation, or non-deterministic orchestration. It can contain agent calls, compose other flows, call tools, use typed memory APIs, pass human gates, use bounded loops, and satisfy optional protocol constraints.

```etas
flow PaperWriting(topic: string) -> PaperDraft
{
    let plan = Planner.run(topic);

    let (related, formal, eval) = join((
        () => Researcher.run(plan),
        () => Theorist.run(plan),
        () => Evaluator.run(plan),
    ));

    let draft = Writer.run({ plan, related, formal, eval });

    let review = Reviewer.run(draft);

    if review.score < 7 {
        draft = Writer.run({ draft, review });
    }

    return draft;
}
```

### 1.2 Flow as a First-Class Value

A declared flow is both a callable definition and a first-class value. It can be passed as an argument, returned from another flow, stored in records, and composed with other flows.

A simplified surface flow type annotation can be written as:

```text
Input -> Output ![Effects]
Input -> Output
```

The compiler normally infers flow types, then normalizes them to the canonical internal type `Flow<Input, Output, Effects>`.

For example:

```etas
flow SummarizePaper(paper: Paper) -> Summary
{
    return Summarizer.run(paper);
}

flow SummarizeAll(
    step: Paper -> Summary,
    papers: Array<Paper>
) -> Array<Summary>
{
    var summaries: Array<Summary> = [];

    for paper in papers
        limit Iterations(20), Tokens(20_000)
    {
        summaries = summaries.push(step(paper));
    }

    return summaries;
}
```

Calling a flow value is effect-checked like calling a named flow. If a flow receives another flow as an argument, the caller must account for the callee's inferred or declared effects.

Flows can also be returned to build small reusable strategies:

```etas
flow choose_writer(fast: bool) -> Outline -> Draft {
    if fast {
        FastWriterFlow
    } else {
        CarefulWriterFlow
    }
}
```

Returning a flow value does not execute it. Flow execution happens when another flow calls it or when the runtime selects it as the program entry.

This keeps composition in the language without introducing a separate orchestration DSL.

### 1.3 Stage Composition

Etas also supports a stage composition operator:

```etas
let pipeline = Researcher | Writer | Publisher;
```

The operands of `|` are stages, and the operator is left-associative. A stage may be an `agent`, a `flow`, or a `tool`, as long as the output type of the left stage matches the input type of the right stage:

```text
A ~ Stage<I, M, E1>
B ~ Stage<M, O, E2>
----------------
A | B : I -> O ![E1 + E2]
```

The result is always a flow value, not an `agent`. A composition such as `Researcher | Writer | Publisher` represents multiple runtime steps with separate prompts, model calls, schema validation, trace events, effect/action checks, and policy checks.

Examples:

```etas
let draft_pipeline = Researcher | Writer;
// draft_pipeline : Topic -> Draft

let checked_pipeline = Researcher | sanitize_notes | Writer;
// checked_pipeline : Topic -> Draft

let publish_pipeline = Researcher | Writer | publish.write;
// publish_pipeline : Topic -> PublishedPost
```

The effect set of a composed flow is the union of the stage effects. Its determinism class is the maximum of the stage classes:

```text
Deterministic < NonDeterministic
```

So any composition containing an agent or another non-deterministic flow is itself `NonDeterministic`.

`|` composes stages into a flow value. `~>` applies a value to a stage or composed flow:

```etas
let pipeline = Researcher | Writer | Publisher;
let post = topic ~> pipeline;

let same_post = topic ~> (Researcher | Writer | Publisher);
```

### 1.4 Concurrent Composition

Etas does not need a dedicated `parallel` statement in the MVP. Concurrent flow or tool composition can be expressed by a standard library combinator:

```etas
let (a, b, c) = join((
    () => StepA(input),
    () => StepB(input),
    () => StepC(input),
));
```

`join` is a library flow with runtime support. The compiler may still lower independent branches into AIR regions that can be scheduled concurrently.

The full concurrency model, including effect/resource conflicts, policy
ordering, limits, cancellation, and trace semantics, is specified in
[Concurrency](16-concurrency.md).

### 1.5 Graph View

The compiler can extract a graph:

```text
Planner
  ├── Researcher
  ├── Theorist
  └── Evaluator
        ↓
      Writer
        ↓
      Reviewer
        ↓
   conditional revise
```

This graph becomes the basis for static analysis, scheduling, optimization, and runtime execution.

### 1.6 Flow Effects

A flow infers the union of effects it may perform. Local flows normally omit `![...]`:

```etas
flow W(...) -> T {
    ...
}
```

At public API, package, or deployment boundaries, a flow may still expose an explicit or generated effect signature. If an explicit `![...]` contract is present and the body performs an effect outside that set, compilation fails.

### 1.7 Control Structures

Etas supports ordinary deterministic control flow inside flows:

```etas
if condition {
    ...
} else {
    ...
}

for paper in papers {
    index.add(paper);
}

while not done
    limit Iterations(10)
{
    done = Checker.run(state);
}
```

The important distinction is that loops may contain agent calls and compose other flows. They may also contain tool calls, typed memory access, and human gates. Non-deterministic loops are part of the analyzable flow graph rather than opaque runtime callbacks.

### 1.8 Bounded and Budgeted Loops

Unbounded non-deterministic loops are unreliable because they may never converge and can consume unlimited tokens, context, money, time, tool calls, approval requests, or memory API operations. Etas requires any loop whose termination cannot be statically proven to declare an explicit `limit`.

```etas
while review.score < 8
    limit Iterations(3), Tokens(40_000), Cost(usd(2.00))
{
    draft = Writer.run({ draft, review });
    review = Reviewer.run(draft);
}
```

`for` loops can also declare limits. This is useful when the collection comes from retrieval, model output, a stream, or any source whose size should be capped by policy:

```etas
for paper in papers
    limit Iterations(20), Tokens(20_000)
{
    summaries = summaries.push(Summarizer.run(paper));
}
```

A `for` loop over values with statically known or runtime-bounded size may omit `Iterations(...)`, but it may still need budget limits if the body performs expensive effects. If both the collection size and `Iterations(...)` are present, the loop processes at most the smaller of the two bounds.

Retry and validation patterns should use `while` with an explicit limit:

```etas
while true
    limit Iterations(3), ContextTokens(128_000)
{
    let review = Reviewer.run(draft);

    if review.accepted {
        break;
    }

    draft = Writer.run({ draft, review });
}
```

Limit dimensions are typed runtime support constructors, not keywords:

| Limit | Meaning |
|---|---|
| `Iterations(n)` | Maximum number of loop iterations |
| `Tokens(n)` | Maximum model input and output tokens consumed in the loop |
| `ContextTokens(n)` | Maximum context window used by any agent call in the loop |
| `Cost(money)` | Maximum monetary cost |
| `WallTime(duration)` | Maximum wall-clock time |
| `Attempts(n)` | Maximum retry attempts |

Multiple limits are conjunctive. A loop may continue only while all declared limits still allow the next iteration or effect. If the same limit dimension appears more than once, the effective bound is the stricter one.

```etas
for paper in papers
    limit Iterations(20), Tokens(20_000)
{
    summaries = summaries.push(Summarizer.run(paper));
}
```

In this example, `Iterations(20)` caps the number of papers processed. The token budget is an additional constraint, not an alternative.

Conceptually:

```text
continue =
  condition_or_next_item_exists
  && Iterations_remaining
  && Tokens_remaining
  && Cost_remaining
  && WallTime_remaining
```

For `for`, an `Iterations(...)` limit caps how many elements are consumed and normal completion means "processed at most this many items." For `while`, exhausting an iteration limit while the condition still holds performs `Error<BudgetExceeded>.raise(...)` by default, because the loop did not establish its intended postcondition. Budget limits such as `Tokens(...)`, `ContextTokens(...)`, `Cost(...)`, and `WallTime(...)` perform the same error operation when exhausted unless the flow explicitly handles it.

The compiler uses limits for static cost analysis and flow graph construction. The runtime enforces limits dynamically.

Limits are primarily a reliability construct. They make non-deterministic agent execution bounded, resumable, and predictable. They also support safety by constraining high-impact operations, and they support optimization by giving the compiler and runtime explicit budgets for context slicing, retrieval planning, parallel scheduling, cheaper model selection, and early cancellation.

Typical frameworks expose similar knobs as runtime configuration, for example max iterations, request timeout, max tokens, or retry counts on an agent executor, model client, graph node, or tool wrapper. Those knobs are useful, but they are usually scattered across host-language objects and callbacks. Etas treats limits as typed execution contracts in the flow graph:

- the compiler can reject non-deterministic loops that have no statically visible bound;
- effect analysis can check that model calls, tool calls, approvals, and memory API operations are covered by active limits;
- exported flows and deployment manifests can report token, cost, wall-time, attempt, and iteration budgets before execution;
- checkpoint/resume stores remaining budget, not just local variables, so retries do not silently reset the resource contract;
- `join([...])` and other combinators can use budgets to cancel losing branches, cap fan-out, or choose sequential fallback when a parallel plan would exceed policy;
- traces can explain why execution stopped: success, ordinary control flow, policy denial, or `BudgetExceeded`.

In short, `limit` is not just a loop guard. It is the language-visible resource contract for agentic computation.

---

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
expression operator.

```etas
spec SafeAction: trace =
    +Approval.request
    & +CompanyEmail.send<WorkAccount>
    & +StripePayment.charge<BillingAccount>
    & +WorkCalendar.write<WorkCalendar>
    & (Approval.request >> CompanyEmail.send<WorkAccount>)
    & (Approval.request >> StripePayment.charge<BillingAccount>)
    & (Approval.request >> WorkCalendar.write<WorkCalendar>);
```

If a flow attempts to perform one of these effects without an approval event dominating it in the control-flow graph, the compiler or runtime rejects the program.

The same shape can express other gates:

```etas
spec PublishingSafety: trace =
    +HumanReview
    & +PublicPublish
    & +Sanitized
    & +PromptSystemWrite
    & (HumanReview >> PublicPublish)
    & (Sanitized >> PromptSystemWrite);
```

These normalize to trace-spec terms such as
`trace.before(HumanReview, PublicPublish)` and
`trace.before(Sanitized, PromptSystemWrite)`.

### 2.3 Human as a Typed Actor

Humans can participate in protocols:

```etas
protocol ApprovalProtocol {
    Agent -> Human: ProposedAction;
    Human -> Agent: Approval | Rejection;
}
```

---

## 3. Protocol

### 3.1 Motivation

Multi-agent systems are not just collections of agents. They are communicating systems. Their communication should be typed.

Etas distinguishes runtime conversation semantics from optional protocol checking:

```text
Message<T> / SessionConfig / Conversation = runtime support types for message flow, history, handoff, and context policy
protocol                                = optional static contract for allowed communication order
```

Most MVP programs should use typed `Message<T>` values and session support types without declaring a protocol. A `protocol` declaration is for advanced decentralized or long-lived interactions where the order of messages itself needs static checking.

Instead of arbitrary messages:

```text
string from A to B
```

Etas uses typed messages:

```text
Message<Payload>
```

Sender, receiver, session id, role, trust, and provenance are metadata on the runtime support value, not extra source-level keywords.

### 3.2 Protocol Declaration

```etas
protocol ReviewLoop {
    Writer -> Reviewer: Draft;
    Reviewer -> Writer: Review;

    choice {
        accept:
            Writer -> Chair: FinalDraft;

        revise:
            Writer -> Reviewer: RevisedDraft;
    }
}
```

### 3.3 Protocol Checking

A flow can declare protocol conformance with `~`:

```etas
flow ReviseUntilAccept(draft: Draft) -> FinalDraft
    ~ ReviewLoop
{
    ...
}
```

The compiler checks:

1. all required messages are sent;
2. messages have the correct payload type;
3. no agent receives a message that may never be sent;
4. no illegal handoff occurs;
5. branches respect the declared choices;
6. the flow has no protocol-level deadlock;
7. terminal states are reachable.

---
