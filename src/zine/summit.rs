//! Summit: snowcapped peaks under a turning sunburst, with a halftone disc behind the head.
use askama::Template;

use crate::geom::{fmt, pts};
use crate::rng::PyRandom;
use crate::zine::print::{Frame, hatch, sunburst};
use crate::zine::style::{PAPER, RAYS, SHEET};

struct Peak {
    apex: (f64, f64),
    hw: f64,
    base: f64,
}

/// Row centres left to right, `cx ± (d + step·k)`, while a peak there can still reach into the frame.
fn row(f: &Frame, d: f64, step: f64) -> Vec<f64> {
    let reach = f.w / 2.0 + 200.0 * f.s;
    let offsets: Vec<f64> = (0..)
        .map(|k| d + step * k as f64)
        .take_while(|o| *o < reach)
        .collect();
    let left = offsets.iter().rev().map(|o| f.cx - o);
    left.chain(offsets.iter().map(|o| f.cx + o)).collect()
}

/// The back row, the peak behind the head, the front row: drawing order, each left to right.
/// A wide frame adds peaks rather than stretching them; a narrow one pulls the flanking pair in.
fn peaks(f: &Frame, seed: u32) -> [Vec<Peak>; 3] {
    let s = f.s;
    let mut rng = PyRandom::new(&format!("summit:{seed}"));
    // seeded jitter around a fixed composition: three draws per peak, in this order
    let mut peak = |x: f64, height: f64, hw: f64, base: f64| {
        let x = x + rng.uniform(-12.0, 12.0) * s;
        let height = height * rng.uniform(0.92, 1.08);
        let hw = hw * rng.uniform(0.92, 1.08);
        Peak {
            apex: (x, base - height),
            hw,
            base,
        }
    };
    let back_base = f.h - 15.0 * s;
    let back = row(f, (185.0 * s).min(0.32 * f.w), 375.0 * s)
        .into_iter()
        .map(|x| peak(x, 267.0 * s, 220.0 * s, back_base))
        .collect();
    let behind = vec![peak(f.cx + 10.0 * s, 205.0 * s, 140.0 * s, back_base)];
    let front = row(f, 270.0 * s, 540.0 * s)
        .into_iter()
        .map(|x| peak(x, 150.0 * s, 125.0 * s, f.h + 2.0 * s))
        .collect();
    [back, behind, front]
}

struct PeakSvg {
    id: String,
    tri: String,
    face: String,
    hatch: Vec<(String, String, String, String)>,
    cap: String,
}

fn draw(i: usize, p: &Peak, s: f64) -> PeakSvg {
    let (ax, ay) = p.apex;
    let right = (ax + p.hw, p.base);
    // the shadow side: apex to the right foot and back to a point 18% along the base
    let face = [p.apex, right, (ax + 0.18 * p.hw, p.base)];
    // the snowcap covers the top 27%; its lower edge zig-zags as in the reference
    let (cw, ly) = (0.27 * p.hw, ay + 0.27 * (p.base - ay));
    let cap = [
        (ax - cw, ly),
        p.apex,
        (ax + cw, ly),
        (ax + 0.64 * cw, ly - 9.0 * s),
        (ax + 0.2 * cw, ly + 0.1 * (ly - ay)),
        (ax - 0.4 * cw, ly - 7.0 * s),
    ];
    PeakSvg {
        id: format!("zh{i}"),
        tri: pts(&[(ax - p.hw, p.base), p.apex, right]),
        face: pts(&face),
        hatch: hatch(&face, 6.0 * s)
            .into_iter()
            .map(|[(x1, y1), (x2, y2)]| (fmt(x1), fmt(y1), fmt(x2), fmt(y2)))
            .collect(),
        cap: pts(&cap),
    }
}

