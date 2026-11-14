//! The park rider: a front-facing snowboarder in a beanie, rainbow mirror goggles and a neck gaiter.
use std::f64::consts::PI;

use askama::Template;

use crate::geom::{fmt, pts};
use crate::params::ParkLook;
use crate::zine::rider::Sticker;
use crate::zine::style::{INK, PAPER, RAINBOW};

/// A gradient stop: its place along the lens, its colour at rest, and its `--i` for the flow.
struct Stop {
    offset: String,
    colour: &'static str,
    style: String,
}

/// `paint` off is the silhouette: the same shapes, no fill, no face.
#[derive(Template)]
#[template(path = "zine/park.svg")]
struct Park<'a> {
    paint: bool,
    lens_flow: bool,
    glint: bool,
    breath: bool,
    paper: &'static str,
    ink: &'static str,
    spot: &'a str,
    beanie: &'a str,
    stops: &'a [Stop],
    pompom: &'a str,
    /// the beanie's 25 diagonal hatch lines: where each meets x = 0 and x = 56
    hatch: &'a [(String, String)],
    ribs: &'a [String],
    zigzag: &'a str,
}

/// The pompom: a 14-point star, 16 out and 12.8 in, hung over the crown at (0, -70).
fn pompom() -> String {
    let star: Vec<_> = (0..28)
        .map(|k| {
            let (s, c) = (k as f64 * PI / 14.0).sin_cos();
            let r = if k % 2 == 0 { 16.0 } else { 12.8 };
            (r * c, -70.0 + r * s)
        })
        .collect();
    pts(&star)
}

