//! Treeline: two rows of hatched pines on rolling hills, under a halftone moon.
use std::f64::consts::PI;

use askama::Template;

use crate::geom::{fmt, pts};
use crate::rng::PyRandom;
use crate::zine::print::{Frame, hatch, hatch_path, header};
use crate::zine::style::{PAPER, SHEET};

struct Pine {
    x: f64,
    /// where the trunk meets the hill
    base: f64,
    height: f64,
}

/// The hill's `y` at `x`: a sine of one fixed wavelength per frame width, whatever the aspect.
fn hill_y(f: &Frame, x: f64, base: f64, amp: f64, phase: f64) -> f64 {
    base - amp * (x / f.w * 1.4 * PI + phase).sin()
}

/// The back hill and the front hill, each `(base, amp, phase)`: one source for the hills drawn and
/// the hills the pines stand on, so no pine floats off its hill.
fn hills(f: &Frame) -> [(f64, f64, f64); 2] {
    [
        (f.h - 110.0 * f.s, 26.0 * f.s, 0.3),
        (f.h - 48.0 * f.s, 18.0 * f.s, 1.6),
    ]
}

/// The hill as a closed polygon, from 20 units beyond each side down to below the frame.
fn hill(f: &Frame, base: f64, amp: f64, phase: f64) -> Vec<(f64, f64)> {
    let (s, bottom) = (f.s, f.h + 20.0 * f.s);
    let n = ((f.w + 40.0 * s) / (20.0 * s)).ceil() as usize;
    let top = (0..=n).map(|i| {
        let x = (-20.0 * s + 20.0 * s * i as f64).min(f.w + 20.0 * s);
        (x, hill_y(f, x, base, amp, phase))
    });
    let mut p = vec![(-20.0 * s, bottom)];
    p.extend(top);
    p.push((f.w + 20.0 * s, bottom));
    p
}

/// Back row then front row, each left to right. Per pine: its height, then the step to the next.
/// The back row still draws a pine's height when it skips it, so the head never moves its neighbours.
fn pines(f: &Frame, seed: u32) -> [Vec<Pine>; 2] {
    let s = f.s;
    let mut rng = PyRandom::new(&format!("treeline:{seed}"));
    let mut row = |hill: (f64, f64, f64), step: (f64, f64), tall: (f64, f64), clear: bool| {
        let mut out = Vec::new();
        let mut x = rng.uniform(-10.0, 30.0) * s;
        while x < f.w + 30.0 * s {
            let height = rng.uniform(tall.0, tall.1) * s;
            if !clear || (x - f.cx).abs() >= 70.0 * s {
                let base = hill_y(f, x, hill.0, hill.1, hill.2) + 4.0 * s;
                out.push(Pine { x, base, height });
            }
            x += rng.uniform(step.0, step.1) * s;
        }
        out
    };
    let [back_hill, front_hill] = hills(f);
    let back = row(back_hill, (26.0, 58.0), (34.0, 52.0), true);
    let front = row(front_hill, (46.0, 90.0), (48.0, 70.0), false);
    [back, front]
}

struct Tier {
    id: String,
    tri: String,
    face: String,
    hatch: String,
}

struct PineSvg {
    trunk: (String, String, String, String),
    tiers: Vec<Tier>,
}

fn draw(row: usize, i: usize, p: &Pine, s: f64) -> PineSvg {
    let h = p.height;
    let tip = p.base - h;
    let tiers = (0..3)
        .map(|k| {
            let k_f = k as f64;
            // three equal-height triangles, each starting a quarter of the way down the last and wider
            let (top, th) = (tip + 0.256 * h * k_f, 0.412 * h);
            let (bottom, hw) = (top + th, th * (0.62 + 0.19 * k_f));
            // the right half is the shaded side
            let face = [(p.x, top), (p.x + hw, bottom), (p.x, bottom)];
            Tier {
                id: format!("zt{row}_{i}_{k}"),
                tri: pts(&[(p.x, top), (p.x + hw, bottom), (p.x - hw, bottom)]),
                face: pts(&face),
                hatch: hatch_path(
                    hatch(&face, 4.5 * s)
                        .into_iter()
                        // `hatch` covers the bounding box; keep the lines that meet the triangle
                        .filter(|[(_, y), _]| (top..=bottom).contains(y)),
                ),
            }
        })
        .collect();
    PineSvg {
        trunk: (
            fmt(p.x - 2.5 * s),
            fmt(p.base - 0.1 * h),
            fmt(5.0 * s),
            fmt(0.1 * h),
        ),
        tiers,
    }
}

/// Layering is the template's: header (sheet, moon), back hill, back pines, front hill, front pines.
#[derive(Template)]
#[template(path = "zine/treeline.svg")]
struct Treeline<'a> {
    header: String,
    sheet: &'static str,
    paper: &'static str,
    spot: &'a str,
    back_hill: String,
    back_ridge: String,
    back: Vec<PineSvg>,
    front_hill: String,
    front_ridge: String,
    ridge_w: String,
    front: Vec<PineSvg>,
    line_w: String,
    hatch_w: String,
}

