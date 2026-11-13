//! What every zine head shares: the gear behind it, its spin, and the die-cut sticker.
use askama::Template;

use crate::geom::fmt;
use crate::params::Rig;
use crate::params::rider::Gear;
use crate::zine::style::{INK, PAPER};

/// A head as the sticker needs it: the shape to cut out, and the art printed on it.
pub struct Sticker {
    /// unpainted, so it takes the fill and stroke of whatever wraps it
    pub silhouette: String,
    pub art: String,
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
    head: &'a Sticker,
}

/// The head over its gear, cut out as one sticker without a filter: a 12-wide paper stroke under
/// each shape is the edge, and a spot copy offset (4, 5) under that is the shadow.
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
    Rider {
        spot,
        paper: PAPER,
        ink: INK,
        skis: rig.gear == Gear::CrossedSkis,
        spin,
        head,
    }
    .render()
    .expect("writing to a String")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{Glyph, parse, resolve};
    use crate::zine::punk;
    use crate::zine::style::SPOT;

    fn draw(gear: Gear, ccw: bool, rpm: u32) -> String {
        icon(SPOT, &Rig { gear, ccw, rpm }, &punk::head())
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
        let Glyph::Punk(rig) = resolve(&parse(r#"{"icon":{"punk":{}}}"#).unwrap()).glyph else {
            panic!("not a punk")
        };
        let svg = icon(SPOT, &rig, &punk::head());
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
}
