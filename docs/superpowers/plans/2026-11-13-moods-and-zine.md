# Moods and the Zine Mood Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make moods a concept in the code (a folder per mood plus a public
catalogue), move today's look under `blueprint` byte for byte, and add the
`zine` mood: three backgrounds and three rider heads with their settings.

**Architecture:**
- **Shared core:** `params` (the boundary), `svg` (page assembly), `geom`,
  `rng` and `color` stay shared.
- **Mood folders:** drawing code lives in `src/blueprint/` and `src/zine/`,
  each with its own templates.
- **Page assembly:** the page is built from per-mood parts (defs, CSS, an
  under-rain and an over-rain layer, icon attributes). An all-blueprint scene
  must concatenate to exactly today's bytes.
- **The catalogue:** `mood::MOODS` tags every config choice, and a test keeps
  it complete against `params::DESCRIPTOR`.

**Tech Stack:** Rust 2024, askama 0.16 templates, prost 0.14 + pbjson 0.9
(proto3 → JSON), `prost-types` 0.14 (new dev-dependency), headless chromium
or `playwright-cli` for the browser checks.

**Spec:** `docs/superpowers/specs/2026-11-13-moods-and-zine-design.md`.
Reference renders: `docs/superpowers/specs/2026-11-13-zine-mood-reference/`.
Read both before any zine task. Where the spec's prose and a reference file
disagree on a number, the reference wins; the prose wins on mechanics.

## Global Constraints

- **The whole check** is `nix develop -c cargo test`. A task is unfinished
  until it passes. There is one exception: the Task 2 commit, which fails by
  design. Also run `nix develop -c cargo clippy --all-targets -- -D warnings`
  and `nix develop -c cargo fmt --check`.
- **The 42 existing golden SVGs never change a byte.** Only Task 3 touches
  their config files.
- **Test-data commits stand alone.** Every corpus regeneration is its own
  commit (`test(golden): …`) containing only `tests/golden/`. A zine task's
  code commit must leave the golden test reporting only `missing:` lines,
  never `changed:` or `stale:`.
- **Determinism:**
  - Zine geometry draws only from `PyRandom::new("summit:{seed}")`,
    `"treeline:{seed}"` or `"snow:{seed}"`.
  - It never draws from `"trihex:{seed}"`.
  - No other RNG is allowed.
- **Templates only substitute:**
  - Rust computes every value.
  - `tests/purity.rs` forbids `"<` in production Rust.
- **Every new config string or number that reaches a template** goes through
  `params::validate()` first. That covers `zine.spot`, `park.beanie`,
  `spin_rpm` and `bpm`.
- **No SVG `filter` in any zine output.** The sticker outline is geometry.
- **Zine defs and CSS** appear only when a zine item is in the scene, and
  every zine CSS class and keyframe name starts with `z`.
- **Reduced motion:** with every animation removed, an element shows its
  resting state.
  - The board rests horizontal and the head rests centred.
  - Glint and breath rest invisible.
  - The sunburst and the snow rest where they start.
- **Palette (verbatim):**

  | role | value |
  |---|---|
  | sheet | `#111114` |
  | paper | `#f3efe6` |
  | ink | `#16161a` |
  | head | `#0d0d12` |
  | rays | `#1c1c21` |
  | default spot | `#e63946` |
  | rainbow | `#ff3b3b #ff8a1f #ffd400 #2ed573 #1e90ff #5b4bff #c04bff` |

- **Scale and anchoring:**
  - `s = min(w,h)/360`.
  - The head centre is `(w/2, h/2 + 14·s)`.
  - A mockup point `(x, y)` that is anchored to the head maps to
    `(w/2 + (x−320)·s, h/2 + 14·s + (y−194)·s)`.
- **Commits:**
  - Conventional Commits, on branch `feature/zine-mood`.
  - Every message ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- **Browser checks:** every task with visual output finishes by rendering its
  sample and looking at it next to the matching reference SVG. Use
  `playwright-cli`, or `chromium --headless --screenshot`.

## Review Focus

These are the inputs and conditions no feature test naturally covers, ranked
by how likely they are to bite. Each line names the task whose tests pin it.

1. **Extreme and portrait frames.** Inputs such as `mobile`, `square`,
   `ultrawide`, `1x1` and `8000x200` must render without panicking and stay
   bounded in size. Flanking peaks and halfpipe lips must stay in frame on
   `mobile`. → Tasks 6, 7, 8.
2. **Colour input in any accepted spelling.** `" #22C8E5 "` must reach the SVG
   as `#22c8e5`, and the raw input must never appear. → Tasks 6 and 11.
3. **Alpha colours** for `zine.spot` or `park.beanie`, such as `#e6394680`,
   are rejected with a message naming the field. → Tasks 6 and 11.
4. **Settings with no visible effect are accepted and change nothing.**
   Examples: a `zine` block with blueprint-only items, `counter_clockwise`
   with `spin_rpm: 0`, and `bpm` with every head animation off. Each renders
   identically to the same config without it. → Tasks 6, 9, 10.
5. **Reduced motion with every zine animation switched on** still shows the
   resting picture. → Tasks 10 and 11.

---

### Task 1: Move today's look under `blueprint` (byte-identical)

**Files:**
- Create: `src/blueprint/mod.rs`, `src/color.rs`
- Move (`git mv`, contents unchanged except `use` paths):
  - `src/{trihex,matrix,icon,style}.rs` → `src/blueprint/`
  - `templates/{defs,hexatri,matrix,ship,trihex}.svg` → `templates/blueprint/`
  - `docs/mood/` → `docs/moods/blueprint/`
- Modify:
  - `src/lib.rs`, `src/svg.rs`, `src/params.rs`
  - every `#[template(path = …)]` in the moved files
  - `include_str!` paths in `src/blueprint/style.rs`
  - `tests/purity.rs`
  - `docs/moods/blueprint/samples/*.json` (output paths)
  - `docs/moods/blueprint/README.md` (the regenerate command)
  - `CLAUDE.md`, `README.md` (links to `docs/mood/`)
- `templates/root.svg` stays in place: it is the shared page shell.

**Interfaces:**
- Produces:
  - `bgsvg::color::{mix(h1: &str, h2: &str, t: f64) -> String, hex_rgba(s: &str) -> Result<(String, f64), String>}`.
    Both move out of `style.rs`, and `mix` becomes `pub`.
  - `bgsvg::blueprint::{trihex, matrix, icon, style}`.
  - `darken`, `PAL`, `css()` and every blueprint constant stay in
    `blueprint::style`.
