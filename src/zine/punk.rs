//! The punk: a black head in three-quarter view, ✕ eyes, a banana grin and a rainbow mohawk.
use std::f64::consts::PI;

use askama::Template;

use crate::geom::{fmt, pts};
use crate::zine::rider::Sticker;
use crate::zine::style::{HEAD, RAINBOW, shade};

/// The head disc, front view, in the 200-unit glyph grid.
const R: f64 = 56.0;
const CY: f64 = 8.0;
/// how far the head is turned to the viewer's right
const PHI: f64 = 35.0 * PI / 180.0;
/// how far each spike reaches past the scalp, forehead to nape
const CREST: [f64; 7] = [26.0, 44.0, 54.0, 52.0, 44.0, 34.0, 24.0];

type Pt = (f64, f64);

/// Turn a head-space point (`z` toward the viewer) about the vertical axis: its screen point, and
/// its depth, positive on the near side.
fn turn(x: f64, y: f64, z: f64) -> (Pt, f64) {
    let (s, c) = PHI.sin_cos();
    ((x * c + z * s, y), z * c - x * s)
}

/// A feature drawn on the front-view disc, lifted onto the sphere and turned.
fn wrap((x, y): Pt) -> (Pt, f64) {
    turn(x, y, (R * R - x * x - (y - CY).powi(2)).sqrt())
}

/// A point of the mid-plane fin: `rho` out at `deg` from the crown (positive toward the
/// forehead), `x` off the plane.
fn fin(x: f64, deg: f64, rho: f64) -> (Pt, f64) {
    let (s, c) = deg.to_radians().sin_cos();
    turn(x, CY - rho * c, rho * s)
}

/// The ✕ eyes, front view: two strokes each.
fn eyes() -> [[[Pt; 2]; 2]; 2] {
    let (y, a) = (1.0, 7.5);
    [-21.0, 21.0].map(|x| {
        [
            [(x - a, y - a), (x + a, y + a)],
            [(x - a, y + a), (x + a, y - a)],
        ]
    })
}

/// A grin edge sagging `sag` at the middle: the upper is `lip(6, x)`, the lower `lip(24, x)`.
fn lip(sag: f64, x: f64) -> f64 {
    20.0 + sag * (1.0 - (x / 36.0).powi(2))
}

/// A tooth 12 wide on `x`, based on the `from` edge and reaching 78% of the way to `to`, shrunk
/// 20% toward its own centroid.
fn tooth(x: f64, from: f64, to: f64) -> [Pt; 3] {
    let t = [
        (x - 6.0, lip(from, x - 6.0)),
        (x + 6.0, lip(from, x + 6.0)),
        (x, lip(from + 0.78 * (to - from), x)),
    ];
    let (gx, gy) = (
        (t[0].0 + t[1].0 + t[2].0) / 3.0,
        (t[0].1 + t[1].1 + t[2].1) / 3.0,
    );
    t.map(|(x, y)| (gx + 0.8 * (x - gx), gy + 0.8 * (y - gy)))
}

/// The banana grin, front view: six teeth hang from the upper edge, five rise between them.
fn teeth() -> ([[Pt; 3]; 6], [[Pt; 3]; 5]) {
    (
        [-30.0, -18.0, -6.0, 6.0, 18.0, 30.0].map(|x| tooth(x, 6.0, 24.0)),
        [-24.0, -12.0, 0.0, 12.0, 24.0].map(|x| tooth(x, 24.0, 6.0)),
    )
}

/// Convex hull (Andrew's monotone chain).
fn hull(points: &[Pt]) -> Vec<Pt> {
    let mut p = points.to_vec();
    p.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let cross = |o: Pt, a: Pt, b: Pt| (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0);
    let mut h: Vec<Pt> = Vec::new();
    for pass in [p.clone(), p.into_iter().rev().collect()] {
        let floor = h.len();
        for q in pass {
            while h.len() >= floor + 2 && cross(h[h.len() - 2], h[h.len() - 1], q) <= 0.0 {
                h.pop();
            }
            h.push(q);
        }
        h.pop();
    }
    h
}

struct Spike {
    /// of its base centre; negative behind the head
    depth: f64,
    style: String,
    lit: String,
    lit_fill: &'static str,
    shade: String,
    shade_fill: String,
}

/// The mohawk back to front, split at the head: the spikes behind it, then those in front.
fn crest() -> (Vec<Spike>, Vec<Spike>) {
    let mut spikes: Vec<Spike> = CREST
        .iter()
        .enumerate()
        .map(|(k, len)| {
            let deg = 46.0 + (-94.0 - 46.0) * k as f64 / 6.0;
            let (base, depth) = fin(0.0, deg, R - 3.0);
            // leaning toward the forehead
            let tip = fin(0.0, deg + 14.0, R + len).0;
            // the half on one side of the base centre: the forehead's is lit, the nape's shaded
            let half = |side: f64| {
                let [a, b] = [-7.0, 7.0].map(|x| fin(x, deg + side, R - 3.0).0);
                pts(&hull(&[a, b, base, tip]))
            };
            Spike {
                depth,
                style: format!("--i:{k}"),
                lit: half(12.5),
                lit_fill: RAINBOW[k],
                shade: half(-12.5),
                shade_fill: shade(RAINBOW[k], 0.28),
            }
        })
        .collect();
    spikes.sort_by(|a, b| a.depth.total_cmp(&b.depth));
    spikes.into_iter().partition(|s| s.depth < 0.0)
}

