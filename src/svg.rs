use askama::Template;

use crate::blueprint;
use crate::blueprint::icon::{ico_hexatri, ico_ship};
use crate::blueprint::matrix::pat_matrix;
use crate::blueprint::style::PAL;
use crate::blueprint::trihex::pat_trihex;
use crate::geom::{Lattice, fmt};
use crate::mood::Mood;
use crate::params::{Glyph, Ground, Scene, Snow, trihex};
use crate::rng::PyRandom;
use crate::zine;
use crate::zine::print::Frame;
use crate::zine::summit::summit;
use crate::zine::treeline::treeline;

/// `bg` is `userSpaceOnUse` so any shape can paint canvas: `objectBoundingBox` would squeeze the ramp
/// into one hexagon, and a shut CLOSEOPEN blind would show as a patch.
#[derive(askama::Template)]
#[template(path = "blueprint/defs.svg")]
struct Defs {
    w04: String,
    h: String,
    bg0: &'static str,
    bg1: &'static str,
    nebulae: Vec<(&'static str, &'static str)>,
}

/// Canvas and lattice: the trihex layer under the rain.
#[derive(askama::Template)]
#[template(path = "blueprint/under.svg")]
struct Under {
    ws: String,
    hs: String,
    lattice: String,
}

/// Halo and vignette: the trihex layer over the rain, so the halo masks it around the icon.
#[derive(askama::Template)]
#[template(path = "blueprint/over.svg")]
struct Over {
    ws: String,
    hs: String,
    cx: String,
    cy: String,
    clear_r: String,
}

/// `root.svg` ends in TWO newlines: askama trims one, and the corpus expects a trailing `\n`.
#[derive(askama::Template)]
#[template(path = "root.svg")]
struct Root {
    view_box: String,
    ws: String,
    hs: String,
    icon_transform: String,
    icon_filter: Option<&'static str>,
    label: String,
    defs: String,
    css: String,
    under: String,
    rain: String,
    over: String,
    glyph: String,
}

pub fn build_svg(w: u32, h: u32, scene: &Scene) -> String {
    let lat = Lattice::new(w, h);
    let frame = Frame::new(w, h);
    let (ws, hs) = (fmt(w as f64), fmt(h as f64));
    let (cx, cy) = (fmt(w as f64 / 2.0), fmt(h as f64 / 2.0));

    // Each mood contributes its defs and CSS only when the scene uses it.
    let (mut defs, mut css) = (String::new(), String::new());
    if scene.uses(Mood::Blueprint) {
        let starfield = matches!(
            scene.ground,
            Ground::Trihex {
                image: trihex::Image::Starfield,
                ..
            }
        );
        let nebulae = if starfield {
            vec![("neba", "#6fb7d1"), ("nebb", "#77c9a6")]
        } else {
            Vec::new()
        };
        defs = Defs {
            w04: fmt(w as f64 * 0.4),
            h: fmt(h as f64),
            bg0: PAL.bg.0.as_str(),
            bg1: PAL.bg.1.as_str(),
            nebulae,
        }
        .render()
        .expect("writing to a String");
        css = blueprint::style::css();
    }
    if scene.uses(Mood::Zine) {
        defs.push_str(&zine::defs(&scene.spot, frame.s));
        css.push_str(&zine::css());
    }

    let (under, over) = match scene.ground {
        Ground::Trihex { motion, image } => {
            let mut rng = PyRandom::new(&format!("trihex:{}", scene.seed));
            (
                Under {
                    ws: ws.clone(),
                    hs: hs.clone(),
                    lattice: pat_trihex(w, h, &lat, scene.seed, motion, image, &mut rng),
                }
                .render()
                .expect("writing to a String"),
                Over {
                    ws: ws.clone(),
                    hs: hs.clone(),
                    cx: cx.clone(),
                    cy: cy.clone(),
                    clear_r: fmt(lat.clear_r),
                }
                .render()
                .expect("writing to a String"),
            )
        }
        Ground::Summit { snow } => (
            summit(&frame, scene.seed, &scene.spot) + &zine::snow::snow(&frame, scene.seed, snow),
            String::new(),
        ),
        Ground::Treeline { snow } => (
            treeline(&frame, scene.seed, &scene.spot) + &zine::snow::snow(&frame, scene.seed, snow),
            String::new(),
        ),
    };
    let rain = scene.overlay.as_ref().map_or(String::new(), |rain| {
        pat_matrix(w, h, &lat, scene.seed, rain.angle, &rain.color)
    });
    // blueprint glyphs are drop-shadowed and scaled to the lattice, whatever the ground
    let on_lattice = format!("translate({cx},{cy}) scale({})", fmt(lat.u * 0.34 / 200.0));
    let (glyph, icon_transform, icon_filter) = match scene.glyph {
        Glyph::Ship => (ico_ship(), on_lattice, Some("url(#ink)")),
        Glyph::Hexatri { rotate } => (ico_hexatri(rotate), on_lattice, Some("url(#ink)")),
    };

    let mut label = match scene.ground {
        Ground::Trihex { .. } => "trihexagonal background",
        Ground::Summit { .. } => "a black print of snowy peaks under a turning sunburst",
        Ground::Treeline { .. } => "a black print of a pine treeline under a halftone moon",
    }
    .to_string();
    if scene.glyph == Glyph::Ship {
        label.push_str(" with a spaceship icon");
    }
    label.push_str(match scene.ground {
        Ground::Trihex {
            motion: trihex::Motion::Closeopen,
            image: trihex::Image::Starfield,
        } => ", some hexagons opening and closing onto a starfield",
        Ground::Trihex {
            image: trihex::Image::Starfield,
            ..
        } => ", some hexagons showing a starfield",
        Ground::Trihex { .. }
        | Ground::Summit { snow: Snow::NoSnow }
        | Ground::Treeline { snow: Snow::NoSnow } => "",
        Ground::Summit { snow: Snow::Flurry } | Ground::Treeline { snow: Snow::Flurry } => {
            ", with snow falling"
        }
        Ground::Summit {
            snow: Snow::Blizzard,
        }
        | Ground::Treeline {
            snow: Snow::Blizzard,
        } => ", in a blizzard",
    });
    if scene.overlay.is_some() {
        label.push_str(", with streams of characters drifting across it");
    }

    Root {
        view_box: format!("0 0 {ws} {hs}"),
        ws,
        hs,
        icon_transform,
        icon_filter,
        label,
        defs,
        css,
        under,
        rain,
        over,
        glyph,
    }
    .render()
    .expect("writing to a String")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{parse, resolve};

    #[test]
    fn a_blueprint_scene_uses_only_blueprint() {
        let s = resolve(&parse("{}").unwrap());
        assert!(s.uses(Mood::Blueprint) && !s.uses(Mood::Zine));
    }

    #[test]
    fn the_document_shell_matches_the_corpus() {
        let want = include_str!("../tests/data/shell.txt");
        let got = build_svg(1920, 1080, &resolve(&parse("{}").unwrap()));
        assert!(
            got.starts_with(want),
            "shell differs:\n{}",
            &got[..want.len().min(got.len())]
        );
    }
}