pub fn head(spot: &str, p: &ParkLook) -> Sticker {
    let stops: Vec<_> = RAINBOW
        .iter()
        .enumerate()
        .map(|(i, colour)| Stop {
            offset: fmt(i as f64 / 6.0),
            colour,
            style: format!("--i:{i}"),
        })
        .collect();
    let hatch: Vec<_> = (0..25)
        .map(|k| {
            let y = -120.0 + 5.0 * k as f64;
            (fmt(y), fmt(y + 56.0))
        })
        .collect();
    let ribs: Vec<_> = (0..13).map(|k| fmt(-50.0 + 8.0 * k as f64)).collect();
    // a zig-zag of 11 teeth across the gaiter, 8 wide and 6 tall
    let zigzag: String = (0..11).fold("M-44 58".to_string(), |d, k| {
        format!("{d} l8 {}", if k % 2 == 0 { -6 } else { 6 })
    });
    let pompom = pompom();
    let draw = |paint| {
        Park {
            paint,
            lens_flow: p.lens_flow,
            glint: p.glint,
            breath: p.breath,
            paper: PAPER,
            ink: INK,
            spot,
            beanie: &p.beanie,
            stops: &stops,
            pompom: &pompom,
            hatch: &hatch,
            ribs: &ribs,
            zigzag: &zigzag,
        }
        .render()
        .expect("writing to a String")
    };
    Sticker {
        silhouette: draw(false),
        art: draw(true),
        beats: p.pompom || p.lens_flow || p.glint || p.breath,
        // the gaiter's foot: its lower edge 68 -> 84 -> 68 bottoms out at 76
        neck: 76.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{Glyph, parse, resolve, validate};
    use crate::zine::style::{INK, SPOT};
    use serde_json::json;

    fn look(park: serde_json::Value) -> ParkLook {
        let cfg = json!({"icon": {"park": park}}).to_string();
        let p = parse(&cfg).unwrap();
        validate(&p).unwrap_or_else(|e| panic!("{cfg}: {e}"));
        let scene = resolve(&p);
        let Glyph::Park(k) = scene.glyph else {
            panic!("not a park rider")
        };
        k
    }

    fn art(park: serde_json::Value) -> String {
        head(SPOT, &look(park)).art
    }

    /// the beanie's dome, up to its closing tag
    fn dome(art: &str) -> &str {
        let i = art.find("<path d=\"M-54 2").expect("the beanie");
        &art[i..i + art[i..].find("/>").unwrap()]
    }

    #[test]
    fn the_beanie_follows_the_spot_unless_set() {
        assert_eq!(look(json!({})).beanie, SPOT);
        assert!(dome(&art(json!({}))).contains(r##"fill="#e63946""##));
        let cfg = json!({"zine": {"spot": "#22c8e5"}, "icon": {"park": {}}}).to_string();
        let Glyph::Park(k) = resolve(&parse(&cfg).unwrap()).glyph else {
            panic!()
        };
        assert_eq!(k.beanie, "#22c8e5", "a spot-coloured beanie");
        let set = look(json!({"beanie": "#00ffff"}));
        assert!(dome(&head("#e63946", &set).art).contains(r##"fill="#00ffff""##));
    }

    #[test]
    fn the_beanie_hatching_stays_ink() {
        let art = art(json!({"beanie": "#00ffff"}));
        let hatch = &art[art.find("<clipPath").unwrap()..];
        let g = &hatch[hatch.find("<g clip-path").unwrap()..];
        let g = &g[..g.find("</g></g>").unwrap()];
        assert!(g.contains(&format!(r#"stroke="{INK}""#)), "{g}");
        assert_eq!(g.matches("<line").count(), 25);
    }

    #[test]
    fn beanie_rejects_alpha_and_bad_hex() {
        for bad in ["#e6394680", "e63946", "#e6394", "red"] {
            let cfg = json!({"icon": {"park": {"beanie": bad}}}).to_string();
            let e = validate(&parse(&cfg).unwrap()).unwrap_err().to_string();
            assert!(e.starts_with("icon.park.beanie:"), "{bad}: {e}");
        }
    }

    #[test]
    fn a_typed_beanie_reaches_the_svg_normalised() {
        let art = art(json!({"beanie": " #22C8E5 "}));
        assert!(art.contains("#22c8e5") && !art.contains("22C8E5"));
    }

    #[test]
    fn glint_and_breath_exist_only_when_on() {
        let rest = art(json!({}));
        assert!(!rest.contains("zgl") && !rest.contains("zbreath"));
        assert!(art(json!({"glint": true})).contains(r#"class="zgl""#));
        assert!(art(json!({"breath": true})).contains(r#"class="zbreath""#));
        assert!(!art(json!({"glint": true})).contains("zbreath"));
    }

    #[test]
    fn glint_and_breath_rest_invisible() {
        let on = art(json!({"glint": true, "breath": true}));
        for class in ["zgl", "zbreath"] {
            let tag = &on[on.find(&format!(r#"class="{class}""#)).unwrap()..];
            let tag = &tag[..tag.find('>').unwrap()];
            assert!(tag.contains(r#"opacity="0""#), "{tag}");
        }
    }

    #[test]
    fn toggles_decide_what_beats() {
        assert!(!head(SPOT, &look(json!({}))).beats);
        for t in ["pompom", "lens_flow", "glint", "breath"] {
            assert!(head(SPOT, &look(json!({ t: true }))).beats, "{t}");
        }
    }

    #[test]
    fn the_silhouette_is_the_shape_without_paint() {
        let s = head(SPOT, &look(json!({"glint": true, "breath": true}))).silhouette;
        assert!(!s.contains("fill=") && !s.contains("stroke="), "{s}");
        assert!(s.contains(r#"class="zpom""#) && !s.contains("zgl"));
    }

    // the approved render printed one decimal
    #[test]
    fn the_pompom_matches_the_reference() {
        let reference = [
            16.0, -70.0, 12.5, -67.2, 14.4, -63.1, 10.0, -62.0, 10.0, -57.5,
        ];
        let ours = pompom();
        let ours: Vec<f64> = ours
            .split([' ', ','])
            .take(10)
            .map(|v| v.parse().unwrap())
            .collect();
        for (a, b) in ours.iter().zip(reference) {
            assert!((a - b).abs() < 0.06, "{ours:?}");
        }
    }

    #[test]
    fn every_animation_has_its_own_rules_and_rests_under_reduced_motion() {
        let css = |park: serde_json::Value| crate::zine::css(&Glyph::Park(look(park)));
        let none = css(json!({}));
        for (toggle, rule) in [
            ("pompom", "@keyframes zpom"),
            ("lens_flow", "@keyframes zlensflow"),
            ("glint", "@keyframes zglint"),
            ("breath", "@keyframes zpuff"),
        ] {
            assert!(!none.contains(rule), "{rule} without {toggle}");
            assert!(css(json!({ toggle: true })).contains(rule), "{toggle}");
        }
        assert!(!none.contains("filter") && !none.contains("hue-rotate"));
        let all = json!({"pompom": true, "lens_flow": true, "glint": true, "breath": true});
        let svg = art(all.clone());
        // the markup carries no transform and nothing visible that only an animation shows
        assert!(
            !svg.contains("transform=") && !svg.contains("filter"),
            "{svg}"
        );
        let css = css(all);
        assert!(
            css.ends_with("@media (prefers-reduced-motion:reduce){*{animation:none!important}}")
        );
        assert_eq!(
            css.matches("stop-color:#").count(),
            8,
            "one lap of the rainbow"
        );
    }
}
