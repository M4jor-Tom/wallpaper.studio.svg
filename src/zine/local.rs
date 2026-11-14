//! The mountain local: a front-facing bearded rider in a rolled cap, black wraparounds and a big
//! black beard that ends in five pointed locks.
use askama::Template;

use crate::geom::fmt;
use crate::params::{Fx, LocalLook};
use crate::zine::rider::Sticker;
use crate::zine::style::{INK, PAPER};

/// the beard mass, shared by its fill, its outline's clip and its hatching's clip
const BEARD: &str = "M-48 14 Q -52 50 -38 68 L 38 68 Q 52 50 48 14 Q 40 34 24 32 Q 10 40 0 34 Q -10 40 -24 32 Q -40 34 -48 14 Z";
/// the wraparounds, which also clip the glint
const WRAPS: &str = "M-50 -8 Q 0 -16 50 -8 L 46 12 Q 24 18 6 8 L -6 8 Q -24 18 -46 12 Z";
/// the five locks, left to right
const LOCKS: [&str; 5] = [
    "M-42 58 Q -40 74 -26 86 Q -28 70 -18 58 Z",
    "M-27 58 Q -25 84 -11 96 Q -13 80 -3 58 Z",
    "M-12 58 Q -10 88 4 100 Q 2 84 12 58 Z",
    "M3 58 Q 5 84 19 96 Q 17 80 27 58 Z",
    "M18 58 Q 20 74 34 86 Q 32 70 42 58 Z",
];
/// the frost: six small crosses on the beard
const FROST: [&str; 6] = [
    "M-33 50h6M-30 47v6",
    "M-15 60h6M-12 57v6",
    "M5 52h6M8 49v6",
    "M23 62h6M26 59v6",
    "M31 46h6M34 43v6",
    "M-41 58h6M-38 55v6",
];

/// `paint` off is the silhouette: the same shapes, no fill, no face.
#[derive(Template)]
#[template(path = "zine/local.svg")]
struct Local<'a> {
    paint: bool,
    fx: Fx,
    paper: &'static str,
    ink: &'static str,
    spot: &'a str,
    beard: &'static str,
    wraps: &'static str,
    /// each lock's path and its `--i`
    locks: &'a [(&'static str, String)],
    /// each cross's path and its `--i`
    frost: &'a [(&'static str, String)],
    /// the cap band's 12 ribs
    ribs: &'a [String],
    /// the beard's 23 diagonal hatch lines: where each meets x = 0 and x = 56
    hatch: &'a [(String, String)],
}

