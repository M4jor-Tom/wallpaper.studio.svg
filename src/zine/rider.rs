//! What every zine head shares: the gear behind it, its spin, and the die-cut sticker.
use askama::Template;

use crate::geom::fmt;
use crate::params::Rig;
use crate::params::rider::{Gear, Motion};
use crate::zine::style::{INK, PAPER};

/// A head as the sticker needs it: the shape to cut out, and the art printed on it.
pub struct Sticker {
    /// unpainted, so it takes the fill and stroke of whatever wraps it
    pub silhouette: String,
    pub art: String,
    /// some part of the head moves to the beat, so the rider must set one
    pub beats: bool,
}

/// Bottom to top: every shadow, every outline, then the art, so the outlines merge into one edge.
#[derive(Template)]
#[template(path = "zine/rider.svg")]
struct Rider<'a> {
    spot: &'a str,
    paper: &'static str,
    ink: &'static str,
    skis: bool,
    /// the spin groups' inline style; none when still, so the gear rests horizontal
    spin: Option<String>,
    /// the head groups' class; none when still, so the head rests centred
    motion: Option<&'static str>,
    /// `--beat` for every head animation; none when nothing moves to it
    beat: Option<String>,
    head: &'a Sticker,
}

/// The head over its gear, cut out as one sticker without a filter: a 12-wide paper stroke under
/// each shape is the edge, and a spot copy offset (4, 5) under that is the shadow. One beat,
/// `60000 / bpm` ms, times every head animation; the gear keeps its own rpm.
pub fn icon(spot: &str, rig: &Rig, head: &Sticker) -> String {
    let spin = (rig.rpm > 0).then(|| {
        let reverse = if rig.ccw {
            ";animation-direction:reverse"
        } else {
            ""
        };
        format!(
            "animation-duration:{}s{reverse}",
            fmt(60.0 / rig.rpm as f64)
        )
    });
    let motion = match rig.motion {
        Motion::Still => None,
        Motion::Bob => Some("zbob"),
        Motion::Sway => Some("zsway"),
    };
    // in ms, so every bpm is exact to fmt's two decimals; the unit travels into each calc()
    let beat = (motion.is_some() || head.beats)
        .then(|| format!("--beat:{}ms", fmt(60000.0 / rig.bpm as f64)));
    Rider {
        spot,
        paper: PAPER,
        ink: INK,
        skis: rig.gear == Gear::CrossedSkis,
        spin,
        motion,
        beat,
        head,
    }
    .render()
    .expect("writing to a String")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::rider::Motion;
    use crate::params::{Glyph, PunkLook, parse, resolve};
    use crate::zine::punk;
    use crate::zine::style::SPOT;
    use serde_json::json;

    fn still(rig: Rig) -> PunkLook {
        PunkLook {
            rig,
            crest_wave: false,
            rainbow_flow: false,
            chatter: false,
            eye_pop: false,
        }
    }

    fn draw(gear: Gear, ccw: bool, rpm: u32) -> String {
        let rig = Rig {
            gear,
            ccw,
            rpm,
            motion: Motion::Still,
            bpm: 96,
        };
        icon(SPOT, &rig, &punk::head(&still(rig)))
    }

    /// The punk a config resolves to.
    fn punk_of(punk: serde_json::Value) -> PunkLook {
        let cfg = json!({"icon": {"punk": punk}}).to_string();
        let Glyph::Punk(p) = resolve(&parse(&cfg).unwrap()).glyph else {
            panic!("not a punk")
        };
        p
    }

    fn rendered(p: &PunkLook) -> String {
        icon(SPOT, &p.rig, &punk::head(p))
    }

    /// The CSS with every `@keyframes` block cut out: what applies with animations off.
    fn at_rest(css: &str) -> String {
        let (mut out, mut rest) = (String::new(), css);
        while let Some(i) = rest.find("@keyframes") {
            out.push_str(&rest[..i]);
            let mut depth = 0;
            let end = rest[i..]
                .char_indices()
                .find_map(|(j, c)| {
                    depth += match c {
                        '{' => 1,
                        '}' => -1,
                        _ => return None,
                    };
                    (depth == 0).then_some(i + j + 1)
                })
                .expect("a closed @keyframes");
            rest = &rest[end..];
        }
        out + rest
    }

    fn transforms(svg: &str) -> Vec<&str> {
        svg.match_indices(r#"transform=""#)
            .map(|(i, m)| {
                let v = &svg[i + m.len()..];
                &v[..v.find('"').unwrap()]
            })
            .collect()
    }

    #[test]
    fn the_sticker_is_geometry_not_a_filter() {
        for gear in [Gear::Board, Gear::CrossedSkis] {
            let svg = draw(gear, false, 8);
            assert!(
                !svg.contains("<filter") && !svg.contains("filter="),
                "{svg}"
            );
            let paint = |c: &str| format!(r#"fill="{c}" stroke="{c}" stroke-width="12""#);
            assert!(svg.contains(&paint(SPOT)), "no spot shadow");
            assert!(svg.contains(&paint(PAPER)), "no paper outline");
        }
    }

    #[test]
    fn unset_spin_is_eight_turns_a_minute() {
        let svg = rendered(&punk_of(json!({})));
        assert_eq!(svg.matches("animation-duration:7.5s").count(), 3);
    }

    #[test]
    fn zero_rpm_rests_horizontal() {
        let board = draw(Gear::Board, true, 0);
        assert!(!board.contains("zspin") && !board.contains("animation"));
        // the spin groups carry nothing: the shadow offset is the board's one transform
        assert_eq!(transforms(&board), ["translate(4,5)"]);
        let skis = draw(Gear::CrossedSkis, false, 0);
        assert!(!skis.contains("zspin"));
        // each ski's ±32° is art, not spin
        for t in transforms(&skis) {
            assert!(
                ["translate(4,5)", "rotate(-32)", "rotate(32)"].contains(&t),
                "{t}"
            );
        }
    }

    #[test]
    fn counter_clockwise_reverses() {
        let ccw = draw(Gear::Board, true, 8);
        assert_eq!(
            ccw.matches("animation-duration:7.5s;animation-direction:reverse")
                .count(),
            3
        );
        assert!(!draw(Gear::Board, false, 8).contains("reverse"));
    }

    #[test]
    fn the_shadow_offset_sits_outside_the_spin() {
        let svg = draw(Gear::CrossedSkis, false, 20);
        assert!(
            svg.contains(
                r#"<g transform="translate(4,5)"><g class="zspin" style="animation-duration:3s">"#
            ),
            "{svg}"
        );
        // shadow, outline and art each turn, in step
        assert_eq!(svg.matches(r#"class="zspin""#).count(), 3);
    }

    #[test]
    fn the_beat_follows_bpm() {
        let svg = rendered(&punk_of(json!({"rider": {"motion": "BOB", "bpm": 120}})));
        assert_eq!(
            svg.matches("--beat:").count(),
            1,
            "one beat for the whole rider"
        );
        assert!(svg.contains("--beat:500ms"), "{svg}");
        let unset = rendered(&punk_of(json!({"rider": {"motion": "BOB"}})));
        // exact: seconds to two decimals would print 0.62s, a 96.8 bpm beat
        assert!(unset.contains("--beat:625ms"), "{unset}");
        // a head part dancing alone needs the beat too
        assert!(rendered(&punk_of(json!({"chatter": true}))).contains("--beat:"));
        assert!(!rendered(&punk_of(json!({"rider": {"bpm": 120}}))).contains("--beat"));
    }

    #[test]
    fn head_motion_moves_the_head_alone() {
        for (motion, class) in [("BOB", "zbob"), ("SWAY", "zsway")] {
            let p = punk_of(json!({"rider": {"motion": motion}}));
            let (svg, head) = (rendered(&p), punk::head(&p));
            let moved = |part: &str| format!(r#"<g class="{class}">{part}</g>"#);
            // shadow and outline cut the moving shape, so the sticker stays one piece
            assert_eq!(svg.matches(&moved(&head.silhouette)).count(), 2, "{svg}");
            assert_eq!(svg.matches(&moved(&head.art)).count(), 1, "{svg}");
            assert_eq!(svg.matches(class).count(), 3, "the gear moved: {svg}");
        }
        assert!(!rendered(&punk_of(json!({}))).contains("zbob"));
    }

    #[test]
    fn every_animation_on_still_rests_under_reduced_motion() {
        let p = punk_of(json!({
            "rider": {"motion": "BOB"},
            "crest_wave": true, "rainbow_flow": true, "chatter": true, "eyes": "POP",
        }));
        let svg = rendered(&p);
        // the head and the gear carry no transform of their own: animations off, they rest
        assert_eq!(transforms(&svg), ["translate(4,5)"]);
        assert!(!svg.contains("transform:"), "{svg}");
        let css = crate::zine::css(&Glyph::Punk(p));
        assert!(
            css.ends_with("@media (prefers-reduced-motion:reduce){*{animation:none!important}}"),
            "{css}"
        );
        let rest = at_rest(&css);
        for moved in ["transform:", "fill:", "opacity:", "filter:"] {
            assert!(!rest.contains(moved), "{moved} outside a keyframe: {rest}");
        }
    }
}
