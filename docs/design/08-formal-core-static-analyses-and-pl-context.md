# Formal Core, Static Analyses, and PL Context

## 1. Formal Core Calculus

Etas can be formalized through a smaller core calculus. This section is not the user-facing source syntax. It defines the internal calculus that source Etas lowers into for semantics and static analyses.

### 1.1 Core Syntax

```text
e ::= x
    | v
    | lambda x => e
    | e1 e2
    | let x = e1 in e2
    | if e then e1 else e2
    | for x in e1 do e2
    | while e1 do e2
    | budget(b, e)
    | agentcall A e
    | flowcall F e
    | toolcall t e
    | memread r k
    | memwrite r k v
```

The user-facing source syntax is defined in [General Programming Constructs](02-general-programming-constructs.md), [Flows, Human Gates, and Protocols](04-flows-human-gates-and-protocols.md), and summarized in [Syntax Principles](09-syntax-principles.md). Forms such as `agentcall`, `flowcall`, `toolcall`, `memread`, and `memwrite` are internal core forms, not surface keywords. `memread` and `memwrite` are the lowered form of typed `std.memory` APIs and `Memory.read<R>` / `Memory.write<R>` actions; they are not source-level `memory` syntax. Prompt construction is an ordinary library call that returns a typed `Prompt` value. Human approval is represented by the standard `Approval.request` action and ergonomic library wrappers such as `approve(...)`.

Checkpointing is intentionally not a core expression form. Explicit checkpoints are ordinary standard-library calls such as `runtime.checkpoint(state)` that lower to runtime checkpoint metadata. The runtime may also insert automatic checkpoint boundaries.

### 1.2 Values

```text
v ::= n
    | true | false
    | string
    | record
    | closure
    | agent
    | tool
    | memory-region-ref
    | store-ref
```

### 1.3 Limits

Resource annotations are part of the static and dynamic semantics, not comments.
Source `@limits(bounds)` elaborates to the internal `budget(b, e)` scope around
the annotated computation, including the entire loop or retry when applicable.
`budget` is calculus notation, not a source keyword. Local bound expressions
are checked as deterministic, non-suspending and action-free, evaluated once
before entry under the parent budget, and validated before executing `e`.
Declaration bounds are static; their scopes are entered per invocation.

```text
b ::= LimitValue
    | b, b

LimitValue ::= Iterations(n)
             | Tokens(n)
             | ContextTokens(n)
             | Cost(c)
             | WallTime(t)
             | user_defined_limit(v)
```

Limit composition is conjunctive:

```text
b1, b2 = b1 ∧ b2
```

When two limits constrain the same dimension, the effective bound is the stricter bound:

```text
Iterations(50), Iterations(20) = Iterations(20)
```

A loop may be accepted when either:

1. the compiler proves that it is finite; or
2. the loop has sufficient limits for the effects in its body.

