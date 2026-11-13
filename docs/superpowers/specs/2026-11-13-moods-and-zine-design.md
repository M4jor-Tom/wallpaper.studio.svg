# Moods, and a second one: `zine`

*2026-11-13*

## Intent

Today the crate has exactly one look: the clinical high-altitude blueprint
recorded in `docs/mood/`. Every rule there (no black, no warmth, two
desaturated hues, slow ambient motion) is written as if it governed every
possible render.

The user wants a second, deliberately opposite look: a punk/snow-culture
**zine** print, with a black round head with a rainbow mohawk spinning a
snowboard behind it, two more rider heads, and three snowy backgrounds. They
want the code itself to carry the idea of a **mood**, so that the studio
(`wallpaper.studio.ui`) can later filter items by mood. And mixing moods in
one scene (a zine head on the blueprint lattice, the matrix rain over a
mountain range) must stay possible.

## Goals

- **Moods are a concept in the code.** Each mood owns a folder of code,
  templates and docs. A public catalogue tags every config choice with its
  moods, and the studio will filter on that catalogue later.
- **Blueprint is mood #1.** Today's code moves under it unchanged, and
  `docs/mood/` becomes `docs/moods/blueprint/`.
- **Zine is mood #2**, with:
  - three backgrounds: `summit`, `treeline` and `halfpipe`
  - three icons: `punk`, `park` (park rider) and `local` (mountain local)
  - the settings listed below
- **Any background with any icon with any overlay.** Cross-mood scenes are
  valid configs and render.

## Non-goals

- **No studio changes.** `wallpaper.studio.ui` pins a revision through
  `Cargo.lock`, so it keeps working until someone runs `cargo update -p
  bgsvg`. Adopting the new schema there, and the mood filter, are later work.
- **No visual change to blueprint.** That includes no colour adaptation when
  blueprint items share a scene with zine items. Each item keeps its own look,
  even where a mix looks poor (for example, hexatri's dark steel strokes are
  faint on the black sheet).
- **Explored and dropped:**
  - summit settings for sky, range, shading and print
  - the girly heads, the freeride, backcountry, grom and 80s heads
  - the headbang animation and the eye twist
  - side-by-side skis
  - the riso, collage and positive-print backgrounds

## Anti-goals

These would make the work a failure even if every feature rendered.

- **Any byte moves in the 42 existing renders.** ADR 0009 treats byte
  identity as the correctness proof, and it applies here: the restructure must
  leave every existing SVG identical.
- **Zine CSS or defs appear in an all-blueprint render.** That is the same
  failure as above, seen from a different angle.
- **An SVG `filter` wraps animated content.** The mockups did this for the
  sticker outline (`feMorphology`). Wrapped around a spinning board, it is
  re-rasterised every frame. `docs/mood/README.md` records how much frame rate
  this project measured and won by not painting work it does not need.
- **A template computes a value, or a config string reaches a template
  unvalidated.** The rule from CLAUDE.md applies: the `Text` escaper is safe
  only because every value is computed or checked at the `params` boundary.

## Constraints

- **Determinism.** Geometry depends only on `seed`. Each zine generator draws
  from its own `PyRandom` stream (for example `"summit:{seed}"`,
  `"treeline:{seed}"`, `"snow:{seed}"`), never from `"trihex:{seed}"`. It uses
  the crate's CPython-compatible RNG and no other.
- **Reduced motion.** Every animated element's un-animated state is its
  resting state:
  - the board rests horizontal
  - the head rests upright and centred
  - elements that exist only in motion (glint, breath) rest invisible
  - the sunburst and the snow rest where they start

  The existing `@media (prefers-reduced-motion){*{animation:none!important}}`
  is the mechanism, so resting states must hold with every animation removed.
- **Purity.** `tests/purity.rs` forbids `"<` in production Rust and reads only
  the top level of `src/`. Once code moves into subfolders it must walk
  recursively, or the rule silently stops applying to everything that moved.
