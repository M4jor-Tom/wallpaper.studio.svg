# 0015. Drop the WASM target; consumers link the crate

| Field    | Value                          |
|----------|--------------------------------|
| Date     | 2026-11-13                     |
| Status   | Accepted; supersedes [[0011]] and [[0012]] |
| Deciders | theta                          |
| Branch   | `master`                       |
| Commit   | pending                        |

## Context

[[0011]] built `bgsvg` for `wasm32-unknown-unknown` so a static editor page
could render in the browser, and [[0012]] kept that binding here as the
`bgsvg-wasm` workspace member, with `tests/wasm.mjs` sweeping the corpus
through it.

That editor, `wallpaper.studio.ui`, is now a Rust server. It depends on this
repository as a Cargo git dependency and calls `render_to_string`, `load`,
`params::parse_res`, `params::RESOLUTIONS` and `params::DESCRIPTOR` natively.
Nothing loads the wasm module any more. What remained was a crate, a Nix
derivation, three devShell tools (`lld`, `wasm-bindgen-cli`, `nodejs`) and the
only test in the tree that skips: without `BGSVG_WASM` set it passed without
checking anything, including in `nix build`.

## Decision

We will delete `crates/bgsvg-wasm`, `tests/wasm.rs`, `tests/wasm.mjs` and
`packages.bgsvg-wasm`, and stop being a workspace. `bgsvg` is one crate again,
and its public Rust API is the integration surface for consumers.

`render_to_string` stays. [[0011]] introduced it for the binding; the UI now
calls it in the binding's place, and it holds the zero-dimension guard that the
CLI gets from `parse_res` instead.

## Alternatives Considered

### Keep `bgsvg-wasm` in case a static editor comes back

Rejected. It would have no consumer and a test that skips by default, so it
would rot without anything noticing. Git history keeps it; reverting this ADR's
commit restores it in one step.

### Move the binding into `wallpaper.studio.ui`

Rejected: the UI has no use for it. It renders server-side by calling the crate,
which is exactly what a binding would have wrapped.

## Consequences

### Positive

- `cargo test` is the whole check with nothing skipped: a green run has checked
  everything, which [[0014]] could only promise with `BGSVG_WASM` set.
- No non-Rust code in the tree. The devShell drops `lld`, `wasm-bindgen-cli` and
  `nodejs`; `Cargo.lock` drops `wasm-bindgen`, `js-sys` and
  `console_error_panic_hook` with their dependencies.

### Negative / Trade-offs

- The property [[0011]] chose WASM for — an editor that runs as a static page,
  offline, with no process — is gone. That is the UI's decision; this records
  that this repository no longer offers it.
- The UI's builds break on a renamed public item rather than on a changed JS
  export, and nothing in this tree checks for it. `CLAUDE.md` names the items it uses.

### Neutral

- `bgsvg --descriptor` was added alongside the binding ([[0012]]) for the old
  TypeScript UI's drift check. The Rust UI reads `params::DESCRIPTOR` instead,
  but the flag is CLI surface for other languages and is kept.
- The UI pins a revision, so nothing changes for it until it updates (Next steps).

## Resumption (for Agent)

### Current state

Complete. One crate, no workspace; the golden corpus is unchanged.

### Key files / entry points

| File | Role |
|------|------|
| `Cargo.toml` | no `[workspace]` block |
| `flake.nix` | one package, `bgsvg`; the devShell carries no wasm or Node tooling |
| `src/lib.rs`, `src/params.rs` | the public items the UI links; `CLAUDE.md` lists them |

### Next steps

None here. In `wallpaper.studio.ui`: `cargo update -p bgsvg`, then refresh
`cargoLock.outputHashes."bgsvg-0.1.0"` in its `flake.nix` (`nix build` prints
the wanted value).

### How to verify

```bash
nix develop -c cargo test        # every check; nothing prints SKIPPED
nix build                        # runs cargo test in the sandbox
nix flake show                   # packages: bgsvg, default
rg -n 'wasm-bindgen|js-sys' Cargo.lock   # no hits
```

### Gotchas

- The docs under `docs/superpowers/` and ADRs [[0011]]–[[0014]] still describe
  the wasm build. They are history; read them as such.

### Related

- ADRs: [[0011]] superseded (the decision to render via WASM) · [[0012]]
  superseded (where the binding lived) · [[0014]] its wasm sweep removed