For example, a loop that performs `AgentCall` must be bounded by `Iterations(...)`
or another enforceable budget that bounds its repeated work. `ContextTokens`
alone is a per-call cap, not a decreasing bound on the number of iterations.
Nested scopes retain parent accounting; retries and checkpoint/resume do not
reset it. A potentially exhausted budget contributes the boundary-delivered
`Error<BudgetExceeded>` described in [Resource Contracts](04-flows-human-gates-and-protocols.md#18-resource-contracts-and-budgeted-loops).

### 1.4 Configurations

A runtime configuration may be:

```text
⟨e, σ, μ, κ, β, τ⟩
```

where:

| Symbol | Meaning |
|---|---|
| `e` | expression |
| `σ` | ordinary store |
| `μ` | memory store |
| `κ` | effect boundary environment |
| `β` | remaining loop and flow budgets |
| `τ` | trace |

### 1.5 Agent Call Rule

An agent call executes its resolved method body in the agent's checked context.
For the default entry, the core elaboration is schematically:

```text
agentcall A v  ~~>  flowcall A.run v
                   with A's checked agent/request context
```

This does not sample a model merely because `A` is an agent. A method may be
deterministic. Model sampling occurs at `Agentic.step<C, O>`; the default
`Agentic.infer<C, O>` controller combines steps and checked tool invocations.
Those boundaries perform schema, policy, trace, and budget checks as specified
in [Agentic Runtime Actions](06-effect-system-and-inference.md#12-agentic-runtime-actions).

### 1.6 Tool Call Rule

```text
⟨toolcall t v, σ, μ, κ, β, τ⟩
  ↦
⟨v', σ', μ', κ, β', τ · ToolCall(t, v, v')⟩
```

only if:

```text
κ ⊢ allowed(t)
```

This predicate abbreviates checks of the resolved tool contract, arguments,
transitive actions, active trace specs/policy, and runtime grants. It is not a
second tool-name-based permission mechanism. Calling a helper flow cannot
bypass the same checks on the actions it requests.

Otherwise:

```text
ActionDenied
```

or, if the relevant budget is exhausted:

```text
BudgetExceeded
```

### 1.7 Approval Action Rule

```text
approve : ApprovalRequest -> ApprovalDecision ![Approval.request]
```

After type checking, the ergonomic support flow lowers to the standard approval action:

```text
⟨call approve a, σ, μ, κ, β, τ⟩
  ↦
⟨accepted, σ, μ, κ, β, τ · Action(Approval.request, a, accepted)⟩
```

or:

```text
⟨call approve a, σ, μ, κ, β, τ⟩
  ↦
⟨rejected, σ, μ, κ, β, τ · Action(Approval.request, a, rejected)⟩
```

---

## 2. Static Analyses

Etas analyses operate over possible traces without enumerating every concrete execution. A useful abstract trace summary contains:

```text
AbstractTrace = {
  may_call_tools: Set<Tool>,
  may_read_memory: Set<MemoryStore>,
  may_write_memory: Set<MemoryStore>,
  may_send_messages: Set<(Agent, Agent)>,
  may_leak_secrets: bool,
  must_human_approve: Set<Effect>,
  max_cost: CostBound,
  max_depth: i32,
  may_reach_error: Set<ErrorKind>,
}
```

### 2.1 Effect Analysis

Computes the set of possible source-level escaping effects and the broader set
of requested actions.

```text
Effects(e)          = may-escaping-effect summary after handling
RequestedActions(e) = may-requested-action summary, including handled/default actions
```

These summaries are computed separately. Handling an action can remove its
escaping obligation without removing its requested-action metadata. Neither set
records order; temporal checks use control flow and trace/transfer summaries,
not set inclusion alone.

Reject if:

```text
Effects(body) ⊄ DeclaredEffects(declaration)   // when an explicit bound is present
```

For each source `flow`, infer a determinism class:

```text
Determinism(body) =
  Deterministic
  | NonDeterministic
```

Agent calls use the callee method summary. `Agentic.infer<C, O>`/`step<C, O>` requested
actions, approval gates, non-deterministic subflows, runtime choice,
non-deterministic runtime primitive calls, Etas tool calls whose bodies are
non-deterministic, typed memory API calls, file I/O, network, time, command
effects, and other external state observations force `NonDeterministic`. Pure
local computation, including an agent method with no non-deterministic operation,
remains `Deterministic` and may lower to internal `lambda`/function forms.

### 2.2 Effect Action Analysis

Checks whether each tool call is authorized by the current effect boundary environment.

```text
EffectBoundaryEnv ⊢ toolcall t allowed
```

### 2.3 Prompt Taint Analysis

Tracks trust labels through prompt construction.

```text
Untrusted -> system prompt     rejected
Secret -> model input          rejected unless declassified
Sanitized -> user prompt       allowed
```

### 2.4 Memory Access Analysis

Checks whether agents access only allowed typed memory regions or stores. Memory access facts come from parameterized actions, as defined in [Effect System and Inference](06-effect-system-and-inference.md#13-parameterized-effects):

```text
Read(A)  = { R | Memory.read<R>  ∈ RequestedActions(A) }
Write(A) = { R | Memory.write<R> ∈ RequestedActions(A) }

Read(A)  ⊆ AllowedRead(A)
Write(A) ⊆ AllowedWrite(A)
```

Containment uses memory-place hierarchy. A trace spec or runtime policy over
`ProjectMemory` covers `ProjectMemory.Papers`, but one over
`ProjectMemory.Papers` does not cover `ProjectMemory.Drafts`.

### 2.5 Protocol Analysis

Checks that communication traces conform to declared protocols.

```text
TraceMessages(W) ∈ Language(Protocol)
```

### 2.6 Temporal Trace-Spec Analysis

Checks temporal properties over traces.

Examples:

```text
G(CompanyEmail.send<WorkAccount> -> previously accepted-matching-approval)
G(Command -> has SandboxProfile)
G(ClaimCreated -> eventually CitationVerified)
```

Here `accepted-matching-approval` is a predicate over successful, relevant
approval evidence, not a new source action. A request that was denied, failed,
or concerned another payload does not establish it.

### 2.7 Cost Analysis

Computes upper bounds or estimates:

```text
MaxTokens(W)
MaxCost(W)
MaxTime(W)
MaxIterations(W)
```

A loop needs a proven finite bound or an enforceable budget when termination
cannot be proved. Widening makes abstract analysis converge; it is not itself
evidence that the concrete loop terminates or stays within a resource bound.

### 2.8 Loop Limit Analysis

| Analysis Fact | Resource Check |
|---|---|
| Proven finite loop | No explicit iteration cap is required; inherited token, cost and time contracts still apply. |
| Termination not proved | Require an enforceable `@limits(...)` contract covering repeated work, not just a fresh budget inside each iteration. |
| Requested model/tool/memory actions | Check resource admission and trace-spec/runtime-policy constraints even when default handlers remove escaping effects. |
| Empty escaping row | Does not prove termination, purity or absence of requested actions. |

The analysis uses checked behavior summaries, not a synthetic `Pure` effect tag.
Dimension and exhaustion semantics are defined by the resource contract rather
than duplicated in the effect-set analysis.

---

## 3. Relationship to Existing Programming Language Ideas

Etas connects naturally to many PL concepts:

| PL Concept | Etas Interpretation |
|---|---|
| Effect systems | Tool and external action control |
| Authority systems | Authority management for agents |
| Session types | Multi-agent communication protocols |
| Schema validation | Structured output constraints |
| Information-flow control | Prompt injection and secret leakage prevention |
| Abstract interpretation | Static analysis over possible agent traces |
| Operational semantics | Formal execution model for flows |
| Program slicing | Context minimization for agent calls |
| Partial evaluation | Prompt specialization |
| Compiler IR | Agent Intermediate Representation |
| Runtime monitoring | Dynamic trace-spec/runtime-policy enforcement |
| Type-directed compilation | Backend-independent agent flow generation |

---