- **Public API.** `render_to_string`, `load` and
  `params::{parse_res, RESOLUTIONS, DESCRIPTOR}` keep their paths. The new
  catalogue joins them. Module paths of the drawing code (`bgsvg::trihex` and
  so on) may move, because no consumer uses them.
- **CSS class names.** Today's CSS uses unprefixed class names (`spin`,
  `rspin`, `scan`, `light` and others), and `.spin` is hexatri's 24 s
  rotation. Zine classes need their own prefix so a cross-mood scene cannot
  collide.

## Approaches considered

1. **A folder per mood, plus a catalogue (picked).** Blueprint code moves
   as-is into `src/blueprint/`, with the matching `templates/blueprint/` and
   `docs/moods/blueprint/`. Zine gets the same three folders.
   `params::MOODS` tags every config choice. Zine-wide settings get their own
   `zine` block in the config.
   - **Cost:** the largest diff, because of the file moves.
   - **Gain:** the code reads by mood, every existing render stays identical,
     and the next mood is a new folder.
2. **Catalogue only.** The same `Mood` type, catalogue and `zine` block, with
   files left flat. A smaller diff, but moods exist only as data.
3. **Moods as custom options in `parameters.proto`.** This would be a single
   source of truth. It was rejected because prost discards custom options when
   it decodes a descriptor, so both this crate and the studio would need a
   hand-written descriptor reader.

**The schema change.** `background` becomes a `oneof`, and today's `motion`
and `image` move under `background.trihex`. This breaks existing configs:
`{"background":{"motion":"SCAN"}}` becomes
`{"background":{"trihex":{"motion":"SCAN"}}}`.

The non-breaking alternative was to keep the two fields and reject them on
zine backgrounds in `validate()`. That is the "add a check instead of moving a
field" pattern CLAUDE.md steers away from. The user accepted the break.

## Decided

### Moods in code

```text
src/
  lib.rs, params.rs, svg.rs, geom.rs, rng.rs   shared: boundary, page assembly, geometry, RNG
  mood.rs                                       Mood + MOODS
  blueprint/  trihex.rs matrix.rs icon.rs style.rs   (moved, contents unchanged)
  zine/       style.rs summit.rs treeline.rs halfpipe.rs snow.rs rider.rs punk.rs park.rs local.rs
templates/blueprint/…   templates/zine/…
docs/moods/blueprint/   (was docs/mood/, moved as-is)
docs/moods/zine/        README with the zine rules (below) and samples/
```

The exact file split inside `zine/` is the builder's call. What is decided:

- **What is shared.** Mood-neutral helpers stay shared: `hex_rgba`, `mix`,
  `darken`, `geom::fmt` and `pts`.
- **The catalogue.** It is public and keyed by the dotted config path of every
  `oneof` branch, which is the vocabulary the studio's `FIELDS` already uses:

  ```rust
  pub enum Mood { Blueprint, Zine }          // with as_str(): "blueprint" / "zine"
  pub const MOODS: &[(&str, &[Mood])] = &[
      ("background.trihex", &[Mood::Blueprint]),
      ("background.summit", &[Mood::Zine]),
      ("background.treeline", &[Mood::Zine]),
      ("background.halfpipe", &[Mood::Zine]),
      ("icon.hexatri", &[Mood::Blueprint]),
      ("icon.ship", &[Mood::Blueprint]),
      ("icon.punk", &[Mood::Zine]),
      ("icon.park", &[Mood::Zine]),
      ("icon.local", &[Mood::Zine]),
      ("overlay.matrix", &[Mood::Blueprint]),
      ("zine", &[Mood::Zine]),               // the mood-wide settings block
  ];
  ```

  An item may carry several moods, which is why each entry holds a slice.
  Settings inside a branch inherit the branch's moods.
