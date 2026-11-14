use askama::Template;

use crate::blueprint;
use crate::blueprint::icon::{ico_hexatri, ico_ship};
use crate::blueprint::matrix::pat_matrix;
use crate::blueprint::trihex::pat_trihex;
use crate::geom::{Lattice, fmt};
use crate::mood::Mood;
use crate::params::{Glyph, Ground, Rig, Scene, Snow, rider::Gear, trihex};
use crate::rng::PyRandom;
use crate::zine;
use crate::zine::halfpipe::halfpipe;
use crate::zine::print::Frame;
use crate::zine::summit::summit;
use crate::zine::treeline::treeline;
use crate::zine::{local, park, punk, rider};

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
    /// blueprint's drop shadow under the glyph
    ink: bool,
    label: String,
    defs: String,
    css: String,
    under: String,
    rain: String,
    over: String,
    glyph: String,
}

/// # Panics
///
/// On a zero width or height, where the lattice divides by zero and a zine ground's rows never
/// advance. `render_to_string` returns an error instead.
pub fn build_svg(w: u32, h: u32, scene: &Scene) -> String {
    assert!(
        w > 0 && h > 0,
        "build_svg needs a non-zero width and height, got {w}x{h}"
    );
    let lat = Lattice::new(w, h);
    let frame = Frame::new(w, h);
    let (ws, hs) = (fmt(w as f64), fmt(h as f64));
    let (cx, cy) = (fmt(w as f64 / 2.0), fmt(h as f64 / 2.0));

    // Each mood contributes its defs and CSS only when the scene uses it.
    let (mut defs, mut css) = (String::new(), String::new());
    for mood in Mood::ALL.into_iter().filter(|&m| scene.uses(m)) {
        let (d, c) = match mood {
            Mood::Blueprint => (blueprint::defs(scene, w, h), blueprint::css()),
            Mood::Zine => (zine::defs(&scene.spot, frame.s), zine::css(&scene.glyph)),
        };
        defs.push_str(&d);
        css.push_str(&c);
    }

    let (mut under, over) = match scene.ground {
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
        Ground::Summit { .. } => (summit(&frame, scene.seed, &scene.spot), String::new()),
        Ground::Treeline { .. } => (treeline(&frame, scene.seed, &scene.spot), String::new()),
        Ground::Halfpipe { .. } => (halfpipe(&frame, &scene.spot), String::new()),
    };
    if let Some(snow) = scene.ground.snow() {
        under.push_str(&zine::snow::snow(&frame, scene.seed, snow));
    }
    let rain = scene.overlay.as_ref().map_or(String::new(), |rain| {
        pat_matrix(w, h, &lat, scene.seed, rain.angle, &rain.color)
    });
    // blueprint glyphs are drop-shadowed and scaled to the lattice, whatever the ground
    let on_lattice = |glyph| {
        let at = format!("translate({cx},{cy}) scale({})", fmt(lat.u * 0.34 / 200.0));
        (glyph, at, true)
    };
    // zine glyphs are die-cut stickers drawn as geometry, scaled to the print and hung on the head centre
    let in_print = |rig, head| {
        let (x, y, k) = (frame.cx, frame.cy, 1.06 * frame.s);
        let at = format!("translate({},{}) scale({})", fmt(x), fmt(y), fmt(k));
        (rider::icon(&scene.spot, rig, &head), at, false)
    };
    let (glyph, icon_transform, ink) = match &scene.glyph {
        Glyph::Ship => on_lattice(ico_ship()),
        Glyph::Hexatri { rotate } => on_lattice(ico_hexatri(*rotate)),
        Glyph::Punk(p) => in_print(&p.rig, punk::head(p)),
        Glyph::Park(p) => in_print(&p.rig, park::head(&scene.spot, p)),
        Glyph::Local(l) => in_print(&l.rig, local::head(&scene.spot, l)),
    };

    let mut label = match scene.ground {
        Ground::Trihex { .. } => "trihexagonal background",
        Ground::Summit { .. } => "a black print of snowy peaks under a turning sunburst",
        Ground::Treeline { .. } => "a black print of a pine treeline under a halftone moon",
        Ground::Halfpipe { .. } => "a black print of a snow halfpipe under a turning sunburst",
    }
    .to_string();
    label.push_str(match scene.glyph {
        Glyph::Hexatri { .. } => "",
        Glyph::Ship => " with a spaceship icon",
        Glyph::Punk(_) => " with a punk head with a rainbow mohawk",
        Glyph::Park(_) => " with a snowboarder in a beanie and goggles",
        Glyph::Local(_) => " with a bearded mountain local",
    });
    if let Some(Rig { gear, rpm, .. }) = scene.glyph.rig() {
        label.push_str(match (gear, *rpm > 0) {
            (Gear::Board, true) => " over a spinning snowboard",
            (Gear::CrossedSkis, true) => " over spinning crossed skis",
            (Gear::Board, false) => " over a snowboard",
            (Gear::CrossedSkis, false) => " over crossed skis",
        });
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
        _ => match scene.ground.snow() {
            None | Some(Snow::NoSnow) => "",
            Some(Snow::Flurry) => ", with snow falling",
            Some(Snow::Blizzard) => ", in a blizzard",
        },
    });
    if scene.overlay.is_some() {
        label.push_str(", with streams of characters drifting across it");
    }

    Root {
        view_box: format!("0 0 {ws} {hs}"),
        ws,
        hs,
        icon_transform,
        ink,
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
    #[should_panic(expected = "a non-zero width and height")]
    fn a_zero_dimension_is_refused_not_drawn() {
        // at s = 0 a zine ground's rows never advance: release builds hang
        let halfpipe = resolve(&parse(r#"{"background":{"halfpipe":{}}}"#).unwrap());
        build_svg(0, 1080, &halfpipe);
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
