# 0017. `background` becomes a `oneof`; motion and image move under `trihex`

| Field    | Value                         |
|----------|-------------------------------|
| Date     | 2026-11-14                    |
| Status   | Accepted                      |
| Deciders | theta                         |
| Branch   | `master`                      |
| Commit   | `2eb1e85` (schema), `b6004ca` (corpus re-keyed) |

## Context

`Background` was `{ motion, image }`, the two parameters `pat_trihex` takes.
Zine adds three grounds (`summit`, `treeline`, `halfpipe`) that have neither,
and one setting of their own, `snow`. The rule in CLAUDE.md is: when adding a
rule, try moving a field before adding a check.

## Decision

`Background` is a `oneof ground { Trihex trihex; Summit summit; Treeline treeline; Halfpipe halfpipe; }`.
Today's `motion` and `image` move into `Trihex`, still two orthogonal enums.
Each zine ground owns its `snow`. This breaks existing configs:
`{"background":{"motion":"SCAN"}}` becomes
`{"background":{"trihex":{"motion":"SCAN"}}}`. An unset background is trihex
with its defaults, so `{}` still renders the same picture.

The corpus was re-keyed in a test-data commit of its own: the set of directory
names under `tests/golden/` (the SVG hashes) is identical before and after, and
only each `_parameters.json` changed shape.

## Alternatives Considered

### Keep the flat fields and reject them on zine backgrounds

Non-breaking, but it is the pattern the project steers away from: a motion on a
zine ground would be expressible and rejected in `validate()`, where moving the
field makes it unwritable. Rejected. The user accepted the break.

### Put `snow` on `Background` itself

One field instead of three. It would need a rule rejecting `snow` on `trihex`,
the same pattern again. Per-ground ownership leaves it unwritable there.

## Consequences

### Positive

- A motion on a zine ground, or a snow setting on the lattice, cannot be written.
- `MOODS` keys line up with the oneof branches (`background.trihex`,
  `background.summit`).

### Negative

- A breaking schema change: any saved config using `background.motion` or
  `background.image` must be rewritten. The studio pins this crate through
  `Cargo.lock` and is unaffected until it runs `cargo update -p bgsvg`.
- Three near-identical `{ Snow snow }` messages.

### Neutral

- `CLOSEOPEN` with `NONE` is still the one cross-field rule `validate()` carries.

## Resumption (for Agent)

**Current state.** Done.

**Key files.** `parameters.proto`, `src/params.rs` (`resolve`, `validate`,
`valid_configs`), `tests/golden/`.

**Next steps.** The studio adopts the new paths with its mood filter (ROADMAP).

**How to verify.** `nix develop -c cargo test`. To re-check the re-key:
`git show --stat b6004ca` lists only `tests/golden/`, and the directory names
match those of its parent.

**Gotchas.** A config in the old flat shape fails with `unknown field`, by
design. A new zine ground needs a `MOODS` row.

**Related.** [[0001]], [[0009]], [[0016]].