- **The coverage test.** A test walks `DESCRIPTOR` and fails if any `oneof`
  branch under `Background`, `Icon` or `Overlay`, or any top-level mood block,
  is missing from `MOODS`.
  - The crate already compiles `prost-types` for the build script. Declaring
    it as a dev-dependency adds nothing new to the tree.
  - Matching names in a string, as `tests/descriptor.rs` does today, is not
    enough, because a new branch would slip through.

### Schema

The following is a sketch. Field numbers and the enum scoping are for the
builder to finalise.

```proto
message Parameters {
  uint32     seed       = 1;
  Output     output     = 2;
  Background background = 3;  // unset -> trihex, static, no image (unchanged picture)
  Icon       icon       = 4;
  Overlay    overlay    = 5;
  Zine       zine       = 6;  // zine-mood settings, read by every zine item in the scene
}

message Zine { string spot = 1; }  // #rrggbb; empty = #e63946

message Background {
  oneof ground {
    Trihex   trihex   = 1;
    Summit   summit   = 2;
    Treeline treeline = 3;
    Halfpipe halfpipe = 4;
  }
}
message Trihex {                     // today's Background, moved
  enum Motion { STATIC = 0; SCAN = 1; LIGHTS = 2; CLOSEOPEN = 3; }
  enum Image  { NONE = 0; STARFIELD = 1; }
  Motion motion = 1;
  Image  image  = 2;
}
enum Snow { FLURRY = 0; NO_SNOW = 1; BLIZZARD = 2; }   // zero = the approved look
message Summit   { Snow snow = 1; }
message Treeline { Snow snow = 1; }
message Halfpipe { Snow snow = 1; }

message Icon {
  oneof glyph {
    Hexatri hexatri = 1;
    Ship    ship    = 2;
    Punk    punk    = 3;
    Park    park    = 4;
    Local   local   = 5;
  }
}
message Rider {                      // shared by the three zine heads
  enum Gear   { BOARD = 0; CROSSED_SKIS = 1; }
  enum Motion { STILL = 0; BOB = 1; SWAY = 2; }
  Gear   gear              = 1;
  bool   counter_clockwise = 2;
  optional uint32 spin_rpm = 3;      // 0-30; unset = 8; 0 = the gear does not spin
  Motion motion            = 4;
  uint32 bpm               = 5;      // 40-200; 0 = 96
}
message Punk  { Rider rider = 1; bool crest_wave = 2; bool rainbow_flow = 3; bool chatter = 4;
                enum Eyes { STILL = 0; POP = 1; } Eyes eyes = 5; }
message Park  { Rider rider = 1; bool pompom = 2; bool lens_flow = 3; bool glint = 4; bool breath = 5;
                string beanie = 6; }   // #rrggbb; empty = follow zine.spot
message Local { Rider rider = 1; bool beard_wind = 2; bool frost = 3; bool glint = 4; bool breath = 5; }
```

Notes on the sketch:

- **`spin_rpm` is `optional`.** The user's slider makes 0 mean "still", while
  the crate's rule is that a zero value is the default. Presence settles the
  conflict.
- **`bpm` uses zero as the default,** since 0 lies outside the 40–200 range.
- **`validate()` gains these checks:**
  - `zine.spot` and `park.beanie` pass through `hex_rgba`, accepting
    `#rrggbb` only, with no alpha, because they are print inks.
  - `spin_rpm` is at most 30.
  - `bpm` is 0 or within 40–200.

  These are the only new strings and numbers that reach templates, so routing
  them through `validate()` is the standing obligation from CLAUDE.md.
