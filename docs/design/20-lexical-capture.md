# Lambda And Handler Capture

## 1. Scope

This chapter specifies the shared lexical capture, lifetime and task-affinity
rules for anonymous flows (lambdas) and Handler values.

## 2. Binding Identity And Assignment

A lambda or Handler resolves free variables in its definition's lexical scope.
Creating the value records those bindings without executing its body.

| Captured binding | Reads | Assignment |
| --- | --- | --- |
| `let` or an ordinary immutable parameter | Read the bound value | Reassignment is rejected |
| Local `var` | Read the current value of the original binding | Rebind that original binding in the same task |
| Restricted `var self` receiver | Subject to the existing updating-receiver access rules | Update access follows its existing scope and escape restrictions |

A captured `var` keeps its binding identity even when a particular lambda only
reads it. Mutability belongs to that declaration and does not depend on which
expression happens to access it. Assignment is visible immediately after the
assignment completes, following ordinary evaluation order.

```etas
var n = 0;
let snapshot = n;
let current = () => n;
let saved = () => snapshot;

n = 7;
current(); // 7
saved();   // 0
```

Separate lambdas and Handlers created in the same activation and capturing
the same `var` observe the same binding. Lexical shadowing introduces a new
binding. Separate invocations of the declaring flow also create distinct
bindings, even though their declarations have the same source symbol.

Invocation parameters, handler payload parameters and body-local declarations
have their own activation-local bindings. A captured outer variable retains
its identity across those invocations.

## 3. Value Semantics And Callable Copies

The ordinary value rules for numbers, records and collections continue to
apply when assigning to a captured variable.

```etas
var xs = [1];
let original = xs;
let append = () => { xs = xs.push(2); };

append();
// xs is [1, 2].
// original is [1].
```

Copying a lambda or Handler value preserves the identities of its captured
bindings. Storing that callable in a tuple, record or collection preserves
the same relationship. Updates to those value containers still follow their
ordinary value semantics.

To give a callable an independent captured value, introduce an immutable
snapshot binding before constructing it, as with `snapshot` in section 2.
Opaque resource values retain their declared resource identity and ownership
contracts when copied or captured.

## 4. Captured Mutable Bindings And Lifetime

A lambda or Handler that captures an outer local `var` is bound to that
binding's current lexical scope. It may read and rebind the variable within
that scope, but cannot escape it. This applies even when the callable only
reads the captured mutable binding. Copies and enclosing value containers
retain the same restriction.

Returning such a callable, storing it in a longer-lived binding, or transferring
it to another task is rejected. Passing it to another callable requires a
checked promise that the argument remains within the owning scope. Lambdas
and Handlers that capture only immutable values retain their ordinary value
and return behavior.

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

Scope-bound resources, updating-receiver access and single-shot continuation
handles retain their existing ownership and escape checks. An independent
immutable snapshot can be captured where its value type permits it. Mutable
module-global state remains prohibited.

## 5. Handler Completion And Resumption

Inline Handler arms and separately constructed Handler values use the same
capture rules as lambdas.

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

`example()` returns `1`. Code resumed after `perform` also observes the updated
binding. A lambda and a Handler capturing the same `n` within this task see
each other's assignments.

Assignments completed before `resume`, `finish`, or error propagation remain
observable while the binding remains live. These control forms do not roll
back ordinary local assignment. The restricted updating receiver's separate
commit/rollback contract applies to its pending receiver value.

## 6. Task Boundaries And Analysis

Captured mutable local bindings belong to their owning logical task. A lambda
or Handler retaining such a binding is task-local, including through nested
callables or value containers. The logical task can suspend, resume or migrate
between host threads while retaining the same bindings.

Synchronous `run` and its protected `Scope.enter` execution remain in the
caller's logical task. A scheduler's internal owner job does not introduce a
new capture boundary; independently started tasks and concurrent branches do.

Crossing to another task by `start`, concurrent branch execution, inherited
Handlers or task results requires checking this capture contract. A callable
that retains a local `var` cannot share that binding with the other task.
Capture an immutable snapshot for independent task input, or use an explicit
shared resource with its checked Effect and ownership contract.

The compiler carries captured-binding read/write and task-affinity facts
through higher-order calls, imports, callable containers and Handler metadata.
Where static provenance is abstract, task admission checks the retained
environment before permitting execution in another task. Changing a callable's
ordinary function type does not discard its environment constraints.

Local binding assignment adds no external Memory authority action by itself.
The compiler still records its captured-state behavior. An empty escaping
Effect row alone does not justify memoization, repeated-call elimination or
automatic parallel execution for such a callable. Reads of a captured `var`
can observe previous assignments, and writes change subsequent observations.
Optimization requires the corresponding capture and state-dependence proof.

## 7. Runtime And Checkpoint Representation

The runtime preserves one storage location for each captured mutable binding
in an activation, using stable shared slots or binding cells. Invocation-local
storage remains separate. Source symbol identity is combined with activation
identity so separate calls to `example` retain independent state.

Assignments through any live alias update that location. Copying a callable,
capturing it in another callable, suspending it or evaluating a Handler must
preserve this alias relationship. Ordinary value copies retain their value
semantics.

Checkpoint encoding records the binding graph, including callable references,
current values and logical-task ownership. Restoration reconstructs the same
alias relationships within the restored execution. Separate restorations of
one checkpoint create independent executions. Recompilation and source
restoration relocate binding identities using the existing checked-code
mapping mechanisms.

## 8. Conformance Cases

| Case | Required Result |
|---|---|
| Reader versus snapshot | A captured `var` observes later rebinding; a captured immutable snapshot does not. |
| Copies and activations | Callable copies share the same captured binding; separate declaring activations do not. |
| Handler completion | Resumed code observes completed assignments; `finish` does not roll them back. |
| Scope/task boundary | Reject captured-var escape or cross-task sharing; permit an independently captured immutable value when its type allows it. |
| Imports and checkpoint | Preserve binding ownership, aliases and capture summaries across separate compilation and restoration. |
