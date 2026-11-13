use std::fmt::Write as _;
use std::sync::LazyLock;

/// Light-theme palette: `a` = hexagons, `b` = triangles.
pub struct Palette {
    pub a: String,
    pub b: String,
    pub bg: (String, String),
    pub ink: String,
}

pub static PAL: LazyLock<Palette> = LazyLock::new(|| Palette {
    a: darken("#6fb7d1", 0.58),
    b: darken("#77c9a6", 0.58),
    bg: ("#eef3f6".into(), "#d9e3ea".into()),
    ink: darken("#6fb7d1", 0.70),
});

/// hexagon border baseline opacity (also the reduced-motion value)
pub const STROKE_O: f64 = 0.27;
/// triangle fill baseline opacity
pub const FILL_O: f64 = 0.38;

// STARFIELD. VOID stays blue-slate: true black reads as a hole from another design.
pub static VOID: LazyLock<String> = LazyLock::new(|| darken("#6fb7d1", 0.90));
/// share of eligible hexagons that become windows
pub const SPACE_FRAC: f64 = 0.08;
/// scattered per cell bbox; ~75% survive the hexagon clip
pub const SPACE_STARS: u32 = 24;
pub const SPACE_STROKE_O: f64 = 0.34;
// CLOSEOPEN: sparseness comes from the duty cycle (~14% showing, ~4% open), hence the long period.
/// per-cell shutter period, so cells never sync
pub const BLIND_S: (f64, f64) = (60.0, 90.0);
// Blind keyframes, % of cycle: leaves scale(1), open, still open, shut. The window derives its own from these.
pub const BLIND_KF: (i32, i32, i32, i32) = (44, 49, 53, 58);
// ASCII only (no font can be embedded), and no < > & " ' so glyphs never need escaping.
pub const MATRIX_GLYPHS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ*+-=/\\|:;.#$%@?!";
/// PAL['b'] at 0.70: the default stays in-palette
pub const MATRIX_COLOR: &str = "#395e53b3";
/// share of column slots carrying a stream
pub const MATRIX_FRAC: f64 = 0.34;
/// seconds for a head to walk one column
pub const MATRIX_S: (f64, f64) = (18.0, 34.0);
// One glyph's life, % of the walk: flare at 0, trail level by [0], out by [1].
pub const MATRIX_KF: (f64, f64) = (3.0, 29.0);
/// the drop from head to the glyph just behind it
pub const MATRIX_HEAD_STEP: f64 = 0.62;

/// Mix two `#rrggbb` colours. `round_ties_even`: Python's `round` is half-to-even.
fn mix(h1: &str, h2: &str, t: f64) -> String {
    let channel = |h: &str, i: usize| {
        u8::from_str_radix(&h[i..i + 2], 16).expect("palette literals are valid hex") as f64
    };
    let mut out = String::from("#");
    for k in 0..3 {
        let i = 1 + 2 * k;
        let v = channel(h1, i) + (channel(h2, i) - channel(h1, i)) * t;
        write!(out, "{:02x}", v.round_ties_even() as u8).expect("writing to a String");
    }
    out
}

pub fn darken(h: &str, t: f64) -> String {
    mix(h, "#0c1017", t)
}

pub fn hex_rgba(s: &str) -> Result<(String, f64), String> {
    let t = s.trim();
    let bad = || format!("bad colour '{s}': use #rrggbb or #rrggbbaa");
    let digits = t.strip_prefix('#').ok_or_else(bad)?;
    let is_hex = |c: &u8| c.is_ascii_hexdigit();
    if !(digits.len() == 6 || digits.len() == 8) || !digits.bytes().all(|c| is_hex(&c)) {
        return Err(bad());
    }
    let (rgb, a) = digits.split_at(6);
    let alpha = if a.is_empty() {
        0xff
    } else {
        u8::from_str_radix(a, 16).map_err(|_| bad())?
    };
    Ok((format!("#{}", rgb.to_lowercase()), alpha as f64 / 255.0))
}