- Root modules after this task: `blueprint, color, geom, params, rng, svg`.

- [ ] **Step 1: Make the purity test recursive and prove it reaches subfolders.**
  Change `no_svg_markup_in_rust_sources` to walk `src/` recursively. Add a
  test, `purity_walks_subfolders`, that asserts the walk visits
  `src/blueprint/trihex.rs`.
- [ ] **Step 2: Run** `nix develop -c cargo test --test purity`.
  Expected: FAIL, because `src/blueprint/trihex.rs` does not exist yet.
- [ ] **Step 3: Move the files.**
  - Do the `git mv`s listed above.
  - Create `src/blueprint/mod.rs` with the four `pub mod`s.
  - Create `src/color.rs` with `mix` and `hex_rgba`, and move the `mix`
    ties-to-even test and the `hex_rgba` test with them.
  - Fix the `use` paths.
  - Update the four doc references to the new `docs/moods/blueprint/` path.
    ADRs are historical records: leave them.
- [ ] **Step 4: Run** `nix develop -c cargo test`.
  Expected: everything passes, including `the_corpus_is_unchanged`, with no
  regeneration.
- [ ] **Step 5: Check the samples are byte-identical.**
  ```bash
  nix develop -c cargo build --release
  for f in docs/moods/blueprint/samples/*.json; do target/release/bgsvg "$f"; done
  git diff --exit-code docs/moods/blueprint/samples/*.svg
  ```
  Expected: exit 0.
- [ ] **Step 6: Commit** `refactor: move today's look under src/blueprint and docs/moods/blueprint`.

### Task 2: `background` becomes a choice, `trihex` its first branch (commit 1 of 2)

**Files:**
- Modify:
  - `parameters.proto`
  - `src/params.rs` (`resolve`, `validate`, `valid_configs`, `Scene::slug`, tests)
  - `src/blueprint/trihex.rs`, `src/blueprint/matrix.rs` (test scenes),
    `src/svg.rs`
  - `tests/configs.rs`, `tests/reject.rs`, `tests/pipeline.rs`,
    `tests/descriptor.rs`
  - `docs/moods/blueprint/samples/*.json`, `docs/moods/blueprint/README.md`,
    `README.md`

**Interfaces:**
- Produces:
  - **Proto:**
    - `message Background { oneof ground { Trihex trihex = 1; } }`
    - `message Trihex { enum Motion {STATIC=0;SCAN=1;LIGHTS=2;CLOSEOPEN=3;} enum Image {NONE=0;STARFIELD=1;} Motion motion = 1; Image image = 2; }`
  - **Rust:** `params::background::Ground::Trihex(params::Trihex)`,
    `params::trihex::{Motion, Image}`.
  - **`Scene`:** keeps its fields `motion: trihex::Motion` and
    `image: trihex::Image` in this task. An unset `background` resolves to
    `Trihex::default()`.

- [ ] **Step 1: Rewrite the tests that spell configs** to the new shape. Every
  `{"background":{"motion":X,"image":Y}}` becomes
  `{"background":{"trihex":{"motion":X,"image":Y}}}`. That covers:
  - `params.rs` tests, including `valid_configs_are_the_corpus`, whose first
    config becomes
    `{"background":{"trihex":{"image":"NONE","motion":"STATIC"}},"icon":{"hexatri":{"motion":"ROTATE"}},"overlay":{},"seed":0}`
  - `tests/configs.rs` (`an_empty_config_renders_the_defaults`)
  - `tests/reject.rs`
  - `tests/pipeline.rs`

  Also add `"Trihex"` to the names in `tests/descriptor.rs`, and add one more
  test in `params.rs`:
  ```rust
  #[test]
  fn the_old_flat_background_is_rejected() {
      assert!(parse(r#"{"background":{"motion":"SCAN"}}"#).is_err());
  }
  ```
- [ ] **Step 2: Run** `nix develop -c cargo test --lib`. Expected: FAIL, with
  parse errors on the new shape.
- [ ] **Step 3: Implement.**
  - Change the proto.
  - Change `resolve` and `validate` to read `background.trihex`.
  - The CLOSEOPEN error message names `background.trihex.motion` and
    `background.trihex.image`.
  - `valid_configs` emits the nested shape.
  - Update the README's config docs and the blueprint README's config paths
    (`background.motion X` → `background.trihex.motion X`), and the sample
    JSONs.
- [ ] **Step 4: Run** `nix develop -c cargo test`.
  - Expected: every test passes except `the_corpus_is_unchanged`.
  - Its report lists exactly 42 `missing:` and 42 `stale:` entries and zero
    `changed:`.
  - Re-run Task 1 Step 5, and the samples must still be identical.
- [ ] **Step 5: Commit, knowingly red.**
  - Subject: `feat(schema)!: move background motion and image under background.trihex`.
  - Body: the golden test fails by design until the next commit re-keys the
    corpus; no SVG changed.
  - Footer: `BREAKING CHANGE: {"background":{"motion":…}} is now {"background":{"trihex":{"motion":…}}}`.

### Task 3: Re-key the corpus (commit 2 of 2, test data only)

**Files:** `tests/golden/**` only.

- [ ] **Step 1: Record the directory names.**
  `ls tests/golden > /tmp/claude-1000/golden-before.txt`.
- [ ] **Step 2: Regenerate.**
  `nix develop -c cargo test --test golden -- --ignored regenerate_the_corpus`.
- [ ] **Step 3: Prove no SVG moved.**
  - `ls tests/golden | diff /tmp/claude-1000/golden-before.txt -`
  - Expected: no output, meaning the directory names (which are SVG hashes)
    are identical.
  - `git status --porcelain tests/golden | rg -v '_parameters\.json$'`
  - Expected: no output.
- [ ] **Step 4: Run** `nix develop -c cargo test`. Expected: all green.
- [ ] **Step 5: Commit** `test(golden): re-key the corpus for background.trihex`.
  The body states the proof from Step 3: 42 directories unchanged, and only
  the `_parameters.json` file inside each one rewritten.

### Task 4: The mood catalogue

**Files:**
- Create: `src/mood.rs`, `tests/moods.rs`
- Modify: `src/lib.rs` (`pub mod mood;`), `Cargo.toml` (dev-dependency
  `prost-types = "0.14"`)

