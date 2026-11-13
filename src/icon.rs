use std::f64::consts::PI;

use askama::Template;

use crate::geom::{fmt, pts, regular_poly};
use crate::style::PAL;

/// One ring of `ico_hexatri`'s badge: radius, sides, start angle, colour,
/// stroke width, stroke opacity, rotate-origin-y%.
type Ring = (f64, usize, f64, &'static str, f64, f64, Option<f64>);

struct RingMotion {
    cls: &'static str,
    style: String,
}

struct HexRing {
    points: String,
    col: &'static str,
    swd: String,
    o: String,
    motion: Option<RingMotion>,
}

#[derive(askama::Template)]
#[template(path = "hexatri.svg")]
struct Hexatri {
    rings: Vec<HexRing>,
    a: &'static str,
    core: String,
}

pub fn ico_hexatri(rotate: bool) -> String {
    let (a, b) = (PAL.a.as_str(), PAL.b.as_str());
    let rings: [Ring; 4] = [
        (88.0, 6, PI / 6.0, a, 3.6, 1.0, None),
        (80.0, 3, -PI / 2.0, b, 3.0, 1.0, Some(66.7)), // up triangle
        (48.0, 6, PI / 6.0, a, 2.6, 0.8, None),
        (42.0, 3, PI / 2.0, b, 2.4, 0.8, Some(33.3)), // down triangle
    ];
    let rings = rings
        .into_iter()
        .enumerate()
        .map(|(idx, (r, n, rot, col, swd, o, oy))| {
            let motion = oy.filter(|_| rotate).map(|oy| {
                let cls = if idx == 1 { "rspin" } else { "spin" };
                let dur = 24 - idx * 3;
                RingMotion {
                    cls,
                    style: format!("animation-duration:{dur}s;transform-origin:50% {oy}%"),
                }
            });
            HexRing {
                points: pts(&regular_poly(0.0, 0.0, r, n, rot)),
                col,
                swd: fmt(swd),
                o: fmt(o),
                motion,
            }
        })
        .collect();
    let core = pts(&regular_poly(0.0, 0.0, 16.0, 6, PI / 6.0));
    Hexatri { rings, a, core }
        .render()
        .expect("writing to a String")
}

struct ShipFacet {
    points: String,
    ramp: &'static str,
    opacity: String,
}

struct Exhaust {
    x1: i32,
    x2: i32,
    y: i32,
    o: String,
}

#[derive(askama::Template)]
#[template(path = "ship.svg")]
struct Ship {
    a: &'static str,
    b: &'static str,
    lit: &'static str,
    cy0: String,
    cy1: String,
    ramps: Vec<(&'static str, &'static str)>,
    facets: Vec<ShipFacet>,
    hull: String,
    core: String,
    exhaust: Vec<Exhaust>,
}

/// Cloaked delta ship: four translucent facets tiling the hull. No rotate variant: a spinning ship reads as a crash.
/// Relief is fill value, not linework: only the silhouette is stroked. One shared `userSpaceOnUse` ramp, so it
/// reads as one solid under one light. Nothing reaches 0.3 opacity: the translucency is the cloak.
pub fn ico_ship() -> String {
    let (a, b, lit) = (PAL.a.as_str(), PAL.b.as_str(), PAL.bg.0.as_str());
    let (n, r, l, t) = ((0.0, -80.0), (68.0, 46.0), (-68.0, 46.0), (0.0, 20.0));
    // ridge feet, 35% along the trailing edges
    let (kl, kr) = ((-23.8, 29.1), (23.8, 29.1));
    let (cy0, cy1) = (n.1 + 6.0, t.1);
    // shallow ramp (0.5 -> 1): the step between planes carries the relief, not the shading within one
    let ramps: Vec<(&'static str, &'static str)> = vec![("a", a), ("b", b)];
    let facets = [
        ([n, kl, l], "a", 0.09),
        ([n, t, kl], "b", 0.04),
        ([n, kr, t], "b", 0.28),
        ([n, r, kr], "a", 0.22),
    ]
    .into_iter()
    .map(|(facet, ramp, opacity)| ShipFacet {
        points: pts(&facet),
        ramp,
        opacity: fmt(opacity),
    })
    .collect();
    let hull = pts(&[n, r, t, l]);
    let core = pts(&regular_poly(0.0, -22.0, 8.0, 6, PI / 6.0));
    // exhaust, aft of the wing roots
    let exhaust = [(14, 42), (8, 54)]
        .into_iter()
        .enumerate()
        .map(|(i, (hw, y))| Exhaust {
            x1: -hw,
            x2: hw,
            y,
            o: fmt(0.6 - i as f64 * 0.28),
        })
        .collect();
    Ship {
        a,
        b,
        lit,
        cy0: fmt(cy0),
        cy1: fmt(cy1),
        ramps,
        facets,
        hull,
        core,
        exhaust,
    }
    .render()
    .expect("writing to a String")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attr<'a>(tag: &'a str, name: &str) -> &'a str {
        let key = format!("{name}=\"");
        let start = tag.find(&key).expect("attribute present") + key.len();
        let rest = &tag[start..];
        &rest[..rest.find('"').expect("closing quote")]
    }

    fn parse_points(s: &str) -> Vec<(f64, f64)> {
        s.split_whitespace()
            .map(|p| {
                let (x, y) = p.split_once(',').expect("point is x,y");
                (x.parse().unwrap(), y.parse().unwrap())
            })
            .collect()
    }

    fn polygons(glyph: &str) -> Vec<&str> {
        glyph
            .match_indices("<polygon ")
            .map(|(i, _)| {
                let end = glyph[i..].find("/>").expect("self-closing tag") + i;
                &glyph[i..end]
            })
            .collect()
    }

    fn area(poly: &[(f64, f64)]) -> f64 {
        let n = poly.len();
        let sum: f64 = (0..n)
            .map(|i| {
                let (x, y) = poly[i];
                let (xp, yp) = poly[(i + n - 1) % n];
                x * yp - xp * y
            })
            .sum();
        sum.abs() / 2.0
    }

    fn facet_polys(glyph: &str) -> Vec<(Vec<(f64, f64)>, f64)> {
        polygons(glyph)
            .into_iter()
            .filter(|tag| {
                tag.contains("fill=\"url(#shpa)\"") || tag.contains("fill=\"url(#shpb)\"")
            })
            .map(|tag| {
                (
                    parse_points(attr(tag, "points")),
                    attr(tag, "fill-opacity").parse().expect("a number"),
                )
            })
            .collect()
    }

    fn hull_poly(glyph: &str) -> Vec<(f64, f64)> {
        let hulls: Vec<&str> = polygons(glyph)
            .into_iter()
            .filter(|tag| tag.contains("fill=\"none\""))
            .collect();
        assert_eq!(
            hulls.len(),
            1,
            "the ship needs exactly one silhouette outline"
        );
        parse_points(attr(hulls[0], "points"))
    }

    /// Asserted against the glyph alone: the lattice would match these patterns by chance.
    #[test]
    fn the_ship_is_a_folded_solid() {
        let glyph = ico_ship();
        let facets = facet_polys(&glyph);
        assert_eq!(facets.len(), 4, "the hull needs four facets to fold");
        let vals: Vec<f64> = facets.iter().map(|(_, v)| *v).collect();
        for (i, v) in vals.iter().enumerate() {
            assert!(*v < 0.3, "a facet is too opaque to read as a cloak");
            assert!(
                vals[i + 1..].iter().all(|w| w != v),
                "two facets share a value, so that fold has no relief"
            );
        }
        let hull = hull_poly(&glyph);
        let tiled: f64 = facets.iter().map(|(p, _)| area(p)).sum();
        assert!(
            (tiled - area(&hull)).abs() < 1e-6,
            "the facets no longer tile the hull: the cloak changed the silhouette"
        );
        assert_eq!(
            glyph.matches("<linearGradient").count(),
            glyph.matches(r#"gradientUnits="userSpaceOnUse""#).count(),
            "a facet ramp restarts inside its facet, so that facet is lit on its own"
        );
    }

    #[test]
    #[should_panic(expected = "the ship needs exactly one silhouette outline")]
    fn hull_poly_rejects_a_second_silhouette() {
        let glyph = r#"<polygon points="0,0 1,0 1,1" fill="none"/><polygon points="2,2 3,2 3,3" fill="none"/>"#;
        hull_poly(glyph);
    }

    #[test]
    fn hexatri_spins_only_its_triangles() {
        let spinning = ico_hexatri(true);
        assert_eq!(spinning.matches("class=\"spin\"").count(), 1);
        assert_eq!(spinning.matches("class=\"rspin\"").count(), 1);
        assert!(!ico_hexatri(false).contains("class=\"spin\""));
        assert_eq!(
            ico_hexatri(true).matches("<polygon").count(),
            ico_hexatri(false).matches("<polygon").count(),
            "the motion must not change the glyph's geometry"
        );
    }
}