- **Settings with no visible effect are accepted, not rejected** (my call,
  after the builder review). Examples:
  - a `zine` block when no zine item is in the scene
  - `counter_clockwise` with `spin_rpm: 0`
  - `bpm` with every head animation off

  The reasons:
  - **Cross-mood editing.** Switching the icon in the studio should never turn
    a saved config invalid. That would be the effect of rejecting `zine.spot`
    when the scene has only blueprint items.
  - **Forward compatibility.** Accepting now and rejecting later would break
    saved configs, while the reverse breaks nothing.
  - **The precedent does not apply.** `CLOSEOPEN` + `NONE` is rejected because
    the combination is broken: there is nothing to reveal. These are only
    inert.

  `valid_configs` changes its contract accordingly, from "every config
  validate keeps" to "one config for every distinct picture validate allows".
  Update the doc comment and CLAUDE.md to say so.
- **Snow is per background.** Each zine background owns the setting. Moving it
  onto `Background` would need a rule rejecting it on `trihex`.
- **The spot colour is per mood,** not per item. With one value, the
  background and the head can never disagree. A zine icon on the blueprint
  lattice still reads `zine.spot`.

### The zine look

The reference renders in `2026-11-13-zine-mood-reference/` are the approved
designs.

**Which source wins** (my call, after the builder review):
- **The reference files win** on geometry, positions, colours and timing,
  because they are what the user approved on screen. Where this prose gives a
  number that disagrees with them, the prose is the paraphrase.
- **This prose wins** on mechanics: how a thing is drawn, scaled, seeded, and
  how it rests under reduced motion. The files are throwaway mockup output and
  use an `feMorphology` filter for the sticker outline, which is exactly what
  the anti-goals forbid.
- **Scale:** the files are drawn on a 640×360 canvas with the head centred at
  (320, 194) and scaled ×1.06 from a 200-unit glyph grid. Sizes scale by
  `min(w, h) / 360`, as the blueprint scales by `u`. Positions follow the
  layout rules below.

**Layout at any aspect ratio** (my call, after the builder review). The
mockups were all 16:9, but three of the eight resolution presets are portrait
or square, and the blueprint lattice fills any frame. Zine backgrounds must
too.
- **Head-anchored:** the head, the halftone disc and the sunburst stay at the
  frame centre with the reference offsets (sunburst at (320, 190) on summit
  and (320, 150) on halfpipe; disc at (320, 180), (320, 170) and (320, 168)),
  scaled from the centre.
- **Frame-anchored:**
  - Horizontal scenery spans the frame width: summit peaks are placed at
    fractions of the width, treeline hills and pine rows run edge to edge, and
    the halfpipe's near section spans the full width.
  - Heights scale with `min(w, h)`.
  - The ground, the hill feet and the halfpipe's near floor are anchored to
    the bottom edge.
- **Summit:** on a wide frame it adds peaks rather than stretching five. On a
  narrow one it keeps the two flanking peaks inside the frame.
- **Proof:** the zine samples in `docs/moods/zine/samples/` include `mobile`
  and `ultrawide` renders, and the plan has a browser check on them, because
  the corpus renders only at 1080p.

**Palette:**

| role | value |
|---|---|
| sheet | `#111114` |
| paper (lines, snowcaps, snow, sticker edge) | `#f3efe6` |
| ink on paper | `#16161a` |
| head fill | `#0d0d12` |
| sunburst rays | `#1c1c21` |
| spot (default) | `#e63946` |
| rainbow, front→back | `#ff3b3b #ff8a1f #ffd400 #2ed573 #1e90ff #5b4bff #c04bff` |

**The spot colour drives everything that carries it:**
- the halftone disc behind the head (dots of radius 2.6 on an 8-unit grid,
  rotated 30°)
- the sticker's offset shadow (4, 5)
- the stripe on the board and skis
- the hatching on every shadow side of the scenery and on the beard (45°
  lines, spacing 6, width 1.4)
- the park rider's beanie, unless `beanie` is set. The beanie's own hatching
  stays ink `#16161a`, as in the reference, because spot hatching on a
  spot-coloured beanie would vanish.

**The sticker.** Every zine icon reads as one die-cut sticker:
- a paper outline about 6 units wide around the silhouette
- a spot-coloured hard shadow offset (4, 5)
- the board and skis carry their own outline and shadow, and spin with them

