# Working With Sibling Checkouts

## Published Dependencies And Local Integration

The committed workspace manifest and lockfile select exact Git revisions of
Core, Frontend, and Interpreter. Those revisions form the published dependency
set. Updating sibling repositories to their latest `main` does not, by itself,
change that dependency set.

For local integration, Cargo patches can select the sibling checkouts instead:

```text
workspace/
  etas/
  etas-core/
  etas-frontend/
  etas-interpreter/
```

For example, an `etas/.cargo/config.toml` can include:

```toml
[patch."https://github.com/etas-project/etas-core.git"]
etas_host = { path = "../etas-core/crates/etas_host" }
```

This example overrides one crate only. A complete cross-repository integration
must override all participating shared crates consistently; otherwise Cargo can
build both Git and path copies of the same crate, producing incompatible type
identities. The Frontend and Interpreter workspaces need their own patches when
tested independently. Keep machine-local patch configuration out of published
manifests and commits.

Paths in the conventional `.cargo/config.toml` above are relative to the Cargo
workspace root. Run commands from the repository being tested so Cargo discovers
its local configuration.

## Why A Lockfile Can Change Without Source Changes

A path patch changes a package's source identity. Cargo therefore removes the
corresponding Git `source` entry from the resolved lockfile. Local IDE metadata
refreshes can also trigger Cargo resolution. This is not evidence that an old
feature was merged or that the Rust source changed.

Do not publish path-patched lockfiles as the released Git dependency set. Keep
their changes separate from source fixes and preserve any independent lockfile
changes before cleanup. To validate the published pins, use a checkout without
local patch configuration and its committed lockfile. Use `--locked` there.
Do not conceal local differences with `assume-unchanged` or `skip-worktree`.

## Verification

Run each command from its named repository. Cross-repository integration uses
the local patches; published-pin validation uses the unpatched dependency set.

```sh
# etas-core
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# etas-frontend
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# etas-interpreter
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# etas
cargo build -p etas_cli
target/debug/etas --no-config --cache off std install --prefix target
cargo test --workspace
cargo check -p etas_cli --all-features
cargo fmt --all --check
```

Add `--offline` only when the dependencies are already cached. Host network
tests need permission to bind loopback listeners; a sandbox denial is not a
reason to remove assertions or ignore the tests. Tests requiring a live model
provider or Linux-specific isolation remain separate acceptance runs. An
ignored live test is not evidence of successful provider integration.

The CLI's in-process integration tests look up the matching source SDK under
the workspace's `target/lib/etas/` prefix. Install it with the freshly built
debug binary before running those tests; do not replace the provider with
legacy declarations to conceal a missing SDK. Reinstall when code or dependency
changes alter the toolchain build identity. This test SDK is a build artifact,
not a user-wide installation or source file to commit.

For the source standard-library SDK installation workflow, follow the current
[README](../../README.md#quickstart) and
[package specification](../design/15-package-management.md#71-using-the-installed-standard-library).
An old CLI installation command without a matching SDK is not an equivalent
replacement.

## Recovering Older Local Work

Compare old local changes against both their original base and current `main`.
A dirty-file count is not an unmerged-feature count. Even a textually clean
three-way merge can duplicate a type or hook introduced elsewhere upstream.

Keep replaced implementation paths deleted. Port only independently reviewed
changes and current-contract regression tests. Historical architecture worklists
belong in an explicitly marked archive, not in the active implementation queue.

During the 2026-11-21 integration, the three previously automatic local patch
configurations were backed up and removed from automatic Cargo discovery.
This prevents editor metadata refreshes from silently replacing the published
Git dependency identities and dirtying committed lockfiles. Explicit local
integration profiles with absolute paths remain in the private recovery backup.
They can be selected for a deliberate integration run with
`cargo --config /absolute/path/to/profile.toml test`; that still changes the
workspace lockfile, so preserve any independent lock edits first. Normal builds
and final synchronization checks use the committed pins without a patch profile.