/// Layering is the template's: sheet, sunburst, disc, back peaks, ground band, front peaks.
#[derive(Template)]
#[template(path = "zine/summit.svg")]
struct Summit<'a> {
    ws: String,
    hs: String,
    sheet: &'static str,
    paper: &'static str,
    rays: &'static str,
    spot: &'a str,
    origin: String,
    wedges: Vec<String>,
    disc: (String, String, String),
    back: Vec<PeakSvg>,
    band_y: String,
    band_h: String,
    front: Vec<PeakSvg>,
    line_w: String,
    hatch_w: String,
}

/// Sheet, sunburst, disc and peaks: the layer under the rain. Snow is separate (`zine::snow`).
pub fn summit(f: &Frame, seed: u32, spot: &str) -> String {
    let s = f.s;
    let [mut back, behind, front] = peaks(f, seed);
    back.extend(behind);
    let n = back.len();
    let back = back
        .iter()
        .enumerate()
        .map(|(i, p)| draw(i, p, s))
        .collect();
    let front = front
        .iter()
        .enumerate()
        .map(|(i, p)| draw(n + i, p, s))
        .collect();
    let rays = f.at(320.0, 190.0);
    let disc = f.at(320.0, 180.0);
    Summit {
        ws: fmt(f.w),
        hs: fmt(f.h),
        sheet: SHEET,
        paper: PAPER,
        rays: RAYS,
        spot,
        origin: format!("transform-origin:{}px {}px", fmt(rays.0), fmt(rays.1)),
        // the diagonal outreaches every corner, however the wedges turn
        wedges: sunburst(rays, f.w.hypot(f.h)),
        disc: (fmt(disc.0), fmt(disc.1), fmt(138.0 * s)),
        back,
        // hides the back row's feet
        band_y: fmt(f.h - 17.0 * s),
        band_h: fmt(17.0 * s),
        front,
        line_w: fmt(2.4 * s),
        hatch_w: fmt(1.4 * s),
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

    fn scene(snow: Snow, spot: &str, glyph: Glyph) -> Scene {
        Scene {
            seed: 0,
            ground: Ground::Summit { snow },
            glyph,
            overlay: None,
            spot: spot.to_string(),
        }
    }

    #[test]
    fn settings_and_icons_never_move_a_peak() {
        // everything before the icon group: the summit layer and the snow over it
        let points = |s: &Scene| -> Vec<String> {
            let svg = build_svg(1920, 1080, s);
            let page = &svg[..svg.rfind("<g transform=").expect("the icon group")];
            page.split("points=\"")
                .skip(1)
                .map(|p| p[..p.find('"').expect("a closing quote")].to_string())
                .collect()
        };
        let want = points(&scene(Snow::Flurry, SPOT, Glyph::Hexatri { rotate: true }));
        assert!(want.len() > 5 * 4, "the peaks are missing");
        for snow in [Snow::Flurry, Snow::NoSnow, Snow::Blizzard] {
            for spot in [SPOT, "#22c8e5"] {
                for glyph in [
                    Glyph::Hexatri { rotate: true },
                    Glyph::Hexatri { rotate: false },
                    Glyph::Ship,
                ] {
                    let s = scene(snow, spot, glyph);
                    assert_eq!(points(&s), want, "{s:?} moved a peak");
                }
            }
        }
    }

    #[test]
    fn flanking_peaks_stay_in_a_mobile_frame() {
        let f = Frame::new(1080, 1920);
        for seed in 0..50 {
            let [back, ..] = peaks(&f, seed);
            let n = back.len();
            for p in &back[n / 2 - 1..=n / 2] {
                assert!(
                    0.0 < p.apex.0 && p.apex.0 < f.w,
                    "seed {seed}: a flanking apex at x={} is out of frame",
                    p.apex.0
                );
            }
        }
    }

    #[test]
    fn a_wide_frame_adds_peaks() {
        let back = |w, h| peaks(&Frame::new(w, h), 0)[0].len();
        assert!(back(3440, 1440) > back(1920, 1080));
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
