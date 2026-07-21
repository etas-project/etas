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
    | for x in e1 limit b do e2
    | while e1 limit b do e2
    | agentcall A e
    | flowcall F e
    | toolcall t e
    | memread r k
    | memwrite r k v
```

The user-facing source syntax is defined in [General Programming Constructs](02-general-programming-constructs.md), [Flows, Human Gates, and Protocols](04-flows-human-gates-and-protocols.md), and summarized in [Syntax Principles](09-syntax-principles.md). Forms such as `agentcall`, `flowcall`, `toolcall`, `memread`, and `memwrite` are internal core forms, not surface keywords. Prompt construction is an ordinary library call that returns a typed `Prompt` value. Human approval is also an ordinary Agent/runtime support flow call whose type carries `![Approval]`.

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

Loop limits are part of the static and dynamic semantics, not comments.

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

For example, a loop that performs `AgentCall` must be bounded by `Iterations(...)` or another budget that the runtime can monotonically decrement.

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

Informally:

```text
⟨agentcall A v, σ, μ, κ, β, τ⟩
  ↦
⟨v', σ, μ', κ, β', τ · AgentCall(A, v, v')⟩
```

where `v'` is one possible output satisfying the declared schema and policy, and `β'` is the remaining budget after token, context, cost, and time accounting.

Because the model is non-deterministic, the transition relation is non-deterministic.

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

Otherwise:

```text
PolicyViolation
```

or, if the relevant budget is exhausted:

```text
BudgetExceeded
```

### 1.7 Approval Support Flow Rule

```text
approve : ApprovalRequest -> bool ![Approval]
```

After type checking, a call to the support flow lowers to an internal approval transition:

```text
⟨call approve a, σ, μ, κ, β, τ⟩
  ↦
⟨true, σ, μ, κ, β, τ · Approval(a, accepted)⟩
```

or:

```text
⟨call approve a, σ, μ, κ, β, τ⟩
  ↦
⟨false, σ, μ, κ, β, τ · Approval(a, rejected)⟩
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
Effects(e) = {Network, FileIO, Command, Memory.read<R>, Memory.write<R>, ...}
RequestedActions(e) = Effects(e) ∪ {Agentic.infer<C, O>, compiler-generated runtime actions, ...}
```

Reject if:

```text
Effects(body) ⊄ DeclaredEffects(declaration)
```

For each source `flow`, infer a determinism class:

```text
Determinism(body) =
  Deterministic
  | NonDeterministic
```

Agent calls use the callee method summary. `Agentic.infer<C, O>` requested
actions, approval gates, non-deterministic subflows, runtime choice,
runtime-provided primitive calls, Etas tool calls whose bodies are
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

Checks whether agents access only allowed typed memory regions or stores. Memory access facts come from parameterized effects, as defined in [Effect System and Inference](06-effect-system-and-inference.md#11-parameterized-effects):

```text
Read(A)  = { R | Memory.read<R>  ∈ Effects(A) }
Write(A) = { R | Memory.write<R> ∈ Effects(A) }

Read(A)  ⊆ AllowedRead(A)
Write(A) ⊆ AllowedWrite(A)
```

Containment uses memory-place hierarchy. A policy over `ProjectMemory` covers `ProjectMemory.Papers`, but a policy over `ProjectMemory.Papers` does not cover `ProjectMemory.Drafts`.

### 2.5 Protocol Analysis

Checks that communication traces conform to declared protocols.

```text
TraceMessages(W) ∈ Language(Protocol)
```

### 2.6 Temporal Policy Analysis

Checks temporal properties over traces.

Examples:

```text
G(CompanyEmail.send<WorkAccount> -> previously Approval.request)
G(Command -> has SandboxProfile)
G(ClaimCreated -> eventually CitationVerified)
```

### 2.7 Cost Analysis

Computes upper bounds or estimates:

```text
MaxTokens(W)
MaxCost(W)
MaxTime(W)
MaxIterations(W)
```

For loops, require bounded iteration, explicit budget, widening in abstract interpretation, or runtime enforcement.

### 2.8 Loop Limit Analysis

Checks that loops are compatible with the effects they may perform.

```text
LoopEffects(body) ⊆ {Pure}
  allowed without explicit limit if termination is proved

LoopEffects(body) contains AgentCall
  requires Iterations(...) or another enforceable token/context/cost/time budget

LoopEffects(body) contains ToolCall
  checked through the same loop budget plus tool policy
```

The analyzer rejects non-deterministic `while` forms with neither a proven termination argument nor an enforceable `limit`.

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
| Runtime monitoring | Dynamic policy enforcement |
| Type-directed compilation | Backend-independent agent flow generation |

---