/// `paint` off is the silhouette: the same shapes, no fill, no face.
#[derive(Template)]
#[template(path = "zine/punk.svg")]
struct Punk<'a> {
    paint: bool,
    behind: &'a [Spike],
    front: &'a [Spike],
    r: String,
    cy: String,
    head: &'static str,
    eyes: &'a [Vec<(String, String, String, String)>],
    upper: &'a [String],
    lower: &'a [String],
}

pub fn head() -> Sticker {
    let (behind, front) = crest();
    let at = |p| wrap(p).0;
    let eyes: Vec<Vec<_>> = eyes()
        .iter()
        .map(|eye| {
            eye.iter()
                .map(|&[a, b]| {
                    let (a, b) = (at(a), at(b));
                    (fmt(a.0), fmt(a.1), fmt(b.0), fmt(b.1))
                })
                .collect()
        })
        .collect();
    let (upper, lower) = teeth();
    let teeth = |t: &[[Pt; 3]]| -> Vec<String> { t.iter().map(|t| pts(&t.map(at))).collect() };
    let (upper, lower) = (teeth(&upper), teeth(&lower));
    let draw = |paint| {
        Punk {
            paint,
            behind: &behind,
            front: &front,
            r: fmt(R),
            cy: fmt(CY),
            head: HEAD,
            eyes: &eyes,
            upper: &upper,
            lower: &lower,
        }
        .render()
        .expect("writing to a String")
    };
    Sticker {
        silhouette: draw(false),
        art: draw(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The markup of spike `k`, up to the end of its group.
    fn spike(art: &str, k: usize) -> &str {
        let open = format!(r#"<g class="zsp" style="--i:{k}">"#);
        let i = art.find(&open).unwrap_or_else(|| panic!("no spike {k}"));
        &art[i..i + art[i..].find("</g>").expect("a closed group")]
    }

    #[test]
    fn the_crest_runs_red_at_the_forehead_to_violet_at_the_nape() {
        let art = head().art;
        // lit half, then its shade (darkened 28%)
        assert!(spike(&art, 0).contains(r##"fill="#ff3b3b""##));
        assert!(spike(&art, 0).contains(r##"fill="#b82a2a""##));
        assert!(spike(&art, 6).contains(r##"fill="#c04bff""##));
        assert!(spike(&art, 6).contains(r##"fill="#8a36b8""##));
        assert_eq!(art.matches(r#"class="zsp""#).count(), 7);
    }

    #[test]
    fn the_grin_has_six_upper_and_five_lower_teeth() {
        let (upper, lower) = teeth();
        assert_eq!((upper.len(), lower.len()), (6, 5));
        let art = head().art;
        let teeth = &art[art.find(r##"<g fill="#fff">"##).expect("the teeth")..];
        let lo = teeth.find(r#"<g class="zlo">"#).expect("the lower jaw");
        assert_eq!(teeth[..lo].matches("<polygon").count(), 6);
        let lower = &teeth[lo..teeth[lo..].find("</g>").unwrap() + lo];
        assert_eq!(lower.matches("<polygon").count(), 5);
    }

    #[test]
    fn every_wrapped_feature_is_on_the_visible_hemisphere() {
        let (upper, lower) = teeth();
        let eyes = eyes().into_iter().flatten().flatten();
        let teeth = upper.into_iter().chain(lower).flatten();
        for p in eyes.chain(teeth) {
            assert!(wrap(p).1 > 0.0, "{p:?} wraps onto the far side");
        }
    }

    // a few points of the approved render, which printed one decimal
    #[test]
    fn the_head_matches_the_reference() {
        let near = |(a, b): (f64, f64), (x, y): (f64, f64)| {
            assert!(
                (a - x).abs() < 0.06 && (b - y).abs() < 0.06,
                "{a},{b} vs {x},{y}"
            );
        };
        near(wrap((-28.5, -6.5)).0, (3.0, -6.5)); // the left eye's first arm
        let (upper, lower) = teeth();
        near(wrap(upper[0][0]).0, (-4.4, 20.6));
        near(wrap(lower[2][1]).0, (29.1, 42.4));
        let art = head().art;
        assert!(spike(&art, 0).contains("40.73,-33"), "the red tip moved");
    }

    #[test]
    fn the_silhouette_is_the_shape_without_paint() {
        let s = head().silhouette;
        assert!(!s.contains("fill=") && !s.contains("stroke="), "{s}");
        assert_eq!(s.matches(r#"class="zsp""#).count(), 7);
        assert!(!s.contains("zeye") && !s.contains("zlo"));
    }
}