Draw it as geometry: a thick paper stroke under each shape, and a translated
spot-coloured copy beneath that. Whether the head's outline cuts over the
board where they overlap, or merges with it as in the mockups, is the
builder's call, as long as the result still reads as one sticker.

**Backgrounds.** All three are a black sheet with paper linework, spot
hatching, a spot halftone disc behind the head, and snow.
- **summit:**
  - a sunburst of 14 dim wedges (out of 28) radiating from just above the head
    centre, at (320, 190), and turning once every 120 s
  - five peaks: two big ones flanking the head, one behind it, two small ones
    in front. Seeded jitter around that composition, never a uniform random
    range.
  - each peak has a black fill, a paper outline on both slopes and no
    baseline. Its shadow face runs from the apex to the right foot and back to
    a point 18 % along the base, and is hatched.
  - a snowcap covers the top 27 % of each peak, with a jagged lower edge
  - a black ground band hides the peaks' feet
- **treeline:** no sunburst.
  - two rolling hill lines in paper
  - pines along them at seeded spacing. Each pine has three stacked triangle
    tiers with a black fill, a paper outline of 1.8 and a hatched right half.
  - the back row leaves the head zone clear
- **halfpipe:** the pipe seen from one end ("down the pipe").
  - a near U-section and a far U-section, each with quarter-ellipse
    transitions
  - the far section is wider than the head, so the walls and floor read as
    one continuous pipe underneath it
  - the left wall is hatched, the right wall is black, and the floor is spot
    halftone at 60 %
  - three ribs between the sections, plus lip, coping and deck lines
  - the same sunburst as summit, centred at (320, 150) instead
  - the seed varies only the snow
- **snow:**
  - `FLURRY`: paper squares (1.6–2.9 units, opacity 0.4–0.9) falling in an
    18 s loop
  - `BLIZZARD`: slanted paper streaks, about three times as dense, in a 6 s
    loop drifting left
  - `NO_SNOW`: none
  - Density is per unit of area. Count against file size, as the starfield
    does in `docs/mood/README.md`, and scale with resolution.

**Icons.** All three share the board or crossed skis, the spin, the head
motion and the tempo.
- **gear:**
  - **board:** a rounded rect 216×42, black, with a paper outline of 3, a spot
    stripe 192×6 and paper bindings 22×28 at x ∈ ±[64, 86]
  - **crossed skis:** two 216×26 twin-tips crossed at ±32°, each with a spot
    stripe, paper tip lines at ±88 and a central binding
- **spin:**
  - the duration of one turn is `60 / spin_rpm` seconds, with no animation at 0
  - `counter_clockwise` reverses the direction
- **head motion** applies to the head group only, never to the board:
  - `BOB` lifts 7 units on every beat
  - `SWAY` rocks ±8° about the neck over 2 beats
- **tempo:** one beat lasts `60 / bpm` seconds, and every head animation runs
  on it
- **`punk`**
  - **head:** three-quarter view, turned 35° to the viewer's right.
  - **sphere-wrap.** Features are drawn on the front-view disc (head r = 56 at
    (0, 8)) and wrapped onto the sphere:
    - `Z = √(r² − x² − (y−8)²)`
    - `x' = x·cos φ + Z·sin φ`
    - a feature is visible where `−x·sin φ + Z·cos φ > 0`
  - **eyes:** white crosses at (±21, 1), arm half-extent 7.5, stroke 6.5,
    round caps.
  - **banana grin:**
    - it sits between `U(x) = 20 + 6·(1 − (x/36)²)` and
      `L(x) = 20 + 24·(1 − (x/36)²)`
    - six teeth hang from `U` and five rise from `L`, interleaved, each
      reaching 78 % of the local gap
    - every tooth shrinks 20 % toward its own centroid, then is
      sphere-wrapped
  - **mohawk:** a fin in the head's mid-plane.
    - seven spikes at angles θ from 46° (forehead) to −94° (nape), measured
      from the crown
    - each base sits at radius r − 3 and spans θ ± 12.5°, with a half-thickness
      of 7
    - each tip sits at radius r + L, with L = [26, 44, 54, 52, 44, 34, 24],
      leaning 14° toward the forehead
    - each spike is the convex hull of its projected corners, split into a lit
      forehead half and a shade nape half (shade = colour darkened 28 %)
    - spikes whose base has negative depth go behind the head; the rest go in
      front, drawn back to front
  - **animations:**
    - **crest wave:** each spike rotates ±7° about its base, one beat per
      cycle, each phase-shifted by 0.13 beat
    - **rainbow flow:** the crest hue travels around once every 8 beats
    - **chatter:** the lower teeth drop 4 units every half beat
    - **eye pop:** the eyes scale to 1.45 for an instant every 4 beats