pub fn css() -> String {
    let (k0, k1, k2, k3) = BLIND_KF;
    let (f0, f1) = MATRIX_KF;
    let (f0, f1) = (f0 as i64, f1 as i64);
    let mut out = String::new();
    out.push_str("@keyframes spin{to{transform:rotate(360deg)}}");
    out.push_str("@keyframes rspin{to{transform:rotate(-360deg)}}");
    out.push_str("@keyframes scan{0%,100%{stroke-opacity:.16}50%{stroke-opacity:.62}}");
    out.push_str("@keyframes wavef{0%,100%{fill-opacity:.12}50%{fill-opacity:.6}}");
    out.push_str(
        "@keyframes light{0%{stroke-opacity:.27;fill-opacity:0}7%{stroke-opacity:.85;fill-opacity:0}\
         16%{stroke-opacity:.85;fill-opacity:.42}26%{stroke-opacity:.27;fill-opacity:0}\
         100%{stroke-opacity:.27;fill-opacity:0}}",
    );
    out.push_str(
        "@keyframes lightb{0%{stroke-opacity:.34}8%{stroke-opacity:.9}\
         24%{stroke-opacity:.34}100%{stroke-opacity:.34}}",
    );
    // closing is opening played backwards, so one symmetric cycle covers both.
    write!(
        out,
        "@keyframes blind{{0%,{k0}%{{transform:scale(1)}}\
         {k1}%,{k2}%{{transform:scale(0)}}{k3}%,100%{{transform:scale(1)}}}}"
    )
    .expect("writing to a String");
    // ...and the window is off while its blind covers it (1% margin). Both spans come from BLIND_KF.
    write!(
        out,
        "@keyframes winvis{{0%,{k0m2}%{{display:none}}\
         {k0m1}%,{k3p1}%{{display:inline}}{k3p2}%,100%{{display:none}}}}",
        k0m2 = k0 - 2,
        k0m1 = k0 - 1,
        k3p1 = k3 + 1,
        k3p2 = k3 + 2,
    )
    .expect("writing to a String");
    out.push_str(
        ".spin{animation:spin 24s linear infinite;transform-box:fill-box;transform-origin:center}",
    );
    out.push_str(
        ".rspin{animation:rspin 24s linear infinite;transform-box:fill-box;transform-origin:center}",
    );
    out.push_str(".scan{animation:scan 5s ease-in-out infinite}");
    out.push_str(".wavef{animation:wavef 5s ease-in-out infinite}");
    out.push_str(".light{animation:light 9s ease-in-out infinite}");
    out.push_str(".lightb{animation:lightb 9s ease-in-out infinite}");
    // both rest *open*: under reduced motion a shut blind would hide the starfield
    out.push_str(
        ".blind{animation:blind 75s ease-in-out infinite;transform-box:fill-box;\
         transform-origin:center;transform:scale(0)}",
    );
    out.push_str(".win{animation:winvis 75s ease-in-out infinite;display:inline}");
    write!(
        out,
        "@keyframes rain{{0%{{fill-opacity:var(--o)}}{f0}%{{fill-opacity:var(--t)}}\
         {f1}%,100%{{fill-opacity:0}}}}"
    )
    .expect("writing to a String");
    out.push_str(".rain{animation:rain var(--d) linear infinite}");
    out.push_str("@media (prefers-reduced-motion:reduce){*{animation:none!important}}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_matches_the_corpus() {
        assert_eq!(PAL.a, "#365665");
        assert_eq!(PAL.b, "#395e53");
        assert_eq!(PAL.ink, "#2a424f");
        assert_eq!(*VOID, "#16212a");
    }

    // half-to-even: no palette colour lands on a tie, so only this test catches f64::round.
    #[test]
    fn mix_rounds_ties_to_even() {
        assert_eq!(mix("#000000", "#050505", 0.5), "#020202");
    }

    #[test]
    fn hex_rgba_splits_colour_and_alpha() {
        assert_eq!(hex_rgba("#8899AA").unwrap(), ("#8899aa".into(), 1.0));
        assert_eq!(hex_rgba(" #8899aa80 ").unwrap().1, 0x80 as f64 / 255.0);
        assert!(hex_rgba("395e53").is_err());
        assert!(hex_rgba("#395e5").is_err());
        assert!(hex_rgba("#395e53ccc").is_err());
    }

    #[test]
    fn css_matches_the_corpus() {
        let want = include_str!("../tests/data/style.txt");
        let rules = want
            .trim_end_matches('\n')
            .strip_prefix("<style>")
            .and_then(|s| s.strip_suffix("</style>"))
            .expect("the fixture wraps the rules in <style> tags");
        assert_eq!(css(), rules);
    }
}
