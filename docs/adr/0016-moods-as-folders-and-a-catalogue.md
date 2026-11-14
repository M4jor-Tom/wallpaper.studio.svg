# 0016. Moods are folders of code, plus a public catalogue

| Field    | Value                                              |
|----------|----------------------------------------------------|
| Date     | 2026-11-14                                         |
| Status   | Accepted                                           |
| Deciders | theta                                              |
| Branch   | `master`                                           |
| Commit   | `b294623` (move), `57919f1` (catalogue), `35fe0a4` (page assembly) |

## Context

The crate had one look, the blueprint, and every rule in its mood document
(no black, no warmth, two desaturated hues, slow ambient motion) was written as
if it governed every possible render. A second, deliberately opposite look, the
black `zine` print, was wanted. Two further needs came with it:

- The studio (`wallpaper.studio.ui`) should later filter items by mood, so the
  mood of each config choice must be machine-readable.
- Mixing moods in one scene (a zine head on the lattice, the matrix rain over
  a mountain range) must stay valid.

The 42 existing renders are pinned byte for byte ([[0002]], [[0009]]), so
whatever carries the idea of a mood cannot move one of them.

## Decision

A mood owns a folder of code, one of templates and one of docs:
`src/<mood>/`, `templates/<mood>/`, `docs/moods/<mood>/`. Blueprint's code moved
there unchanged; zine was written there. Mood-neutral helpers (`hex_rgba`,
`mix`, `geom::fmt`, the RNG, the params boundary, page assembly) stay shared.

`mood::MOODS` is the public catalogue: a slice of
`(dotted config path, &[Mood])`, keyed by every `oneof` branch
(`background.summit`, `icon.ship`, ...) and by mood-wide blocks (`zine`). An item may
carry several moods, and settings inside a branch inherit the branch's moods.
A test walks `params::DESCRIPTOR` and fails if any branch under `Background`,
`Icon` or `Overlay`, or any top-level mood block, is absent from `MOODS`.

Page assembly (`svg::build_svg`) is per mood: a background provides a layer
under the rain and one over it, an icon provides its own group attributes, and
each item adds its own defs and CSS. Zine classes and keyframes start with `z`
so a cross-mood scene cannot collide with blueprint's unprefixed ones.

## Alternatives Considered

### A catalogue only, files left flat

Same `Mood` type and table, no folders. A smaller diff, but moods would exist
only as data: the code would not read by mood, each mood's docs would not sit
beside its code, and the next mood would still need the restructure.

### Moods as custom options in `parameters.proto`

One source of truth, next to the schema. Rejected because prost discards custom
options when it decodes a descriptor, so this crate and the studio would each
need a hand-written descriptor reader to recover them.

### Matching names in a string for the coverage test

`tests/descriptor.rs` does this for other checks. Not enough here: a new branch
would slip through. Walking the descriptor (`prost-types` becomes a
dev-dependency; it is already compiled for the build script) catches it.

## Consequences

### Positive

- The code reads by mood, every existing render stayed byte-identical, and the
  next mood is a new folder plus catalogue rows.
- The studio can filter on a public, tested table.

### Negative

- The largest diff of the work: every blueprint file moved (`git mv`, so history follows).
- `MOODS` is a hand-kept table. The coverage test makes forgetting a row fail,
  but writing the right moods is still a human call.

### Neutral

- Module paths of the drawing code (`bgsvg::trihex` and so on) moved. No
  consumer used them. `render_to_string`, `load` and
  `params::{parse_res, RESOLUTIONS, DESCRIPTOR}` keep their paths.

## Resumption (for Agent)

**Current state.** Done. Two moods, 145 corpus configs, goldens unchanged for
the original 42.

**Key files.** `src/mood.rs`, `src/svg.rs`, `src/blueprint/`, `src/zine/`,
`templates/{root.svg,blueprint/,zine/}`, `docs/moods/*/README.md`,
`tests/moods.rs`.

**Next steps.** A new mood: add `src/<mood>/`, `templates/<mood>/`,
`docs/moods/<mood>/`, a `Mood` variant and `MOODS` rows. The mood filter in
the studio is on the ROADMAP.

**How to verify.** `nix develop -c cargo test`; `tests/moods.rs` fails on an
uncatalogued branch.

**Gotchas.**
- Zine CSS and defs must be absent from an all-blueprint render, or the golden
  test fails.
- `tests/purity.rs` walks `src/` recursively; a new subfolder is covered.
- Blueprint icons need `#ink`, so a blueprint icon on a zine ground still
  emits blueprint's defs.

**Related.** [[0002]], [[0009]], [[0017]]; spec
`docs/superpowers/specs/2026-11-13-moods-and-zine-design.md`.
