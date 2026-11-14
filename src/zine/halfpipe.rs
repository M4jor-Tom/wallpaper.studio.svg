//! Halfpipe: a snow halfpipe seen end-on, its far section wider than the head so the pipe runs on
//! under it as one piece, under a turning sunburst.
use std::f64::consts::FRAC_PI_2;

use askama::Template;

use crate::geom::{fmt, pts};
use crate::zine::print::{Frame, hatch, hatch_path, header, seg};
use crate::zine::style::{PAPER, SHEET};

type P = (f64, f64);

/// One cross-section of the pipe: a flat floor with a quarter-ellipse wall rising to each lip.
#[derive(Clone, Copy)]
struct Section {
    cx: f64,
    floor_y: f64,
    half_floor: f64,
    lip_dx: f64,
    lip_dy: f64,
}

impl Section {
    /// 15 points from the lip (`a = 0`) down to where the wall meets the floor (`a = π/2`).
    fn quarter(&self, side: f64) -> Vec<P> {
        (0..15)
            .map(|i| {
                let a = FRAC_PI_2 * i as f64 / 14.0;
                (
                    self.cx + side * (self.half_floor + self.lip_dx * a.cos()),
                    self.floor_y - self.lip_dy + self.lip_dy * a.sin(),
                )
            })
            .collect()
    }

    /// Lip, floor, lip: the left quarter then the right one reversed.
    fn u_shape(&self) -> Vec<P> {
        let mut u = self.quarter(-1.0);
        u.extend(self.quarter(1.0).into_iter().rev());
        u
    }

    fn lerp(&self, to: &Self, t: f64) -> Self {
        let l = |a: f64, b: f64| a + (b - a) * t;
        Self {
            cx: l(self.cx, to.cx),
            floor_y: l(self.floor_y, to.floor_y),
            half_floor: l(self.half_floor, to.half_floor),
            lip_dx: l(self.lip_dx, to.lip_dx),
            lip_dy: l(self.lip_dy, to.lip_dy),
        }
    }

    fn lip(&self, side: f64) -> P {
        (
            self.cx + side * (self.half_floor + self.lip_dx),
            self.floor_y - self.lip_dy,
        )
    }

    fn foot(&self, side: f64) -> P {
        (self.cx + side * self.half_floor, self.floor_y)
    }
}

/// Frame-anchored near section, head-anchored far one.
fn sections(f: &Frame) -> (Section, Section) {
    let s = f.s;
    let near_lip = f.w / 2.0 + 8.0 * s;
    let (dx, dy) = (118.0 * s, 168.0 * s);
    let near = Section {
        cx: f.cx,
        floor_y: f.h - 8.0 * s,
        half_floor: near_lip - dx,
        lip_dx: dx,
        lip_dy: dy,
    };
    let far = Section {
        cx: f.cx,
        floor_y: f.cy + 98.0 * s,
        half_floor: 98.0 * s,
        lip_dx: 62.0 * s,
        lip_dy: 84.0 * s,
    };
    (near, far)
}

struct Rib {
    points: String,
    width: String,
}

/// Layering is the template's: header (sheet, sunburst, disc), wall masks, walls, floor, ribs, lines.
#[derive(Template)]
#[template(path = "zine/halfpipe.svg")]
struct Halfpipe<'a> {
    header: String,
    sheet: &'static str,
    paper: &'static str,
    spot: &'a str,
    mask_l: String,
    mask_r: String,
    wall_l: String,
    wall_r: String,
    /// one path: a tall frame holds hundreds of strokes
    hatch: String,
    hatch_w: String,
    floor: String,
    ribs: Vec<Rib>,
    near_u: String,
    far_u: String,
    near_w: String,
    far_w: String,
    /// lip joins, foot joins and deck lines, each a pair of segments
    joins: Vec<(String, String, String, String)>,
    join_w: String,
    decks: Vec<(String, String, String, String)>,
    deck_w: String,
}

