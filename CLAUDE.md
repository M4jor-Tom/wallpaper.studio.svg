# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

`svg_builder` generates a **self-contained, CSS-animated SVG** wallpaper. Two **moods** ship: `blueprint` (a hexagon lattice with sparse triangles + a center icon) and `zine` (a black print: snowy backgrounds, a rider head on a spinning board). Any background, icon and overlay combine across moods. Everything is one Rust crate: the askama templates under `templates/` are inlined into the binary at compile time, so nothing is read from disk at runtime, and the rendered SVG itself carries no external assets — even the "space" starfield is drawn as SVG primitives, so output stays small and crisp at any resolution.

## Commands

```sh
nix run .#bgsvg                        # renders the schema's defaults
nix run .#bgsvg -- path/to/config.json
nix develop -c cargo test             # EVERY check — run after ANY change
nix develop -c cargo test --test golden -- --ignored regenerate_the_corpus   # ONLY after an intended visual change
nix build                              # default package = bgsvg; runs cargo test
```
Without Nix: Rust 1.85+ and `protoc` on `PATH`.

**`cargo test` is the whole check** — one command, two questions, and a change is unfinished until it passes. Everything lives under `tests/`; there is no second runner and no other language. Consumers such as `wallpaper.studio.ui` link this crate as a Cargo dependency (`render_to_string`, `load`, `params::{parse_res, RESOLUTIONS, DESCRIPTOR}`, and the mood catalogue `mood::{Mood, MOODS}`), so those are public API: renaming one breaks a build outside this tree. `MOODS` tags every `oneof` branch (`background.summit`, `icon.ship`, the `zine` block) with its moods, and a test walks `DESCRIPTOR` so a new branch cannot be missing from it.

*Is the render well-formed?* `tests/configs.rs` builds every config in `params::valid_configs` (145 today: every background × icon × overlay, plus each zine setting on a fixed base) and asserts the invariants below, alongside the `#[test]` functions in `src/`.

*Is it unchanged?* `tests/golden.rs` re-renders those same configs and compares bytes against `tests/golden/<sha512 of the SVG>/`, each config kept beside the `<sha512 of the SVG>_background.svg` it renders. One rule covers both files — each is named by the sha512 of its own bytes, exactly as written — so `sha512sum` reproduces every name unaided. The SVG is kept, not just its hash, so a failure reports the first differing byte instead of only a moved hash. `tests/configs.rs` and `tests/golden.rs` both enumerate from `params::valid_configs` rather than each carrying its own loop. The contract is **at most one config per distinct picture (no two configs render the same SVG), with every setting reached at least once**, not a full cross product: zine settings are sampled one at a time on a fixed base, and a setting with no visible effect is valid but never listed. So a new axis cannot reach one surface and miss the other — add an enum value and both sweeps grow. Test data is committed alone: a `test(golden): …` commit holds only `tests/golden/`, and the code commit before it must leave the golden test reporting `missing:` lines only, never `changed:` or `stale:`. It fails on any byte that moves, so `regenerate_the_corpus` is `#[ignore]`d and must be asked for by name; read the diff first — a golden change you did not intend is the regression.

## Architecture

`src/svg.rs` assembles the page; `src/blueprint/{trihex,matrix,icon,style}.rs` and `src/zine/*.rs` draw it; `src/mood.rs` is the mood catalogue; `src/params.rs` is the boundary with the outside world; `src/rng.rs` is why the picture is reproducible. `flake.nix` just wraps the crate as the `bgsvg` app. Each mood owns a folder of code (`src/<mood>/`), templates (`templates/<mood>/`) and docs. Read `docs/moods/<mood>/` (README + images + `samples/`) for the mood you are touching before touching anything visual — it is that mood's graphic contract, and the point is to extend the look without breaking it.

