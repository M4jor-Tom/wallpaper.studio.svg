//! The blueprint mood: the trihexagonal lattice, the character rain and the drop-shadowed glyphs.
pub mod icon;
pub mod matrix;
pub mod style;
pub mod trihex;

use askama::Template;

use crate::geom::fmt;
use crate::params::{Ground, Scene, trihex::Image};
use style::PAL;
pub use style::css;

/// `bg` is `userSpaceOnUse` so any shape can paint canvas: `objectBoundingBox` would squeeze the ramp
/// into one hexagon, and a shut CLOSEOPEN blind would show as a patch.
#[derive(Template)]
#[template(path = "blueprint/defs.svg")]
struct Defs {
    w04: String,
    h: String,
    bg0: &'static str,
    bg1: &'static str,
    nebulae: Vec<(&'static str, &'static str)>,
}

/// The blueprint gradients and drop shadow, and a starfield's nebulae.
pub fn defs(scene: &Scene, w: u32, h: u32) -> String {
    let starfield = matches!(
        scene.ground,
        Ground::Trihex {
            image: Image::Starfield,
            ..
        }
    );
    let nebulae = if starfield {
        vec![("neba", "#6fb7d1"), ("nebb", "#77c9a6")]
    } else {
        Vec::new()
    };
    Defs {
        w04: fmt(w as f64 * 0.4),
        h: fmt(h as f64),
        bg0: PAL.bg.0.as_str(),
        bg1: PAL.bg.1.as_str(),
        nebulae,
    }
    .render()
    .expect("writing to a String")
}