- **`park`** (park rider)
  - **look:** a front-facing face; a beanie in `beanie` (or spot), hatched,
    with a ribbed cuff; a paper pompom; rainbow mirror goggles; a black neck
    gaiter with a spot zig-zag
  - **animations:**
    - **pompom:** wobbles ±16° every beat
    - **lens flow:** the lens hue travels every 8 beats
    - **glint:** a paper bar crosses the lens every 4 beats, clipped to the
      lens
    - **breath:** three puffs rise from the gaiter every 2 beats
- **`local`** (mountain local)
  - **look:** front-facing; a rolled black cap with a spot band; black
    wraparounds with spot reflections; a big black beard ending in five
    pointed locks, hatched on its right half and clipped to the beard shape
    (the mockup's first version clipped the hatching to a box, which is a
    bug); frost crosses; a moustache
  - **animations:**
    - **beard wind:** the beard mass stays still and the five locks each swing
      ±9° about their tops, phase-shifted by 0.18 beat
    - **frost:** each frost cross twinkles, staggered
    - **glint:** as on the park rider, but on the wraparounds
    - **breath:** puffs from under the moustache

**Rainbow flow and lens flow** use `filter: hue-rotate()` in the mockups.
- That is a filter over animated content, so it falls under the anti-goal.
- Measure it first. If it costs frames, replace it with per-spike or per-stop
  fill keyframes that cycle the seven colours.
- Shipping it unmeasured is not acceptable.

### Page assembly

`root.svg` today hard-codes the blueprint layering:
- the canvas rect
- the lattice
- the rain
- the halo circle and the vignette rect
- the icon group, carrying `filter="url(#ink)"` and scale `u·0.34/200`

To support every combination:
- **Backgrounds provide two layers:** one under the rain and one over it.
  - trihex: canvas and lattice under; halo and vignette over, exactly as
    today
  - zine: everything under; snow over or under is the builder's choice
- **Icons provide their own group attributes:**
  - blueprint: `filter="url(#ink)"` and today's scale
  - zine: no filter, the zine scale and the head-centre offset
- **Each item contributes its own defs and CSS.**
  - In an all-blueprint scene the concatenation must equal today's bytes. In
    particular `defs.svg` and `style::css()` stay as they are, and
    `tests/data/{shell,style}.txt` stay green unchanged.
  - Zine rules are appended only when a zine item is present.
  - A blueprint icon on a zine background still needs `#ink`. A zine scene can
    emit its own defs freely, because it has no golden history.
- **The aria label** gains fixed phrases for each new item.
- **The output file name.**
  - `Scene::slug()` keeps its exact output for trihex scenes.
  - New backgrounds and icons extend the vocabulary.
  - The `trihex-` prefix in `render()` may become the background's name for
    zine backgrounds.

### Tests and corpus