/// Each path with its `--i:k`, the offset its animation is delayed by.
fn indexed(paths: &[&'static str]) -> Vec<(&'static str, String)> {
    paths
        .iter()
        .enumerate()
        .map(|(k, &d)| (d, format!("--i:{k}")))
        .collect()
}

pub fn head(spot: &str, l: &LocalLook) -> Sticker {
    let (locks, frost) = (indexed(&LOCKS), indexed(&FROST));
    let ribs: Vec<_> = (0..12).map(|k| fmt(-40.0 + 7.0 * k as f64)).collect();
    let hatch: Vec<_> = (0..23)
        .map(|k| {
            let y = -42.0 + 5.0 * k as f64;
            (fmt(y), fmt(y + 56.0))
        })
        .collect();
    let draw = |paint| {
        Local {
            paint,
            fx: l.fx,
            paper: PAPER,
            ink: INK,
            spot,
            beard: BEARD,
            wraps: WRAPS,
            locks: &locks,
            frost: &frost,
            ribs: &ribs,
            hatch: &hatch,
        }
        .render()
        .expect("writing to a String")
    };
    Sticker {
        silhouette: draw(false),
        art: draw(true),
        beats: l.beard_wind || l.frost || l.fx.glint || l.fx.breath,
        // the middle lock's tip, (4, 100)
        neck: 100.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{Glyph, glyph};
    use crate::zine::style::SPOT;
    use serde_json::json;

    fn look(local: serde_json::Value) -> LocalLook {
        let Glyph::Local(k) = glyph(json!({ "local": local })) else {
            panic!("not a local")
        };
        k
    }

    fn art(local: serde_json::Value) -> String {
        head(SPOT, &look(local)).art
    }

    #[test]
    fn the_beard_hatching_is_clipped_to_the_beard() {
        let art = art(json!({}));
        let clip = &art[art.find(r#"<clipPath id="zlbc">"#).unwrap()..];
        let clip = &clip[..clip.find("</clipPath>").unwrap()];
        assert_eq!(clip, format!(r#"<clipPath id="zlbc"><path d="{BEARD}"/>"#));
        assert!(!clip.contains("<rect"));
        let g = &art[art.find(r#"<g clip-path="url(#zlbc)">"#).unwrap()..];
        let g = &g[..g.find("</g></g>").unwrap()];
        assert!(g.contains(&format!(r#"stroke="{SPOT}""#)) && g.matches("<line").count() == 23);
    }

    #[test]
    fn the_beard_ends_in_five_locks() {
        let art = art(json!({}));
        assert_eq!(art.matches(r#"<g class="zlock" style="--i:"#).count(), 5);
        assert_eq!(art.matches(r#"<g class="zfrost" style="--i:"#).count(), 6);
    }

    #[test]
    fn wind_moves_only_the_locks() {
        let (off, on) = (json!({}), json!({"beard_wind": true}));
        // the markup is identical, so the beard mass itself cannot move
        assert_eq!(art(off.clone()), art(on.clone()));
        let css = |l| crate::zine::css(&Glyph::Local(look(l)));
        let (a, b) = (css(off), css(on));
        assert!(!a.contains("zlock") && b.contains("@keyframes zlock"));
        assert!(b.contains("rotate(-9deg)") && !b.contains("skew"));
    }

    #[test]
    fn glint_and_breath_exist_only_when_on_and_rest_invisible() {
        let rest = art(json!({}));
        assert!(!rest.contains("zgl") && !rest.contains("zbreath"));
        let on = art(json!({"glint": true, "breath": true}));
        for class in ["zgl", "zbreath"] {
            let tag = &on[on.find(&format!(r#"class="{class}""#)).unwrap()..];
            assert!(tag[..tag.find('>').unwrap()].contains(r#"opacity="0""#));
        }
        assert!(!on.contains("transform="));
    }

    #[test]
    fn toggles_decide_what_beats() {
        assert!(!head(SPOT, &look(json!({}))).beats);
        for t in ["beard_wind", "frost", "glint", "breath"] {
            assert!(head(SPOT, &look(json!({ t: true }))).beats, "{t}");
        }
    }

    #[test]
    fn every_animation_has_its_own_rules() {
        let css = |l: serde_json::Value| crate::zine::css(&Glyph::Local(look(l)));
        let none = css(json!({}));
        for (toggle, rule) in [
            ("beard_wind", "@keyframes zlock"),
            ("frost", "@keyframes zfrost"),
            ("glint", "@keyframes zglint"),
            ("breath", "@keyframes zpuff"),
        ] {
            assert!(!none.contains(rule), "{rule} without {toggle}");
            assert!(css(json!({ toggle: true })).contains(rule), "{toggle}");
        }
        assert!(css(json!({"frost": true})).contains("calc(var(--i)*var(--beat)*-.29)"));
        assert!(css(json!({"beard_wind": true})).contains("calc(var(--i)*var(--beat)*-.18)"));
    }

    #[test]
    fn the_silhouette_is_the_shape_without_paint() {
        let s = head(SPOT, &look(json!({"glint": true, "breath": true}))).silhouette;
        assert!(!s.contains("fill=") && !s.contains("stroke="), "{s}");
        assert!(s.contains(r#"class="zlock""#) && !s.contains("zgl"));
    }
}
