//! Summit: snowcapped peaks under a turning sunburst, with a halftone disc behind the head.
use askama::Template;

use crate::geom::{fmt, pts};
use crate::rng::PyRandom;
use crate::zine::print::{Frame, hatch, hatch_path, header};
use crate::zine::style::{PAPER, SHEET};

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
    hatch: String,
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
        hatch: hatch_path(hatch(&face, 6.0 * s)),
        cap: pts(&cap),
    }
}

/// Layering is the template's: header (sheet, sunburst, disc), back peaks, ground band, front peaks.
#[derive(Template)]
#[template(path = "zine/summit.svg")]
struct Summit<'a> {
    header: String,
    ws: String,
    sheet: &'static str,
    paper: &'static str,
    spot: &'a str,
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
    Summit {
        header: header(f, Some((320.0, 190.0)), (320.0, 180.0), 138.0),
        ws: fmt(f.w),
        sheet: SHEET,
        paper: PAPER,
        spot,
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
}