- **Commit 1 — the schema move.** It touches:
  - `parameters.proto`
  - `params.rs` (`resolve`, `validate`, `valid_configs`, the tests)
  - `tests/configs.rs`, `tests/reject.rs`, `tests/pipeline.rs`
  - the golden config strings in code

  It does not touch the corpus. `the_corpus_is_unchanged` fails at this commit
  by design, reporting 42 *missing* and 42 *stale* configs and **zero
  changed** SVGs.
- **Commit 2 — test data only, as the user asked.** It regenerates the corpus
  and nothing else.
  - **The reviewable proof:** the set of directory names under `tests/golden/`
    is identical before and after. Directory names are the SVG hashes. Only
    the `_parameters.json` file inside each directory changes, from the flat
    shape to the `trihex` shape.
  - Its commit message states that, with the before and after directory
    listings matching.
- **Zine features land after these two,** and every new corpus entry they add
  goes in its own test-data commit too.
- **`valid_configs` grows on two levels:**
  - The cross product: 10 backgrounds (7 trihex + 3 zine) × 6 glyphs
    (hexatri ×2, ship, punk, park, local) × 2 overlays gives 120 configs, all
    zine settings at their defaults.
  - On top of that, one config per non-default zine setting, on a fixed zine
    base: spot, each snow value, crossed skis, counter-clockwise, `spin_rpm`
    0, bob, sway, a bpm, each toggle, eyes pop and beanie.
  - Never a setting in a scene where it has no effect, such as `zine.spot`
    with only blueprint items. The harness rejects two configs that render one
    picture ("an axis stopped changing the picture"), and that rejection is
    correct.
- **`tests/configs.rs` invariants extend to the new dispatch:**
  - zine CSS is present exactly when a zine item is
  - `url(#ink)` is present exactly when a blueprint icon is
  - every animated zine class has its resting state with animations off
- **Determinism:** a test in the spirit of
  `the_rain_lights_a_column_without_moving_anything` shows that the icon and
  every zine setting leave a zine background's geometry unchanged.
- **Purity:** `tests/purity.rs` walks `src/` recursively.
- **Coverage:** the `MOODS` coverage test described above.

### Docs

- **Move** `docs/mood/` to `docs/moods/blueprint/`.
- **Add** `docs/moods/zine/README.md`, written in the shape of the blueprint
  README with these rules:
  1. **Three inks and a rainbow.** The sheet, paper and one spot colour. The
     rainbow belongs to the character only (crest, lenses) and never to the
     ground.
  2. **Print, don't paint.** Shading is hatching or halftone, never a
     gradient. The rainbow lens is the one gradient.
  3. **Everything at the centre is a sticker,** with a paper die-cut and a
     spot shadow.
  4. **The halftone disc is the halo.** It sets the head apart by printing
     behind it, never by glowing.
  5. **Head motion has a beat; the world is ambient.** Head animations share
     one tempo. The sunburst (120 s) and the snow (18 s) stay slow.
- **Add** `docs/moods/zine/samples/`, with configs and SVGs, as blueprint has.
- **Write two ADRs:** moods as folders plus a catalogue, and `background`
  becoming a `oneof`.
- **Update CLAUDE.md:** moods, the new layout, and the pointer to
  `docs/moods/`.
- **Update the ROADMAP.**

## Left to the builder

- The exact file split, struct names and template boundaries inside `zine/`.
- Seeded jitter ranges for the summit peaks and the treeline pines, and snow
  density per area.
- How the sticker outline is drawn as geometry, and whether the head's outline
  merges with the board's.
- Snow over or under the rain.
- The output file prefix for zine backgrounds.
- Field numbers and enum scoping in the proto. The `Snow` value `NO_SNOW` is
  only there to avoid clashing with `Trihex.Image.NONE` at package scope; any
  clash-free name works.
- Whether rainbow and lens flow keep `hue-rotate`, decided by measurement.
- Whether `local` and `park` are the right JSON keys. Both are approved names
  in the conversation ("park rider", "mountain local"), and single words keep
  the studio's dotted paths simple.
