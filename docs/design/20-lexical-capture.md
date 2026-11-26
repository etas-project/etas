# Lambda And Handler Capture

## 1. Scope

Lambdas and Handlers share lexical capture, lifetime and task-affinity rules.

## 2. Binding Identity And Assignment

Creation records definition-site bindings without executing the body.

| Captured binding | Reads | Assignment |
| --- | --- | --- |
| `let` or an ordinary immutable parameter | Read the bound value | Reassignment is rejected |
| Local `var` | Read the current value of the original binding | Rebind that original binding in the same task |
| Restricted `var self` receiver | Subject to the existing updating-receiver access rules | Update access follows its existing scope and escape restrictions |

Even read-only capture retains a `var`'s mutable binding identity. Completed
assignment is immediately observable in ordinary evaluation order.

```etas
var n = 0;
let snapshot = n;
let current = () => n;
let saved = () => snapshot;

n = 7;
current(); // 7
saved();   // 0
```

Same-activation captures share bindings; shadowing and separate invocations
create distinct bindings. Parameters, action payloads and locals belong to each
activation; captured outer bindings retain their identity across calls.

## 3. Value Semantics And Callable Copies

Captured-variable assignment preserves ordinary data value semantics:

```etas
var xs = [1];
let original = xs;
let append = () => { xs = xs.push(2); };

append();
// xs is [1, 2].
// original is [1].
```

Copies and containers preserve capture identities, not independent environments;
container updates still have value semantics. Use an immutable snapshot for an
independent captured value. Opaque resources retain identity/ownership.
Handler `+`/`~>` preserves all constituent captures and their restrictions;
composition creates no snapshot, task, new ownership or authority.

## 4. Captured Mutable Bindings And Lifetime

Capturing local `var`, even read-only, binds a callable and its copies/containers
to that scope. Reject return, longer-lived storage or cross-task transfer;
helper calls need checked non-escape. Immutable-only captures retain ordinary
value/return behavior, subject to captured resource lifetimes.

```etas
flow example() -> i32 ![] {
    var n = 0;
    let next = () => {
        n = n + 1;
        return n;
    };
    let same = next;
    next(); // 1
    return same(); // 2
}

flow invalid_counter() -> (() -> i32 ![]) ![] {
    var n = 0;
    return () => n; // rejected: captured var would leave its owning scope
}
```

Resource, receiver and continuation ownership/escape checks still apply.
Snapshots require copyable values; mutable module globals remain prohibited.

## 5. Handler Completion And Resumption

Inline and first-class Handler arms use the same capture rules:

```etas
effect E { action tick() -> unit; }

flow example() -> i32 {
    var n = 0;
    handle {
        perform E.tick();
    } with {
        E.tick() => {
            n = n + 1;
            resume ();
        }
    };
    return n;
}
```

`example()` returns 1; resumed code and other same-task captures observe n's
update. Resume, finish and errors do not roll back completed local assignments;
pending receiver updates follow their separate commit/rollback contract.

## 6. Task Boundaries And Analysis

| Boundary | Rule |
|---|---|
| Logical task | Mutable captures stay task-local across suspension, thread migration and containers/callables. Agent calls and synchronous `run`/`Scope.enter` stay in the caller task; scheduler owner jobs add no capture boundary. |
| Child tasks | Check affinity for `start`, concurrent branches, inherited Handlers and task results. No captured-var sharing; use snapshots or checked shared resources. |
| Agents | Reference copies retain aliases/lifetimes/access. Children cannot share `var self`/pending state; same-task handlers cannot bypass exclusivity. Capture fields first; member tools receive dispatch snapshots, not receiver aliases. |
| Analysis | Higher-order calls, imports, containers and composition preserve read/write/affinity/environment constraints. Abstract provenance requires admission checks; type changes erase none. |
| Optimization | Local rebinding adds no Memory action, but retains state dependence. Empty escaping rows do not justify memoization, call elimination or parallelization without capture/state proofs. |

## 7. Suspension And Restoration

Suspension/restoration preserves binding identity, values and task ownership.
Restored copies share bindings; separate restorations are independent. Storage
layout/encoding is not a language contract.

## 8. Conformance Cases

| Case | Required Result |
|---|---|
| Reader versus snapshot | A captured `var` observes later rebinding; a captured immutable snapshot does not. |
| Copies and activations | Callable copies share the same captured binding; separate declaring activations do not. |
| Handler completion | Resumed code observes completed assignments; `finish` does not roll them back. |
| Scope/task boundary | Reject captured-var escape or cross-task sharing; permit an independently captured immutable value when its type allows it. |
| Imports and checkpoint | Preserve binding ownership, aliases and capture summaries across separate compilation and restoration. |