/// Sheet, moon, hills and pines: the layer under the rain. Snow is separate (`zine::snow`).
pub fn treeline(f: &Frame, seed: u32, spot: &str) -> String {
    let s = f.s;
    let [back, front] = pines(f, seed);
    let [back_hill, front_hill] = hills(f).map(|(base, amp, phase)| hill(f, base, amp, phase));
    let rows = |row: usize, ps: &[Pine]| {
        ps.iter()
            .enumerate()
            .map(|(i, p)| draw(row, i, p, s))
            .collect()
    };
    Treeline {
        // a moon, not a sunburst
        header: header(f, None, (320.0, 170.0), 132.0),
        sheet: SHEET,
        paper: PAPER,
        spot,
        back_hill: pts(&back_hill),
        back_ridge: pts(&back_hill[1..back_hill.len() - 1]),
        back: rows(0, &back),
        front_hill: pts(&front_hill),
        front_ridge: pts(&front_hill[1..front_hill.len() - 1]),
        ridge_w: fmt(2.2 * s),
        front: rows(1, &front),
        line_w: fmt(1.8 * s),
        hatch_w: fmt(1.1 * s),
    }
    .render()
    .expect("writing to a String")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{Glyph, Ground, RESOLUTIONS, Rain, Scene, Snow, parse, resolve};
    use crate::svg::build_svg;
    use crate::zine::style::SPOT;

    fn scene(snow: Snow, spot: &str, glyph: Glyph) -> Scene {
        Scene {
            seed: 0,
            ground: Ground::Treeline { snow },
            glyph,
            overlay: None,
            spot: spot.to_string(),
        }
    }

    #[test]
    fn no_back_row_pine_stands_before_the_head() {
        for (w, h) in [(1920, 1080), (1080, 1920), (3440, 1440)] {
            let f = Frame::new(w, h);
            for seed in 0..50 {
                let [back, front] = pines(&f, seed);
                assert!(!back.is_empty() && !front.is_empty());
                for p in &back {
                    assert!(
                        (p.x - f.cx).abs() >= 70.0 * f.s,
                        "{w}x{h} seed {seed}: a pine at x={}",
                        p.x
                    );
                }
            }
        }
    }

    #[test]
    fn the_hills_span_an_ultrawide_frame() {
        let f = Frame::new(3440, 1440);
        for (base, amp, phase) in hills(&f) {
            let h = hill(&f, base, amp, phase);
            assert!(h[1].0 < 0.0 && h[h.len() - 2].0 > f.w);
        }
        // and the pines reach both edges
        let [back, _] = pines(&f, 0);
        assert!(back.first().unwrap().x < 0.1 * f.w && back.last().unwrap().x > 0.9 * f.w);
    }

    #[test]
    fn settings_and_icons_never_move_a_pine() {
        // everything before the icon group, the first group with a transform (riders nest more):
        // the treeline layer, the snow over it and the rain
        let points = |s: &Scene| -> Vec<String> {
            let svg = build_svg(1920, 1080, s);
            let page = &svg[..svg.find("<g transform=").expect("the icon group")];
            page.split("points=\"")
                .skip(1)
                .map(|p| p[..p.find('"').expect("a closing quote")].to_string())
                .collect()
        };
        let want = points(&scene(Snow::Flurry, SPOT, Glyph::Hexatri { rotate: true }));
        assert!(want.len() > 5 * 4, "the pines are missing");
        let glyphs = [
            r#"{"hexatri":{"motion":"ROTATE"}}"#,
            r#"{"hexatri":{"motion":"STATIC"}}"#,
            r#"{"ship":{}}"#,
            r#"{"punk":{}}"#,
            r#"{"park":{}}"#,
            r#"{"local":{}}"#,
            r#"{"punk":{"rider":{"gear":"CROSSED_SKIS","motion":"SWAY"}}}"#,
        ]
        .map(|icon| resolve(&parse(&format!(r#"{{"icon":{icon}}}"#)).unwrap()).glyph);
        let rain = Rain {
            angle: 250.0,
            color: "#395e53cc".to_string(),
        };
        for snow in [Snow::Flurry, Snow::NoSnow, Snow::Blizzard] {
            for spot in [SPOT, "#22c8e5"] {
                for glyph in &glyphs {
                    for overlay in [None, Some(rain.clone())] {
                        let s = Scene {
                            overlay,
                            ..scene(snow, spot, glyph.clone())
                        };
                        assert_eq!(points(&s), want, "{s:?} moved a pine");
                    }
                }
            }
        }
    }

    #[test]
    fn extreme_frames_render_bounded() {
        let frames = RESOLUTIONS.iter().map(|(_, wh)| *wh);
        for (w, h) in frames.chain([(1, 1), (8000, 200)]) {
            for snow in [Snow::Flurry, Snow::Blizzard] {
                let svg = build_svg(w, h, &scene(snow, SPOT, Glyph::Hexatri { rotate: true }));
                assert!(svg.len() < 1 << 20, "{w}x{h}: {} bytes", svg.len());
            }
        }
    }
}
