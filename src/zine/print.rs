//! Print geometry shared by every zine item: the frame, hatching, the sunburst and the header.
use std::f64::consts::PI;

use askama::Template;

use crate::geom::{fmt, pts};
use crate::zine::style::{RAYS, SHEET};

/// The mockups' 640x360 canvas mapped onto any frame. Sizes scale by `s`, as the
/// blueprint scales by `u`; `(cx, cy)` is the head centre.
pub struct Frame {
    pub w: f64,
    pub h: f64,
    pub s: f64,
    pub cx: f64,
    pub cy: f64,
}

impl Frame {
    pub fn new(w: u32, h: u32) -> Self {
        let (w, h) = (w as f64, h as f64);
        let s = w.min(h) / 360.0;
        Self {
            w,
            h,
            s,
            cx: w / 2.0,
            cy: h / 2.0 + 14.0 * s,
        }
    }

    /// A mockup point anchored to the head, which sits at (320, 194) there.
    pub fn at(&self, x: f64, y: f64) -> (f64, f64) {
        (
            self.cx + (x - 320.0) * self.s,
            self.cy + (y - 194.0) * self.s,
        )
    }
}

/// 45° lines, `spacing` apart, covering `poly`'s bounding box; the template clips them to `poly`.
pub fn hatch(poly: &[(f64, f64)], spacing: f64) -> Vec<[(f64, f64); 2]> {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for &(x, y) in poly {
        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
    }
    let span = x1 - x0;
    // from the line through the top-right corner to the one through the bottom-left
    let n = ((y1 - y0 + span) / spacing) as usize;
    (0..=n)
        .map(|i| {
            let y = y0 - span + i as f64 * spacing;
            [(x0, y), (x1, y + span)]
        })
        .collect()
}

/// Hatch lines as one path's data: a wide frame holds hundreds of strokes, so a `line` apiece
/// would not stay small.
pub fn hatch_path(lines: impl IntoIterator<Item = [(f64, f64); 2]>) -> String {
    lines
        .into_iter()
        .map(|[(x1, y1), (x2, y2)]| format!("M{} {}L{} {}", fmt(x1), fmt(y1), fmt(x2), fmt(y2)))
        .collect()
}

/// A segment's ends, formatted for a `line`'s `x1 y1 x2 y2`.
pub fn seg(a: (f64, f64), b: (f64, f64)) -> (String, String, String, String) {
    (fmt(a.0), fmt(a.1), fmt(b.0), fmt(b.1))
}

/// 14 wedges, the even ones of 28, from `c` out to `reach`.
pub fn sunburst(c: (f64, f64), reach: f64) -> Vec<String> {
    let ray = |k: usize| {
        let a = k as f64 * 2.0 * PI / 28.0;
        (c.0 + reach * a.cos(), c.1 + reach * a.sin())
    };
    (0..28)
        .step_by(2)
        .map(|k| pts(&[c, ray(k), ray(k + 1)]))
        .collect()
}

/// The print's backdrop, bottom to top: the sheet, the sunburst turning about its origin (none
/// under a moon), and the halftone disc.
#[derive(Template)]
#[template(path = "zine/header.svg")]
struct Header {
    ws: String,
    hs: String,
    sheet: &'static str,
    rays: &'static str,
    /// the turning origin and the wedges
    burst: Option<(String, Vec<String>)>,
    disc: (String, String, String),
}

/// `rays` and `disc` are mockup points, anchored to the head; `r` is the disc's mockup radius.
pub fn header(f: &Frame, rays: Option<(f64, f64)>, disc: (f64, f64), r: f64) -> String {
    let burst = rays.map(|(x, y)| {
        let c = f.at(x, y);
        let origin = format!("transform-origin:{}px {}px", fmt(c.0), fmt(c.1));
        // the diagonal outreaches every corner, however the wedges turn
        (origin, sunburst(c, f.w.hypot(f.h)))
    });
    let disc = f.at(disc.0, disc.1);
    Header {
        ws: fmt(f.w),
        hs: fmt(f.h),
        sheet: SHEET,
        rays: RAYS,
        burst,
        disc: (fmt(disc.0), fmt(disc.1), fmt(r * f.s)),
    }
    .render()
    .expect("writing to a String")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_head_anchored_point_follows_the_frame() {
        let f = Frame::new(1920, 1080);
        assert_eq!((f.s, f.cx, f.cy), (3.0, 960.0, 582.0));
        assert_eq!(f.at(320.0, 194.0), (f.cx, f.cy));
        assert_eq!(f.at(320.0, 180.0), (960.0, 540.0));
    }

    // the reference's first wedge, drawn there with reach 900
    #[test]
    fn the_sunburst_matches_the_reference() {
        let w = sunburst((320.0, 190.0), 900.0);
        assert_eq!(w.len(), 14);
        assert_eq!(w[0], "320,190 1220,190 1197.44,390.27");
    }

    #[test]
    fn hatching_covers_the_box_corner_to_corner() {
        let lines = hatch(&[(130.0, 78.0), (350.0, 345.0), (169.6, 345.0)], 6.0);
        assert_eq!(lines[0], [(130.0, -142.0), (350.0, 78.0)]);
        let last = lines.last().unwrap();
        assert!(last[0].1 <= 345.0 && last[0].1 > 345.0 - 6.0);
    }
}