**Interfaces:**
- Produces:
  - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub enum Mood { Blueprint, Zine }`
  - `impl Mood { pub const ALL: [Mood; 2]; pub fn as_str(self) -> &'static str }`,
    returning `"blueprint"` or `"zine"`
  - `pub const MOODS: &[(&str, &[Mood])]`, starting with
    `background.trihex`, `icon.hexatri`, `icon.ship` and `overlay.matrix`, all
    `Blueprint`. Each later zine task appends its own entries.

- [ ] **Step 1: Write `tests/moods.rs`.** It decodes `params::DESCRIPTOR` as a
  `prost_types::FileDescriptorSet` and contains three tests:
  ```rust
  #[test] fn every_choice_has_a_mood()            // each oneof field of Background, Icon, Overlay as "background.trihex" etc.,
                                                  // plus each Parameters field not in [seed, output, background, icon, overlay]
                                                  // (a mood block), is a key of MOODS
  #[test] fn every_mood_entry_names_a_real_choice() // no MOODS key is absent from the schema
  #[test] fn every_entry_has_at_least_one_mood()
  ```
- [ ] **Step 2: Run** `nix develop -c cargo test --test moods`. Expected: FAIL,
  because `bgsvg::mood` is unresolved.
- [ ] **Step 3: Implement `src/mood.rs`.** Its doc comment says the keys are
  the dotted config paths the studio's `FIELDS` already uses, that settings
  inside a branch inherit the branch's moods, and that an item may carry
  several moods.
- [ ] **Step 4: Run** `nix develop -c cargo test`. Expected: all green.
- [ ] **Step 5: Commit** `feat(mood): catalogue every config choice by mood`.

### Task 5: Page assembly by mood parts (byte-identical)

**Files:**
- Create: `templates/blueprint/under.svg`, `templates/blueprint/over.svg`
- Modify:
  - `src/params.rs`, `src/svg.rs`, `templates/root.svg`
  - `src/blueprint/trihex.rs`, `src/blueprint/matrix.rs` (tests)
  - `tests/configs.rs`

**Interfaces:**
- Consumes: `mood::Mood`.
- Produces:
  - `pub enum Ground { Trihex { motion: trihex::Motion, image: trihex::Image } }`
    in `params`. `Scene` replaces `motion` and `image` with `ground: Ground`.
  - `impl Scene { pub fn uses(&self, mood: Mood) -> bool }`. It returns true
    when the ground, the glyph or the overlay belongs to that mood: trihex,
    hexatri, ship and matrix are `Blueprint`.
  - `blueprint::trihex::pat_trihex(w, h, lat, seed: u32, motion: trihex::Motion, image: trihex::Image, rng) -> String`.
  - **In `svg.rs`:**
    - `defs` = `blueprint` defs (today's `defs.svg`, when the scene uses
      `Blueprint`), followed by zine defs (empty until Task 6, and only when
      the scene uses `Zine`).
    - `css` follows the same rule, using `blueprint::style::css()`.
    - `Root` takes `under`, `rain`, `over`, `icon_transform` and
      `icon_filter: Option<&'static str>` in place of today's `bg`, `cx`,
      `cy` and `clear_r`.
  - `under.svg` holds today's canvas rect and lattice group, and `over.svg`
    holds today's halo circle and vignette rect, both moved verbatim from
    `root.svg`.

- [ ] **Step 1: Add the failing unit test in `src/svg.rs`.**
  ```rust
  #[test]
  fn a_blueprint_scene_uses_only_blueprint() {
      let s = resolve(&parse("{}").unwrap());
      assert!(s.uses(Mood::Blueprint) && !s.uses(Mood::Zine));
  }
  ```
- [ ] **Step 2: Run** `nix develop -c cargo test --lib`. Expected: FAIL to
  compile, because `uses` is not defined.
- [ ] **Step 3: Implement the seam.**
  - Mind askama's `whitespace = "suppress"` and the `+%}` markers noted in
    `trihex.svg`. The byte proof is `the_document_shell_matches_the_corpus`
    plus the golden test, unchanged.
  - Update the `Scene` literals in the trihex and matrix tests, and the
    `scene.motion` and `scene.image` reads in `tests/configs.rs`, to match on
    `ground`.
- [ ] **Step 4: Run** `nix develop -c cargo test`. Expected: all green, with
  no fixture or golden touched.
- [ ] **Step 5: Commit** `refactor(svg): assemble the page from per-mood defs, css and layers`.

### Task 6: Zine foundation and `summit`

**Files:**
- Create:
  - `src/zine/mod.rs`, `src/zine/style.rs`, `src/zine/print.rs`,
    `src/zine/snow.rs`, `src/zine/summit.rs`
  - `templates/zine/defs.svg`, `templates/zine/summit.svg`,
    `templates/zine/snow.svg`
- Modify:
  - `parameters.proto`, `src/params.rs`, `src/color.rs`, `src/mood.rs`
  - `src/svg.rs`, `src/lib.rs` (output prefix)
  - `tests/configs.rs`

**Interfaces:**
- **Proto:**
  - `Zine zine = 6;` in `Parameters`, with `message Zine { string spot = 1; }`
  - `enum Snow { FLURRY = 0; NO_SNOW = 1; BLIZZARD = 2; }` at the top level
  - `Summit summit = 2;` in `Background.ground`, with
    `message Summit { Snow snow = 1; }`
- **`params`:**
  - `Ground::Summit { snow: Snow }`
  - `Scene.spot: String`, normalised lowercase `#rrggbb`, default `#e63946`
  - `uses(Zine)` is true for a summit ground
- **`color`:** `pub fn hex_rgb(s: &str) -> Result<String, String>`. It wraps
  `hex_rgba`, rejects alpha with `bad colour '…': use #rrggbb (print inks take
  no alpha)`, and returns lowercase `#rrggbb`.
- **`zine::style`:** palette constants `SHEET`, `PAPER`, `INK`, `HEAD`,
  `RAYS`, `SPOT`, and `RAINBOW: [&str; 7]`, plus
  `pub fn shade(hex: &str, t: f64) -> String`, which is `mix(hex, "#000000", t)`.
- **`zine::print`:**
  - `pub struct Frame { pub w: f64, pub h: f64, pub s: f64, pub cx: f64, pub cy: f64 }`,
    built with `Frame::new(w: u32, h: u32)`. Here `cy` is the head centre.
  - `pub fn at(&self, x: f64, y: f64) -> (f64, f64)` maps a head-anchored
    mockup point.
  - `pub fn hatch(poly: &[(f64, f64)], spacing: f64) -> Vec<[(f64, f64); 2]>`
    returns 45° lines covering the polygon's bounding box. The template clips
    them with a `clipPath` of the polygon.
  - `pub fn sunburst(c: (f64, f64), reach: f64) -> Vec<String>` returns 14
    wedge point strings (even indices of 28).
- **`zine::snow::snow(f: &Frame, seed: u32, kind: Snow) -> String`**
  - Flakes sit in one `<pattern>` tile drawn from `PyRandom::new("snow:{seed}")`.
  - The tile is `360·s` square for `FLURRY`, with 55 squares of 1.6–2.9·s at
    opacity 0.4–0.9.
  - For `BLIZZARD` the tile is `360·s` wide and `720·s` tall, with 340 streaks
    of length about 10·s at a 5:9 slant.
  - A `<rect>` covering the frame plus one tile period above is filled with
    the pattern and translated by exactly one period, `(0, T)` or `(−W, 2W)`,
    in 18 s (flurry) or 6 s (blizzard). That keeps the loop seamless and the
    file size independent of resolution.
  - `NO_SNOW` returns `""`.
- **`zine::summit::summit(f: &Frame, seed: u32) -> String`** draws the
  under-rain layer: sheet, sunburst, halftone disc and peaks. The geometry
  comes from `PyRandom::new("summit:{seed}")` (algorithm below).
- **`zine::defs(spot: &str, s: f64) -> String`** renders the halftone
  `<pattern id="zht">`: dot radius 2.6·s, period 8·s, rotated 30°.
  **`zine::css() -> String`** returns a static string of every zine rule,
  ending with the reduced-motion `@media` rule. Task 6 starts it with
  `@keyframes zrot{to{transform:rotate(360deg)}}` and
  `.zrays{transform-box:view-box;animation:zrot 120s linear infinite}` (the
  sunburst's `transform-origin` goes inline), plus the snow rules. Later
  tasks append to it.

**Summit algorithm.** The spec and `summit.svg` are the look; these are the
decisions they leave to code.
- **Rows:** a back row of big peaks, one peak behind the head, and a front
  row of small peaks.
- **Back row:**
  - centres at `cx ± (d + 375·s·k)`, for `k ≥ 0`, while `|offset| < w/2 + 200·s`
  - `d = min(185·s, 0.32·w)`
  - height 267·s, half-width 220·s
  - base at `h − 15·s`
- **Behind the head:** at `cx + 10·s`, height 205·s, half-width 140·s.
- **Front row:**
  - centres at `cx ± (270·s + 540·s·k)`
  - height 150·s, half-width 125·s
  - base at `h + 2·s`
- **Jitter:** each peak, drawn left to right and row by row, takes three
  draws: apex x ± 12·s, height × uniform(0.92, 1.08), half-width ×
  uniform(0.92, 1.08).
- **Each peak is drawn as:**
  - the whole triangle in `SHEET`
  - a paper polyline on both slopes only
  - its shadow face `[apex, right foot, (apex.x + 0.18·(right.x − apex.x), base)]`,
    hatched in spot at spacing 6·s and width 1.4·s
  - a paper cap over the top 27 %, with the jagged lower edge used in
    `summit.svg`
- **Ground band:** a `SHEET` band from `h − 17·s` to `h` hides the back-row
  feet.
- **Head-anchored items:** the sunburst at `f.at(320, 190)` and the disc at
  `f.at(320, 180)`, radius 138·s.

- [ ] **Step 1: Write the failing tests.**
  ```rust
  // params.rs
  #[test] fn spot_is_validated_and_normalised() {
      assert!(validate(&parse(r#"{"zine":{"spot":"#e6394680"}}"#).unwrap()).unwrap_err().to_string().contains("zine.spot"));
      assert!(validate(&parse(r#"{"zine":{"spot":"red"}}"#).unwrap()).is_err());
      assert_eq!(resolve(&parse(r##"{"zine":{"spot":" #22C8E5 "}}"##).unwrap()).spot, "#22c8e5");
      assert_eq!(resolve(&parse("{}").unwrap()).spot, "#e63946");
  }
  #[test] fn an_inert_zine_block_is_accepted() {
      assert!(validate(&parse(r##"{"zine":{"spot":"#22c8e5"}}"##).unwrap()).is_ok());
  }
  // tests/configs.rs (new)
  #[test] fn an_inert_zine_block_changes_nothing() {   // blueprint-only scene, with vs without {"zine":{"spot":"#22c8e5"}}: identical SVG
  }
  #[test] fn a_typed_spot_reaches_the_svg_normalised() { // summit with " #22C8E5 ": contains "#22c8e5", not "22C8E5"
  }
  // src/zine/summit.rs
  #[test] fn settings_and_icons_never_move_a_peak()      // the summit layer's points attrs are identical across every snow, spot and glyph
  #[test] fn flanking_peaks_stay_in_a_mobile_frame()     // Frame::new(1080,1920): both back-row apexes nearest cx have 0 < x < w
  #[test] fn a_wide_frame_adds_peaks()                   // back-row count at 3440x1440 > at 1920x1080
  #[test] fn extreme_frames_render_bounded()             // 1x1 and 8000x200 render, each under 1 MB
  // src/zine/snow.rs
  #[test] fn snow_markup_does_not_grow_with_resolution() // FLURRY at 3840x2160 has as many flakes as at 1280x720
  #[test] fn snow_rests_where_it_starts()                // the animated rect carries no transform attribute
  ```
  In `tests/configs.rs`, also scope the existing `<line x1=` (ship) and
  `<clipPath` (starfield) assertions to scenes with `!scene.uses(Mood::Zine)`,
  because zine hatching draws both. Then add:
  - `svg.contains("@keyframes zrot") == scene.uses(Mood::Zine)`
  - `svg.contains("url(#ink)") == matches!(scene.glyph, Glyph::Hexatri{..} | Glyph::Ship)`
  - `!svg.contains("<filter") || !scene.uses(Mood::Zine) || matches!(scene.glyph, Glyph::Hexatri{..} | Glyph::Ship)`
- [ ] **Step 2: Run** `nix develop -c cargo test`. Expected: FAIL to compile
  on `Scene.spot`, `Ground::Summit` and `zine::`.
- [ ] **Step 3: Implement.**
  - **Proto, `resolve` and `validate`:** add the summit schema. The error
    prefix is `zine.spot:`.
  - **`MOODS` entries:** `("background.summit", &[Mood::Zine])` and
    `("zine", &[Mood::Zine])`.
  - **`svg.rs`:** `Ground::Summit` gives `under = summit + snow` and
    `over = ""`.
  - **Label:** the base is `a black print of snowy peaks under a turning sunburst`.
    Then `, with snow falling` (flurry) or `, in a blizzard`, then the
    existing glyph and overlay phrases.
  - **`slug()`:** zine grounds give `{ground}-{snow}-{fg}-{icon}-{overlay}`,
    with snow being `flurry`, `nosnow` or `blizzard`. Trihex slugs are
    unchanged.
  - **The `render()` file prefix** is `trihex` for a trihex ground and `zine`
    otherwise.
  - **`valid_configs`:**
    - add `{"summit":{}}` to the cross product
    - add, with a hexatri glyph and no overlay:
      `{"background":{"summit":{"snow":"NO_SNOW"}}}`,
      `{"background":{"summit":{"snow":"BLIZZARD"}}}`
      and `{"background":{"summit":{}},"zine":{"spot":"#22c8e5"}}`
    - change the doc comment to say it holds one config for every distinct
      picture `validate` allows
- [ ] **Step 4: Run** `nix develop -c cargo test`. Expected: everything passes
  except golden, which lists only `missing:` lines (the new summit configs).
- [ ] **Step 5: Browser check.** Render summit with hexatri at 1080p and at
  `mobile`, screenshot both, and set them beside
  `2026-11-13-zine-mood-reference/summit.svg`.
  - The sheet, the paper slopes, the red hatching on the right-hand faces and
    the disc behind the icon must match.
  - On `mobile`, both flanking peaks must be in frame.
- [ ] **Step 6: Commit** `feat(zine): add the zine mood and its summit background`.
- [ ] **Step 7: Regenerate the corpus.**
  - Run `regenerate_the_corpus`.
  - Check `git status --porcelain tests/golden | rg -v '^\?\?'`. Expected: no
    output, meaning additions only.
  - Commit `test(golden): add the summit configs`.

### Task 7: `treeline`

**Files:**
- Create: `src/zine/treeline.rs`, `templates/zine/treeline.svg`
- Modify: `parameters.proto`, `src/params.rs`, `src/mood.rs`, `src/svg.rs`

**Interfaces:**
- **Proto:** `Treeline treeline = 3;` in `Background.ground`, with
  `message Treeline { Snow snow = 1; }`.
- **`params`:** `Ground::Treeline { snow: Snow }`.
- **`zine::treeline::treeline(f: &Frame, seed: u32) -> String`**, using
  `PyRandom::new("treeline:{seed}")`:
  - **Hills:** `y = base − amp·sin(x/w·1.4π + phase)` for x from −20·s to
    w + 20·s.
    - back hill: base `h − 110·s`, amp 26·s, phase 0.3
    - front hill: base `h − 48·s`, amp 18·s, phase 1.6
  - **Pines:** start at `uniform(−10, 30)·s` and walk across.
    - back row: steps of `uniform(26, 58)·s`, heights `uniform(34, 52)·s`;
      it skips any pine with `|x − cx| < 70·s`
    - front row: steps of `uniform(46, 90)·s`, heights `uniform(48, 70)·s`
    - each pine sits at the hill's y + 4·s
    - one pine is three stacked triangle tiers. Each tier is `SHEET` with a
      paper stroke of 1.8·s, and its right half is hatched in spot at 4.5·s.
      A paper trunk sits underneath.
  - **Moon:** a halftone disc at `f.at(320, 170)`, radius 132·s. No sunburst.

- [ ] **Step 1: Write the failing tests** in `src/zine/treeline.rs`:
  - `no_back_row_pine_stands_before_the_head`
  - `the_hills_span_an_ultrawide_frame` (first and last hill x lie beyond
    0 and w at 3440x1440)
  - `settings_and_icons_never_move_a_pine`
  - `extreme_frames_render_bounded` (as in Task 6)
- [ ] **Step 2: Run** `nix develop -c cargo test --lib treeline`. Expected: FAIL.
- [ ] **Step 3: Implement**, including:
  - `MOODS` entry `("background.treeline", &[Mood::Zine])`
  - the label base `a black print of a pine treeline under a halftone moon`
  - `{"treeline":{}}` in the `valid_configs` cross product
- [ ] **Step 4: Run** `nix develop -c cargo test`. Expected: golden shows only
  `missing:` lines, and everything else passes.
- [ ] **Step 5: Browser check** against `treeline.svg`, at 1080p and at
  `ultrawide`.
- [ ] **Step 6: Commit** `feat(zine): add the treeline background`.
- [ ] **Step 7: Regenerate (additions only) and commit**
  `test(golden): add the treeline configs`.

### Task 8: `halfpipe`

**Files:**
- Create: `src/zine/halfpipe.rs`, `templates/zine/halfpipe.svg`
- Modify: `parameters.proto`, `src/params.rs`, `src/mood.rs`, `src/svg.rs`

**Interfaces:**
- **Proto:** `Halfpipe halfpipe = 4;` in `Background.ground`, with
  `message Halfpipe { Snow snow = 1; }`.
- **`params`:** `Ground::Halfpipe { snow: Snow }`.
- **`zine::halfpipe::halfpipe(f: &Frame) -> String`.** It has no seeded
  geometry; the seed varies only the snow.
  - **The cross-section helper:**
    `quarter(cx, floor_y, half_floor, lip_dx, lip_dy, side)` returns 15
    points, `x = cx + side·(half_floor + lip_dx·cos a)` and
    `y = floor_y − lip_dy + lip_dy·sin a` for `a` in `[0, π/2]`.
    `u_shape` is the left quarter followed by the reversed right quarter.
  - **The near section** is frame-anchored: lips at `x = cx ± (w/2 + 8·s)`,
    `lip_dx = 118·s`, `lip_dy = 168·s`, floor at `h − 8·s`.
  - **The far section** is head-anchored: floor at `f.cy + 98·s`,
    `half_floor = 98·s`, `lip_dx = 62·s`, `lip_dy = 84·s`.
  - **Ribs:** three, at t = 0.3, 0.55 and 0.78, interpolating every parameter
    between the two sections.
  - **Fills:** the left wall is hatched in spot at 5.5·s; the right wall is
    `SHEET`; the floor is `SHEET` with `url(#zht)` at opacity 0.6.
  - **Lines:** lips, coping and deck lines as in `halfpipe.svg`.
  - **Head-anchored items:** the sunburst at `f.at(320, 150)` and the disc at
    `f.at(320, 168)`, radius 132·s.

- [ ] **Step 1: Write the failing tests.**
  - `the_far_section_is_wider_than_the_head`: the far lips are more than
    `2·56·1.06·s` apart.
  - `the_floor_runs_under_the_head`: the far floor y is greater than the head
    bottom `f.cy + 64·1.06·s`.
  - `the_near_lips_meet_the_frame_edges`: true at 1920x1080, `mobile` and
    `ultrawide`.
  - `extreme_frames_render_bounded`.
- [ ] **Step 2: Run.** Expected: FAIL.
- [ ] **Step 3: Implement**, including:
  - `MOODS` entry `("background.halfpipe", &[Mood::Zine])`
  - the label base `a black print of a snow halfpipe under a turning sunburst`
  - `{"halfpipe":{}}` in the `valid_configs` cross product
- [ ] **Step 4: Run.** Expected: golden shows only `missing:` lines, and
  everything else passes.
- [ ] **Step 5: Browser check** against `halfpipe.svg`, at 1080p and at
  `mobile`.
- [ ] **Step 6: Commit** `feat(zine): add the halfpipe background`.
- [ ] **Step 7: Regenerate (additions only) and commit**
  `test(golden): add the halfpipe configs`.

### Task 9: Rider gear, the sticker, and the punk at rest

**Files:**
- Create:
  - `src/zine/rider.rs`, `src/zine/punk.rs`
  - `templates/zine/rider.svg`, `templates/zine/punk.svg`
- Modify:
  - `parameters.proto`, `src/params.rs`, `src/mood.rs`, `src/svg.rs`,
    `src/zine/mod.rs` (css)
  - `tests/configs.rs`

**Interfaces:**
- **Proto:**
  - `message Rider { enum Gear { BOARD = 0; CROSSED_SKIS = 1; } Gear gear = 1; bool counter_clockwise = 2; optional uint32 spin_rpm = 3; }`
  - `message Punk { Rider rider = 1; }`
  - `Punk punk = 3;` in `Icon.glyph`
- **`params`:**
  - `pub struct Rig { pub gear: rider::Gear, pub ccw: bool, pub rpm: u32 }`,
    where an unset `spin_rpm` resolves to 8
  - `Glyph::Punk(Rig)`, extended in Task 10
  - `validate` rejects `spin_rpm > 30` with prefix `icon.punk.rider.spin_rpm:`
- **`zine::rider`:**
  - `pub struct Sticker { pub silhouette: String, pub art: String }`.
    Silhouette markup carries no `fill` or `stroke` attributes, so it inherits
    them.
  - `pub fn icon(f: &Frame, spot: &str, rig: &Rig, head: &Sticker) -> String`
    renders `rider.svg`. Its layers, in order:
    1. Shadows: the gear silhouette inside
       `<g transform="translate(4,5)"><g class="zspin" …>`, then the head
       silhouette, both `fill`/`stroke` spot.
    2. Outlines: the gear and the head silhouettes, `fill`/`stroke` paper,
       with stroke width 12 and round joins.
    3. Art: the gear art, then the head art.
  - The shadow offset sits outside the spin, so it stays fixed in screen
    space.
  - The whole icon group is `translate(cx, cy) scale(1.06·s)`, with no filter.
  - Spin: when `rpm > 0`, each gear group gets `class="zspin"` and
    `style="animation-duration:{60/rpm}s"`, plus
    `;animation-direction:reverse` when `ccw`. When `rpm == 0` there is no
    class. With no transform attribute, the gear rests horizontal.
  - Gear art follows the spec: the board is 216×42 with bindings at
    ±[64, 86]; the crossed skis are 216×26 at ±32°.
- **`zine::punk::head() -> Sticker`.** The geometry follows the spec's punk
  section (sphere-wrap φ = 35°, eyes, banana grin, seven-spike crest with
  lit/shade halves, back spikes behind the head). Each spike is a
  `<g class="zsp" style="--i:{k}">` with k = 0 (red, forehead) to 6
  (violet, nape). Lower teeth are wrapped in `<g class="zlo">`, and each eye
  in `<g class="zeye">`.
- **`svg.rs`:** zine glyphs get `icon_filter = None` and the zine transform.
  The label phrase is ` with a punk head with a rainbow mohawk`, followed by
  ` over a spinning snowboard`, ` over spinning crossed skis`,
  ` over a snowboard` or ` over crossed skis`.

- [ ] **Step 1: Write the failing tests.**
  ```rust
  // src/zine/punk.rs
  #[test] fn the_crest_runs_red_at_the_forehead_to_violet_at_the_nape() // --i:0 carries #ff3b3b, --i:6 carries #c04bff
  #[test] fn the_grin_has_six_upper_and_five_lower_teeth()
  #[test] fn every_wrapped_feature_is_on_the_visible_hemisphere()       // depth > 0 for all drawn eye and teeth points
  // src/zine/rider.rs
  #[test] fn the_sticker_is_geometry_not_a_filter()                     // icon(...) contains no "<filter"
  #[test] fn unset_spin_is_eight_turns_a_minute()                       // "animation-duration:7.5s"
  #[test] fn zero_rpm_rests_horizontal()                                // no "zspin", no rotate/transform on the gear groups
  #[test] fn counter_clockwise_reverses()
  #[test] fn the_shadow_offset_sits_outside_the_spin()
  // params.rs
  #[test] fn spin_rpm_is_capped_and_zero_means_still()                   // 31 rejected; {"spin_rpm":0} resolves to rpm 0
  // tests/configs.rs
  #[test] fn counter_clockwise_without_spin_changes_nothing()           // {"rider":{"spin_rpm":0}} vs {"rider":{"spin_rpm":0,"counter_clockwise":true}}: identical SVG
  ```
- [ ] **Step 2: Run** `nix develop -c cargo test`. Expected: FAIL.
- [ ] **Step 3: Implement**, including:
  - `MOODS` entry `("icon.punk", &[Mood::Zine])`
  - the `.zspin` rule
    (`transform-box:fill-box;transform-origin:center;animation:zrot linear infinite`)
    in `zine::css()`, reusing Task 6's `zrot` keyframes
  - `slug()`: fg `spin`/`still`, icon `punk`
  - `valid_configs`:
    - `{"punk":{}}` in the glyph cross product
    - on a summit base: crossed skis, `counter_clockwise`, `spin_rpm: 0` and
      `spin_rpm: 20`
- [ ] **Step 4: Run.** Expected: golden shows only `missing:` lines, and
  everything else passes.
- [ ] **Step 5: Browser check** against `punk.svg` (shape only, at rest) and
  `crossed-skis.svg`. Also check the punk on today's trihex lattice, the
  cross-mood case.
- [ ] **Step 6: Commit** `feat(zine): add the punk icon with board and crossed skis`.
- [ ] **Step 7: Regenerate (additions only) and commit**
  `test(golden): add the punk configs`.

### Task 10: Head motion, tempo, and the punk's animations

**Files:**
- Modify: `parameters.proto`, `src/params.rs`, `src/zine/rider.rs`,
  `src/zine/punk.rs`, `src/zine/mod.rs` (css), `tests/configs.rs`

**Interfaces:**
- **Proto:**
  - `Rider` gains `enum Motion { STILL = 0; BOB = 1; SWAY = 2; } Motion motion = 4; uint32 bpm = 5;`
  - `Punk` gains `bool crest_wave = 2; bool rainbow_flow = 3; bool chatter = 4; enum Eyes { STILL = 0; POP = 1; } Eyes eyes = 5;`
- **`params`:**
  - `Rig` gains `motion: rider::Motion, bpm: u32`, where 0 resolves to 96
  - `Glyph::Punk(PunkLook)` with
    `pub struct PunkLook { pub rig: Rig, pub crest_wave: bool, pub rainbow_flow: bool, pub chatter: bool, pub eye_pop: bool }`.
    It is not named `Punk`, because `params` re-exports prost's generated
    `Punk`.
  - `zine::punk::head()` becomes `head(p: &PunkLook) -> Sticker`
  - `validate` rejects a `bpm` that is not 0 and outside 40–200
- **Rider:**
  - the icon group carries `style="--beat:{60/bpm}s"`
  - the head silhouette, outline and art groups each get `class="zbob"` or
    `class="zsway"` when motion is set
- **Punk classes:** each animation's class goes on the root `<g>` of the punk
  art and silhouette only when its toggle is on: `zwave`, `zflow`, `zchat`,
  `zpop`. CSS descendant rules drive the parts: `.zwave .zsp`, `.zflow`
  (crest group), `.zchat .zlo` and `.zpop .zeye`.
- **Keyframes,** with every duration in `var(--beat)` (exact values in
  `punk.svg`'s style):

  | rule | motion |
  |---|---|
  | `zbob` | translateY −7, over 1 beat |
  | `zsway` | rotate ±8° about 50% 100%, over 2 beats, alternate |
  | `zwave` | rotate ±7° about 50% 100%, over 1 beat, delay `var(--i)·beat·−0.13` |
  | `zflow` | crest hue, over 8 beats |
  | `zchat` | translateY 4 in 2 steps, over ½ beat |
  | `zpop` | scale 1.45 at 4 %, over 4 beats |

- [ ] **Step 1: Write the failing tests.**
  - `toggles_add_their_class_and_nothing_else`: for each toggle, the
    `points=` attributes match the toggle-off render.
  - `the_beat_follows_bpm`: 120 BPM gives `--beat:0.5s`, unset gives
    `--beat:0.63s`. That is `geom::fmt(60/96)`; assert the exact `fmt` output.
  - `bpm_out_of_range_is_rejected`: 39 and 201 are rejected, 0 and 40 pass.
  - `bpm_without_any_head_animation_changes_nothing`, in
    `tests/configs.rs`: the SVG is identical. That means `--beat` is emitted
    only when some beat-driven animation is on.
  - `every_animation_on_still_rests_under_reduced_motion`: with every punk
    toggle and BOB on, the markup carries no inline transform on the head or
    the gear, and zine CSS contains the reduced-motion rule.
- [ ] **Step 2: Run.** Expected: FAIL.
- [ ] **Step 3: Implement the keyframes and the classes.**
- [ ] **Step 4: Measure rainbow flow before keeping it.**
  - Build a throwaway HTML harness in the scratchpad that shows a 1080p punk
    on summit and counts `requestAnimationFrame` callbacks over 10 s in
    headless chromium.
  - Compare variant A (`filter: hue-rotate()` on the crest group) with
    variant B (each spike's two fills cycling the seven rainbow colours in
    `@keyframes`, offset by `--i`). Run them back to back, three runs each,
    as `docs/moods/blueprint/README.md` measured.
  - Keep A only if its median is ≥ 95 % of B's. Otherwise ship B.
  - Write down the numbers and the decision; Task 13 puts them in the zine
    README.
- [ ] **Step 5: Run** `nix develop -c cargo test`. Expected: golden shows only
  `missing:` lines, and everything else passes.
- [ ] **Step 6: Browser check** with every punk animation on, at 96 BPM,
  against `punk.svg`.
- [ ] **Step 7: Commit** `feat(zine): animate the rider heads to a shared beat`.
- [ ] **Step 8: Extend `valid_configs` and regenerate.** On the summit + punk
  base, add BOB, SWAY, `{"motion":"BOB","bpm":150}`, each toggle and eyes POP.
  Regenerate (additions only) and commit `test(golden): add the punk animation configs`.

### Task 11: The park rider

**Files:**
- Create: `src/zine/park.rs`, `templates/zine/park.svg`
- Modify: `parameters.proto`, `src/params.rs`, `src/mood.rs`, `src/svg.rs`,
  `src/zine/mod.rs` (css)

**Interfaces:**
- **Proto:** `message Park { Rider rider = 1; bool pompom = 2; bool lens_flow = 3; bool glint = 4; bool breath = 5; string beanie = 6; }`,
  and `Park park = 4;` in `Icon.glyph`.
- **`params`:**
  - `Glyph::Park(ParkLook)` with
    `pub struct ParkLook { pub rig: Rig, pub pompom: bool, pub lens_flow: bool, pub glint: bool, pub breath: bool, pub beanie: String }`
  - an empty `beanie` resolves to `Scene.spot`; otherwise it goes through
    `hex_rgb`, and the error prefix is `icon.park.beanie:`
- **`zine::park::head(spot: &str, p: &ParkLook) -> Sticker`.**
  Transcribe the art from `park-rider.svg`'s head group:
  - the beanie fill is `beanie`, and the beanie hatching stays `INK`
  - the gaiter zig-zag is spot
  - the lens uses a rainbow `linearGradient` id `zlens`
  - the pompom is wrapped in `<g class="zpom">`
  - the lens is wrapped in `<g class="zlensg">`
  - the glint bar and the breath puffs are emitted only when their toggles
    are on, and rest at `opacity:0`
- **Keyframes:**

  | rule | motion |
  |---|---|
  | `zpom` | rotate ±16° about 50% 100%, over 1 beat |
  | `zlensflow` | the same mechanism Task 10 kept for rainbow flow, over 8 beats |
  | `zglint` | translateX 130 with an opacity flash, over 4 beats |
  | `zbreath` | puffs fade in, rise and fade, over 2 beats |

- [ ] **Step 1: Write the failing tests.**
  - `the_beanie_follows_the_spot_unless_set`
  - `the_beanie_hatching_stays_ink`
  - `beanie_rejects_alpha_and_bad_hex`
  - `a_typed_beanie_reaches_the_svg_normalised`
  - `glint_and_breath_exist_only_when_on`
  - `glint_and_breath_rest_invisible`: their base rule or attribute is
    opacity 0, and only the keyframes raise it.
- [ ] **Step 2: Run.** Expected: FAIL.
- [ ] **Step 3: Implement**, including:
  - `MOODS` entry `("icon.park", &[Mood::Zine])`
  - the label phrase ` with a snowboarder in a beanie and goggles` plus the
    gear phrase
  - the slug icon `park`
  - `{"park":{}}` in the glyph cross product
- [ ] **Step 4: Run.** Expected: golden shows only `missing:` lines, and
  everything else passes.
- [ ] **Step 5: Browser check** against `park-rider.svg`, with all animations
  on, then with a cyan beanie on a red spot.
- [ ] **Step 6: Commit** `feat(zine): add the park rider icon`.
- [ ] **Step 7: Extend `valid_configs` and regenerate.** Add each park toggle
  and `{"beanie":"#ffd400"}` on a summit base. Regenerate (additions only) and
  commit `test(golden): add the park rider configs`.

### Task 12: The mountain local

**Files:**
- Create: `src/zine/local.rs`, `templates/zine/local.svg`
- Modify: `parameters.proto`, `src/params.rs`, `src/mood.rs`, `src/svg.rs`,
  `src/zine/mod.rs` (css)

**Interfaces:**
- **Proto:** `message Local { Rider rider = 1; bool beard_wind = 2; bool frost = 3; bool glint = 4; bool breath = 5; }`,
  and `Local local = 5;` in `Icon.glyph`.
- **`params`:** `Glyph::Local(LocalLook)` with
  `pub struct LocalLook { pub rig: Rig, pub beard_wind: bool, pub frost: bool, pub glint: bool, pub breath: bool }`.
- **`zine::local::head(spot: &str, l: &LocalLook) -> Sticker`.** Transcribe the
  art from `mountain-local.svg`:
  - the beard mass path and the five locks, each a
    `<g class="zlock" style="--i:{k}">`
  - the hatching clipped to the beard mass path, which must not be a
    rectangle
  - frost crosses, each a `<g class="zfrost" style="--i:{k}">`
  - glint and breath reuse Task 11's classes and resting rules
- **Keyframes:**

  | rule | motion |
  |---|---|
  | `zlock` | rotate ±9° about 50% 0, over 1 beat, delay `var(--i)·beat·−0.18` |
  | `zfrost` | scale 0.4 to 1.4 twinkle, over 2 beats, delay `var(--i)·beat·−0.29` |

- [ ] **Step 1: Write the failing tests.**
  - `the_beard_hatching_is_clipped_to_the_beard`: the hatch `clipPath` holds
    a `<path` equal to the beard mass, not a `<rect`.
  - `the_beard_ends_in_five_locks`.
  - `wind_moves_only_the_locks`: toggling `beard_wind` changes class
    attributes only.
- [ ] **Step 2: Run.** Expected: FAIL.
- [ ] **Step 3: Implement**, including:
  - `MOODS` entry `("icon.local", &[Mood::Zine])`
  - the label phrase ` with a bearded mountain local` plus the gear phrase
  - the slug icon `local`
  - `{"local":{}}` in the glyph cross product
- [ ] **Step 4: Run.** Expected: golden shows only `missing:` lines, and
  everything else passes.
- [ ] **Step 5: Browser check** against `mountain-local.svg`, with all
  animations on.
- [ ] **Step 6: Commit** `feat(zine): add the mountain local icon`.
- [ ] **Step 7: Extend `valid_configs` and regenerate.** Add each local toggle
  on a summit base. Regenerate (additions only) and commit
  `test(golden): add the mountain local configs`.

### Task 13: Zine docs, samples, ADRs

**Files:**
- Create:
  - `docs/moods/zine/README.md`
  - `docs/moods/zine/samples/*.json` and `*.svg`
  - `docs/adr/0016-moods-as-folders-and-a-catalogue.md`
  - `docs/adr/0017-background-becomes-a-oneof.md`
- Modify: `CLAUDE.md`, `README.md`, `docs/ROADMAP.md`

- [ ] **Step 1: Write the zine README** in the blueprint README's shape: the
  mood in one line, the five rules (verbatim from the spec's Docs section),
  the palette table, the sticker, and each background and icon with the
  reason for its constraints. Include the Task 10 measurement numbers and the
  choice they forced.
- [ ] **Step 2: Add the samples.**
  - Each sample is a config with `output.file` set to `1280x720`, except two:
    one `mobile` and one `ultrawide`.
  - The set: summit + punk (all animations), treeline + park, halfpipe +
    local, summit + hexatri (cross-mood), trihex LIGHTS + punk (cross-mood),
    summit + punk at `mobile`, treeline + local at `ultrawide`.
  - Render them with `target/release/bgsvg`.
- [ ] **Step 3: Browser check every sample.** No element may be cut off on
  `mobile` or `ultrawide`.
- [ ] **Step 4: Write the ADRs and update the other docs.** Use the
  `writing-adrs` skill for 0016 and 0017, with the alternatives from the
  spec's "Approaches considered".
  - **CLAUDE.md:**
    - the module layout
    - moods and `mood::MOODS` as public API
    - `docs/moods/` as the mood contracts (read the relevant one before
      touching anything visual)
    - the per-mood page assembly
    - the zine reduced-motion and no-filter rules
    - the new `valid_configs` contract
    - the 42 → N corpus count
  - **README.md:** the config docs for every new field.
  - **ROADMAP:** a Done entry linking the spec, the plan and the two ADRs, and
    a Planned entry for the studio's mood filter.
- [ ] **Step 5: Final check.**
  - Run `nix develop -c cargo test`,
    `nix develop -c cargo clippy --all-targets -- -D warnings`,
    `nix develop -c cargo fmt --check` and `nix build`. Expected: all green.
  - `git log --oneline master..` shows every `test(golden): …` commit
    touching only `tests/golden/`. Check with
    `git show --stat <sha> | rg -v tests/golden`.
- [ ] **Step 6: Commit** `docs(zine): document the zine mood, its samples and decisions`.