**One config, one render** — an optional JSON config path is the whole input
(no argument renders the schema's defaults) and `parameters.proto` is its
schema; the config path and `--configs`, which dumps `params::valid_configs` for the
corpus harness, are the only CLI surface left; consumers needing the compiled
schema read `params::DESCRIPTOR`. **Conditional rules are structural where the model
allows it, and rejected where it does not.** A motion belongs to the icon that
declares it (`Hexatri.Motion`; `Ship` declares none, and nothing assumes the
next glyph rotates), and the matrix angle and colour live inside `Matrix` — so
both are unwritable rather than rejected. `Background` is a `oneof` (`trihex`, `summit`, `treeline`, `halfpipe`), and `Trihex` keeps `motion` and
`image` as orthogonal enums, matching the parameters `pat_trihex` takes, which
leaves `CLOSEOPEN` + `NONE` expressible; `validate()` rejects it, along with
the angle range, the colour format (`zine.spot` and `park.beanie` take `#rrggbb` only, being print inks), `spin_rpm` above 30, `bpm` outside 40–200 and the resolution format.
A setting with no visible effect (a `zine` block with only blueprint items, `bpm` with every head animation off) is accepted and changes no byte: rejecting it would invalidate a saved config whenever the studio switches an item. When adding a
rule, try moving a field before adding a check.

**Determinism** — geometry depends ONLY on `seed`; the animation/icon/image/overlay choices never move a hexagon. Same seed ⇒ same layout across every combination. `pat_matrix` gets its own `PyRandom`, via `rng::cell_rng`, for exactly this reason, and `matrix.rs`'s `the_rain_lights_a_column_without_moving_anything` compares every `<polygon>` overlay-on vs overlay-off. Keep new features on this rule so seeds stay stable.

**The RNG is CPython's, deliberately** — `src/rng.rs` reimplements MT19937 with
Python's string seeding (`int.from_bytes(s + sha512(s))`) and its exact
`_randbelow`/`shuffle`/`sample` algorithms, because the golden corpus was
generated by the Python and pins every draw. Each of those consumes a specific
number of 32-bit words, so a "compatible" shortcut desynchronises everything
drawn after it. Do not swap in a Rust RNG.

**Pure CSS animation, reduced-motion-safe** — animation is `@keyframes` embedded in the SVG (no SMIL, no JS), and every animated element MUST have a resting state that `prefers-reduced-motion` falls back to (the clean static look). `css()` centralizes this.

**Build pipeline** (`svg::build_svg`): each mood supplies its own parts and `templates/root.svg` (the document shell) stacks them. A background gives a layer *under* the rain and one *over* it, an icon gives its own group attributes, and every item adds its own defs and CSS. Blueprint: `geom::Lattice::new` (the one shared hex grid) → `blueprint::trihex::pat_trihex` (triangles + optional `space` windows + optional `closeopen` blinds) → `blueprint::matrix::pat_matrix` (optional rain) → `blueprint::icon::ico_hexatri`/`ico_ship` (center glyph, drawn through `filter="url(#ink)"`) → `blueprint::style::css`, through `templates/blueprint/` (`trihex.svg` canvas + lattice, `over.svg` halo + vignette, `defs.svg` gradients/filter). Zine: a ground (`zine::summit`/`treeline`/`halfpipe`, with its `snow`) → the rider sticker (`zine::rider` wraps `punk`/`park`/`local` with the gear, paper outline and spot shadow) → `zine::css(glyph)`, through `templates/zine/`. Sizes scale with `min(w, h)` (zine: `s = min(w, h)/360`, head centre `(w/2, h/2 + 14s)`), so pattern density is resolution-independent. In an all-blueprint scene the output is byte-identical to before moods existed, with no zine def or rule in it.

**The seam between Rust and templates runs one way.** Rust computes every value that reaches a template — seeded RNG draws, geometry, animation phases, pre-formatted style-attribute strings (`geom::fmt`) — and a template only substitutes those complete values into markup. A template never computes a coordinate, never introduces a sign, digit, unit or separator adjacent to an interpolation, and never calls into `rng.rs`. This is what keeps determinism intact: nothing a template does can vary between renders of the same seed, because a template cannot vary anything on its own.

**The escaper is `askama::filters::Text` — the no-op — by deliberate choice.** `askama.toml` registers it for the `.svg` extension. Disabling escaping is normally a defect; it is safe here because every value that reaches a template is either computed internally or validated at the `params` boundary: colours pass `style::hex_rgba` (rejects anything but `#rrggbb`/`#rrggbbaa`), angle/seed/resolution are numbers checked by `params::validate()`, `MATRIX_GLYPHS` deliberately excludes `< > & " '`, and the aria label is assembled from fixed literals — so this crate never emits an escapable character anywhere, by design. That is a standing obligation, not a one-time audit: anyone adding a config field that reaches a template must route it through `params::validate()` first, or the `Text` escaper stops being safe.

**Zine rules that bite** (the contract is `docs/moods/zine/README.md`):
- **No SVG `filter` anywhere in zine output.** The sticker outline is geometry (a paper stroke under each shape, a translated spot copy beneath). The rainbow and lens flow use *colour keyframes*, not `hue-rotate`: the filter was measured and cost frames.
- **Reduced motion rests on bare markup.** With every animation removed the picture is the clean print: board horizontal, head centred, sunburst and snow where they start, and elements that exist only in motion (glint, breath) invisible. `zine::css` ends with the reduced-motion rule.
- **Per-feature CSS.** A zine rule and its keyframes ship only when its feature is on in the scene, and every class and keyframe starts with `z`. That is why adding a feature never moves an earlier golden.
- **Own RNG streams.** Zine geometry draws from `"summit:{seed}"`, `"treeline:{seed}"` or `"snow:{seed}"`, never `"trihex:{seed}"`, so an icon, a setting or an overlay never moves a mountain.

**`ship` is a solid, not linework.** The hull is tiled by four translucent facets whose *fill steps* carry the relief; the ship's own gradients live in the glyph, not in `build_svg`'s `defs`, because they have exactly one consumer. `icon.rs`'s `the_ship_is_a_folded_solid` checks the facets still tile the hull *by area*, so the cloak cannot move the silhouette, and it asserts against `ico_ship()` alone — matched against a whole page, the lattice supplies polygons that satisfy those patterns by chance. Why the facets are lit and valued the way they are is argued in `ico_ship`'s docstring; read it there.

**`matrix` rain — nothing moves.** The characters are anchored by `x`/`y` and never translate; the *lighting* travels. Every cell of a column holds one glyph fixed at generation, and all of them run the same `fill-opacity` keyframes offset by one cell-time, so the head advances into a fresh character while those behind dim in place. Animating a transform instead would slide the characters across the canvas — the one thing this effect must not do, and what `matrix.rs`'s `the_rain_lights_a_column_without_moving_anything` guards. The layer sits between the lattice and the halo circle, so the halo *subtracts* it around the icon exactly as it subtracts the lattice; there is no icon-exclusion code and there should not be. The remaining design (one rotated group, upright counter-rotated glyphs, `--o`/`--t`/`--d` inheritance, the t=0 resting attribute) is argued in `pat_matrix`'s docstring — read it there rather than restating it here.

**Triangle invariant** — every hexagon is either a *holder* (owns one edge as a triangle base) or an *intersector* (a triangle tip pokes in), never both, pierced at most once. This is what keeps triangles few, non-overlapping, and out of the icon zone. `trihex.rs`'s `roles_are_exclusive_and_the_centre_is_clear` checks it.

**`closeopen` occlusion trick** — SVG does no occlusion culling, so each window `display:none`s itself for exactly the span its opaque blind covers it (measured large frame-rate/memory wins). The window's off-span and its blind's shut-span are both derived from ONE constant (`blind_phase`); do not split them or a window will desync from its blind.
