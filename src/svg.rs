use askama::Template;

use crate::geom::{Lattice, fmt};
use crate::icon::{ico_hexatri, ico_ship};
use crate::matrix::pat_matrix;
use crate::params::{Glyph, Scene, background};
use crate::rng::PyRandom;
use crate::style::{PAL, css};
use crate::trihex::pat_trihex;

/// `bg` is `userSpaceOnUse` so any shape can paint canvas: `objectBoundingBox` would squeeze the ramp
/// into one hexagon, and a shut CLOSEOPEN blind would show as a patch.
#[derive(askama::Template)]
#[template(path = "defs.svg")]
struct Defs {
    w04: String,
    h: String,
    bg0: &'static str,
    bg1: &'static str,
    nebulae: Vec<(&'static str, &'static str)>,
}

/// `root.svg` ends in TWO newlines: askama trims one, and the corpus expects a trailing `\n`.
#[derive(askama::Template)]
#[template(path = "root.svg")]
struct Root {
    view_box: String,
    ws: String,
    hs: String,
    cx: String,
    cy: String,
    clear_r: String,
    icon_transform: String,
    label: String,
    defs: String,
    css: String,
    bg: String,
    rain: String,
    glyph: String,
}

pub fn build_svg(w: u32, h: u32, scene: &Scene) -> String {
    let mut rng = PyRandom::new(&format!("trihex:{}", scene.seed));
    let lat = Lattice::new(w, h);
    let (u, clear_r) = (lat.u, lat.clear_r);

    let nebulae = if scene.image == background::Image::Starfield {
        vec![("neba", "#6fb7d1"), ("nebb", "#77c9a6")]
    } else {
        Vec::new()
    };
    let defs = Defs {
        w04: fmt(w as f64 * 0.4),
        h: fmt(h as f64),
        bg0: PAL.bg.0.as_str(),
        bg1: PAL.bg.1.as_str(),
        nebulae,
    }
    .render()
    .expect("writing to a String");

    let bg_svg = pat_trihex(w, h, &lat, scene, &mut rng);
    // between the pattern and the halo, so the halo masks the rain around the icon
    let rain_svg = scene.overlay.as_ref().map_or(String::new(), |rain| {
        pat_matrix(w, h, &lat, scene.seed, rain.angle, &rain.color)
    });
    let k = u * 0.34 / 200.0;
    let glyph = match scene.glyph {
        Glyph::Ship => ico_ship(),
        Glyph::Hexatri { rotate } => ico_hexatri(rotate),
    };
    let (ws, hs) = (fmt(w as f64), fmt(h as f64));
    let (cx, cy) = (fmt(w as f64 / 2.0), fmt(h as f64 / 2.0));

    let ship = matches!(scene.glyph, Glyph::Ship);
    let mut label = if ship {
        "trihexagonal background with a spaceship icon"
    } else {
        "trihexagonal background"
    }
    .to_string();
    if scene.image == background::Image::Starfield {
        label.push_str(if scene.motion == background::Motion::Closeopen {
            ", some hexagons opening and closing onto a starfield"
        } else {
            ", some hexagons showing a starfield"
        });
    }
    if scene.overlay.is_some() {
        label.push_str(", with streams of characters drifting across it");
    }

    Root {
        view_box: format!("0 0 {ws} {hs}"),
        icon_transform: format!("translate({cx},{cy}) scale({})", fmt(k)),
        ws,
        hs,
        cx,
        cy,
        clear_r: fmt(clear_r),
        label,
        defs,
        css: css(),
        bg: bg_svg,
        rain: rain_svg,
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