/// Sheet, sunburst, disc and pipe: the layer under the rain. Snow is separate (`zine::snow`).
pub fn halfpipe(f: &Frame, spot: &str) -> String {
    let s = f.s;
    let (near, far) = sections(f);
    let wall = |side: f64| {
        let mut p = near.quarter(side);
        p.extend(far.quarter(side).into_iter().rev());
        p
    };
    let (wall_l, wall_r) = (wall(-1.0), wall(1.0));
    let mask = |side: f64| {
        let (n, fl) = (near.lip(side), far.lip(side));
        pts(&[
            (if side < 0.0 { 0.0 } else { f.w }, n.1),
            n,
            fl,
            (if side < 0.0 { 0.0 } else { f.w }, fl.1 + 30.0 * s),
        ])
    };
    let floor = [
        near.foot(-1.0),
        near.foot(1.0),
        far.foot(1.0),
        far.foot(-1.0),
    ];
    let ribs = [(0.3, 1.5), (0.55, 1.2), (0.78, 0.9)]
        .iter()
        .map(|&(t, w)| Rib {
            points: pts(&near.lerp(&far, t).u_shape()),
            width: fmt(w * s),
        })
        .collect();
    let mut joins = Vec::new();
    let mut decks = Vec::new();
    for side in [-1.0, 1.0] {
        joins.push(seg(near.lip(side), far.lip(side)));
        joins.push(seg(near.foot(side), far.foot(side)));
        let (n, fl) = (near.lip(side), far.lip(side));
        decks.push(seg((n.0, n.1 - 7.0 * s), (fl.0, fl.1 - 3.5 * s)));
    }
    Halfpipe {
        header: header(f, Some((320.0, 150.0)), (320.0, 168.0), 132.0),
        sheet: SHEET,
        paper: PAPER,
        spot,
        mask_l: mask(-1.0),
        mask_r: mask(1.0),
        wall_l: pts(&wall_l),
        wall_r: pts(&wall_r),
        hatch: hatch_path(hatch(&wall_l, 5.5 * s)),
        hatch_w: fmt(1.3 * s),
        floor: pts(&floor),
        ribs,
        near_u: pts(&near.u_shape()),
        far_u: pts(&far.u_shape()),
        near_w: fmt(2.4 * s),
        far_w: fmt(1.8 * s),
        joins,
        join_w: fmt(2.0 * s),
        decks,
        deck_w: fmt(1.3 * s),
    }
    .render()
    .expect("writing to a String")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{Glyph, Ground, RESOLUTIONS, Scene, Snow};
    use crate::svg::build_svg;
    use crate::zine::style::SPOT;

    #[test]
    fn the_far_section_is_wider_than_the_head() {
        for (w, h) in [(1920, 1080), (1080, 1920)] {
            let f = Frame::new(w, h);
            let (_, far) = sections(&f);
            let apart = far.lip(1.0).0 - far.lip(-1.0).0;
            assert!(apart > 2.0 * 56.0 * 1.06 * f.s, "{w}x{h}: {apart}");
        }
    }

    #[test]
    fn the_floor_runs_under_the_head() {
        let f = Frame::new(1920, 1080);
        let (_, far) = sections(&f);
        assert!(far.floor_y > f.cy + 64.0 * 1.06 * f.s);
    }

    #[test]
    fn the_near_lips_meet_the_frame_edges() {
        for (w, h) in [(1920, 1080), (1080, 1920), (3440, 1440)] {
            let f = Frame::new(w, h);
            let (near, _) = sections(&f);
            assert!(near.lip(-1.0).0 <= 0.0 && near.lip(1.0).0 >= f.w, "{w}x{h}");
        }
    }

    #[test]
    fn the_quarter_matches_the_reference() {
        let q = Section {
            cx: 320.0,
            floor_y: 292.0,
            half_floor: 98.0,
            lip_dx: 62.0,
            lip_dy: 84.0,
        }
        .u_shape();
        assert_eq!(q.len(), 30);
        assert_eq!(pts(&q[..2]), "160,208 160.39,217.41");
    }

    #[test]
    fn extreme_frames_render_bounded() {
        let frames = RESOLUTIONS.iter().map(|(_, wh)| *wh);
        for (w, h) in frames.chain([(1, 1), (8000, 200)]) {
            for snow in [Snow::Flurry, Snow::Blizzard] {
                let scene = Scene {
                    seed: 0,
                    ground: Ground::Halfpipe { snow },
                    glyph: Glyph::Hexatri { rotate: true },
                    overlay: None,
                    spot: SPOT.to_string(),
                };
                let svg = build_svg(w, h, &scene);
                assert!(svg.len() < 1 << 20, "{w}x{h}: {} bytes", svg.len());
            }
        }
    }
}
