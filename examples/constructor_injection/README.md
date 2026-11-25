# Construction injection across packages

The library declares record and Agent fields with bare `@inject` and
`@inject(Action)`. The consumer supplies dependencies with type arms and
ordinary action arms. Explicit fields run first; omitted fields
request their providers in declaration order. Explicitly supplied fields do not
request a provider, and reading stored fields does not repeat injection.

With a matching installed Etas SDK, run from this directory:

```sh
etas --no-config --cache off pkg metadata library
etas --no-config --cache off pkg lock records
etas --no-config --cache off run -O 0 records
etas --no-config --cache off run -O 3 records
etas --no-config --cache off pkg lock app
etas --no-config --cache off run -O 0 app
etas --no-config --cache off run -O 3 app
```

Each program returns `0` when its assertions hold. `records` uses the compiled
record construction contracts, including generic providers and a transparent
alias. `app` also constructs an injected Counter and calls its methods. Its
pipeline composes imported empty Agent and record types with public `run`
methods, including a generic Agent method and a transparent type alias. Each
invocation constructs its own instance.
Generated package metadata, lockfiles, and package stores are local build output.
