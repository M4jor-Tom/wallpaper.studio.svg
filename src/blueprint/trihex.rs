use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

use askama::Template;

use crate::blueprint::style::{
    BLIND_S, FILL_O, PAL, SPACE_FRAC, SPACE_STARS, SPACE_STROKE_O, STROKE_O, VOID,
};
use crate::geom::{Lattice, NB, SQRT3, fmt, pts, regular_poly};
use crate::params::trihex::{Image, Motion};
use crate::rng::{PyRandom, cell_rng};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Role {
    Holder,
    Inter,
}

type Tri = ((f64, f64), (f64, f64), (f64, f64), String);

struct Plan {
    role: HashMap<(i32, i32), Role>,
    poked: HashSet<(i32, i32)>,
    tris: Vec<Tri>,
}

fn assign(lat: &Lattice, rng: &mut PyRandom) -> Plan {
    let hexset: HashSet<(i32, i32)> = lat.hexes.iter().copied().collect();
    let mut plan = Plan {
        role: HashMap::new(),
        poked: HashSet::new(),
        tris: Vec::new(),
    };
    let mut order = lat.hexes.clone();
    rng.shuffle(&mut order);
    for (r, c) in order {
        // Python's `or` short-circuits: a hexagon that has a role draws nothing.
        if plan.role.contains_key(&(r, c)) || rng.random() > 0.5 {
            continue;
        }
        let (cx, cy) = lat.center(r, c);
        let v = regular_poly(cx, cy, lat.s, 6, PI / 6.0);
        for k in rng.sample(6, 6) {
            let (dr, dc) = NB[r.rem_euclid(2) as usize][k];
            let n = (r + dr, c + dc);
            if plan.role.get(&n) == Some(&Role::Holder) || plan.poked.contains(&n) {
                continue;
            }
            // V[(k - 1) % 6]: Python wraps to V[5] at k = 0.
            let (v1, v2) = (v[(k + 5) % 6], v[k]);
            let mid = ((v1.0 + v2.0) / 2.0, (v1.1 + v2.1) / 2.0);
            let apex = (2.0 * mid.0 - cx, 2.0 * mid.1 - cy);
            let d = [v1, v2, apex]
                .iter()
                .map(|(x, y)| (x - lat.cx0).hypot(y - lat.cy0))
                .fold(f64::INFINITY, f64::min);
            if d < lat.clear_r {
                continue; // keep the icon zone empty
            }
            let col = if rng.random() < 0.6 { &PAL.a } else { &PAL.b };
            plan.tris.push((v1, v2, apex, col.clone()));
            plan.role.insert((r, c), Role::Holder);
            if hexset.contains(&n) {
                plan.role.insert(n, Role::Inter);
            }
            plan.poked.insert(n);
            break;
        }
    }
    plan
}

/// Hexagons that become windows; the whole cell must clear the icon zone. `every` (CLOSEOPEN) takes them all.
pub fn space_cells(lat: &Lattice, seed: u32, every: bool) -> HashSet<(i32, i32)> {
    let mut out = HashSet::new();
    for (r, c) in &lat.hexes {
        let (cx, cy) = lat.center(*r, *c);
        if (cx - lat.cx0).hypot(cy - lat.cy0) < lat.clear_r + lat.s {
            continue;
        }
        // short-circuit as in Python: no draw under `every`, none for ineligible cells
        if every || cell_rng("pick", seed, *r, *c).random() < SPACE_FRAC {
            out.insert((*r, *c));
        }
    }
    out
}

/// CLOSEOPEN shutter timing. The blind and its window share this one string, so they cannot desync.
fn blind_phase(seed: u32, r: i32, c: i32) -> String {
    let mut g = cell_rng("blind", seed, r, c);
    let d = BLIND_S.0 + g.random() * (BLIND_S.1 - BLIND_S.0);
    // the delay is drawn first, so it is bound first
    let delay = fmt(g.random() * d);
    format!("animation-delay:-{delay}s;animation-duration:{}s", fmt(d))
}

struct Star {
    x: String,
    y: String,
    r: String,
    o: String,
    bloom: Option<String>,
}

/// `phase` feeds both the window and its blind: the window hides while covered (SVG has no occlusion culling).
struct VoidCell {
    cid: String,
    poly: String,
    phase: Option<String>,
    nx: String,
    ny: String,
    nrx: String,
    nry: String,
    neb: &'static str,
    rot: String,
    stars: Vec<Star>,
}

#[allow(clippy::too_many_arguments)]
fn space_cell(
    seed: u32,
    poly: String,
    cx: f64,
    cy: f64,
    s: f64,
    r: i32,
    c: i32,
    phase: Option<String>,
) -> VoidCell {
    let mut g = cell_rng("star", seed, r, c);
    let cid = format!("sp{r}_{c}"); // cell coords, so ids are stable
    let (nx, ny) = (cx + (g.random() - 0.5) * s, cy + (g.random() - 0.5) * s);
    let (nrx, nry) = (s * (0.55 + g.random() * 0.5), s * (0.30 + g.random() * 0.3));
    let neb = if g.random() < 0.6 { "neba" } else { "nebb" };
    let ang = g.random() * 180.0;
    let (nxf, nyf) = (fmt(nx), fmt(ny));
    let rot = format!("rotate({} {nxf} {nyf})", fmt(ang));
    let stars = (0..SPACE_STARS)
        .map(|i| {
            let x = cx + (g.random() - 0.5) * 2.0 * s;
            let y = cy + (g.random() - 0.5) * SQRT3 * s;
            let mut rad = s * (0.008 + g.random().powi(2) * 0.022);
            let o = 0.35 + g.random() * 0.6;
            let bloom = if i < 2 {
                let b = fmt(rad * 3.5);
                rad *= 1.4;
                Some(b)
            } else {
                None
            };
            Star {
                x: fmt(x),
                y: fmt(y),
                r: fmt(rad),
                o: fmt(o),
                bloom,
            }
        })
        .collect();
    VoidCell {
        cid,
        poly,
        phase,
        nx: nxf,
        ny: nyf,
        nrx: fmt(nrx),
        nry: fmt(nry),
        neb,
        rot,
        stars,
    }
}

struct TriFill {
    points: String,
    col: String,
    style: Option<String>,
}

struct HexAnim {
    cls: &'static str,
    style: String,
}

struct HexBorder {
    poly: String,
    anim: Option<HexAnim>,
    fill: &'static str,
    fill_opacity: Option<&'static str>,
    so: String,
}

/// Layering is fixed by the template's loop order (voids, tris, hexes), not by field order.
/// The `+%}` markers in trihex.svg are load-bearing: they keep the space before an optional attribute.
#[derive(askama::Template)]
#[template(path = "blueprint/trihex.svg")]
struct Trihex {
    ws: String,
    hs: String,
    voids: Vec<VoidCell>,
    tris: Vec<TriFill>,
    hexes: Vec<HexBorder>,
    void: &'static str,
    star: &'static str,
    fill_o: String,
    ink: &'static str,
    sw: String,
}

pub fn pat_trihex(
    w: u32,
    h: u32,
    lat: &Lattice,
    seed: u32,
    motion: Motion,
    image: Image,
) -> String {
    let mut rng = PyRandom::new(&format!("trihex:{seed}"));
    let sw = fmt(lat.u * 0.0013);
    let plan = assign(lat, &mut rng);

    let scan_delay = |x: f64, y: f64| fmt(-((x + y) / (w as f64 + h as f64)) * 5.0);
    let closeopen = motion == Motion::Closeopen;
    let space = if image == Image::Starfield {
        space_cells(lat, seed, closeopen)
    } else {
        HashSet::new()
    };

    let mut voids = Vec::new();
    // Eager on purpose: the `lights` branch draws from the global stream in `lat.hexes` order, right after `assign()`.
    let mut hexes = Vec::with_capacity(lat.hexes.len());
    for (r, c) in &lat.hexes {
        let (cx, cy) = lat.center(*r, *c);
        let poly = pts(&regular_poly(cx, cy, lat.s, 6, PI / 6.0));
        let void = space.contains(&(*r, *c));
        if void {
            voids.push(space_cell(
                seed,
                poly.clone(),
                cx,
                cy,
                lat.s,
                *r,
                *c,
                closeopen.then(|| blind_phase(seed, *r, *c)),
            ));
        }
        // Window borders sit brighter, except under closeopen where every cell is a window.
        let so = if void && !closeopen {
            SPACE_STROKE_O
        } else {
            STROKE_O
        };
        let (anim, fill, fill_opacity) = match motion {
            Motion::Scan => (
                Some(HexAnim {
                    cls: "scan",
                    style: format!("animation-delay:{}s", scan_delay(cx, cy)),
                }),
                "none",
                None,
            ),
            // border-only pulse: the usual pale fill flash would wash the stars out
            Motion::Lights if void => {
                let delay = fmt(cell_rng("delay", seed, *r, *c).random() * 9.0);
                (
                    Some(HexAnim {
                        cls: "lightb",
                        style: format!("animation-delay:-{delay}s"),
                    }),
                    "none",
                    None,
                )
            }
            Motion::Lights => {
                // two draws off the global stream, delay before duration
                let delay = fmt(rng.random() * 9.0);
                let dur = fmt(7.0 + rng.random() * 5.0);
                (
                    Some(HexAnim {
                        cls: "light",
                        style: format!("animation-delay:-{delay}s;animation-duration:{dur}s"),
                    }),
                    PAL.a.as_str(),
                    Some("0"),
                )
            }
            _ => (None, "none", None),
        };
        hexes.push(HexBorder {
            poly,
            anim,
            fill,
            fill_opacity,
            so: so.to_string(),
        });
    }

    let tris = plan
        .tris
        .iter()
        .map(|(v1, v2, apex, col)| TriFill {
            points: pts(&[*v1, *v2, *apex]),
            col: col.clone(),
            style: (motion == Motion::Scan).then(|| {
                let (tx, ty) = ((v1.0 + v2.0 + apex.0) / 3.0, (v1.1 + v2.1 + apex.1) / 3.0);
                format!("animation-delay:{}s", scan_delay(tx, ty))
            }),
        })
        .collect();

    Trihex {
        ws: fmt(w as f64),
        hs: fmt(h as f64),
        voids,
        tris,
        hexes,
        void: VOID.as_str(),
        star: PAL.bg.0.as_str(),
        fill_o: FILL_O.to_string(),
        ink: PAL.ink.as_str(),
        sw,
    }
    .render()
    .expect("writing to a String")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{Glyph, Ground, Scene};
    use crate::svg::build_svg;

    fn starfield(motion: Motion) -> Scene {
        Scene {
            seed: 0,
            ground: Ground::Trihex {
                motion,
                image: Image::Starfield,
            },
            glyph: Glyph::Hexatri { rotate: true },
            overlay: None,
            spot: crate::zine::style::SPOT.to_string(),
        }
    }

    fn styles<'a>(svg: &'a str, class: &str) -> Vec<&'a str> {
        let key = format!("class=\"{class}\" style=\"");
        svg.match_indices(&key)
            .map(|(i, _)| {
                let rest = &svg[i + key.len()..];
                &rest[..rest.find('"').expect("an unterminated style attribute")]
            })
            .collect()
    }

    #[test]
    fn space_cells_clear_the_icon_zone_and_keep_their_blinds() {
        for (w, h) in [(1920u32, 1080u32), (1080, 1920)] {
            let lat = Lattice::new(w, h);
            let space = space_cells(&lat, 0, false);
            for (r, c) in &space {
                let (cx, cy) = lat.center(*r, *c);
                let d = (cx - lat.cx0).hypot(cy - lat.cy0);
                assert!(
                    d - lat.s >= lat.clear_r,
                    "a space cell overlaps the icon zone"
                );
            }
            assert!(!space.is_empty(), "no space cells were placed");
            assert!(
                space.len() as f64 <= lat.hexes.len() as f64 * SPACE_FRAC * 2.0,
                "space cells are not sparse"
            );

            let svg = build_svg(w, h, &starfield(Motion::Lights));
            assert_eq!(
                svg.matches("<clipPath id=\"sp").count(),
                space.len(),
                "rendered space cells != selected"
            );
            assert_eq!(
                svg.matches("class=\"lightb\"").count(),
                space.len(),
                "a space cell is missing its border pulse"
            );
            assert_eq!(
                svg.matches("class=\"light\"").count(),
                lat.hexes.len() - space.len(),
                "a space cell got the fill flash"
            );

            let every = space_cells(&lat, 0, true);
            assert!(
                space.is_subset(&every) && space.len() < every.len(),
                "closeopen must widen the window pool, not reuse the sparse one"
            );
            for (r, c) in &every {
                let (cx, cy) = lat.center(*r, *c);
                assert!(
                    (cx - lat.cx0).hypot(cy - lat.cy0) - lat.s >= lat.clear_r,
                    "a closeopen window overlaps the icon zone"
                );
            }

            let svg = build_svg(w, h, &starfield(Motion::Closeopen));
            assert_eq!(
                svg.matches("class=\"blind\"").count(),
                every.len(),
                "every eligible hexagon must be a window"
            );
            let tris = [&PAL.a, &PAL.b]
                .iter()
                .filter_map(|col| svg.find(&format!("fill=\"{col}\" fill-opacity=\"{FILL_O}\"")))
                .min()
                .expect("no triangle to check blind layering against");
            assert!(
                svg.rfind("class=\"blind\"").expect("a blind was rendered") < tris,
                "a blind is painted over the triangles instead of under them"
            );
            assert!(
                svg.contains("transform-origin:center;transform:scale(0)}"),
                "blinds must rest open, so prefers-reduced-motion still shows the starfield"
            );
            assert!(
                svg.contains(".win{animation:winvis 75s ease-in-out infinite;display:inline}"),
                "windows must rest rendered, for the same reason"
            );
            assert!(
                !svg.contains(&format!("stroke-opacity=\"{SPACE_STROKE_O}\"")),
                "closeopen windows must not raise the whole lattice to the window border opacity"
            );

            let wins = styles(&svg, "win");
            let blinds = styles(&svg, "blind");
            assert!(
                wins == blinds && !wins.is_empty(),
                "a window is out of phase with its own blind"
            );
            assert!(
                !build_svg(w, h, &starfield(Motion::Lights)).contains("class=\"win\""),
                "a bg without blinds has nothing covering its windows, so they must never switch off"
            );
        }
    }

    #[test]
    fn roles_are_exclusive_and_the_centre_is_clear() {
        for (w, h) in [(1920u32, 1080u32), (1080, 1920)] {
            let lat = Lattice::new(w, h);
            let mut rng = PyRandom::new("trihex:0");
            let plan = assign(&lat, &mut rng);
            let holders: HashSet<_> = plan
                .role
                .iter()
                .filter(|(_, v)| **v == Role::Holder)
                .map(|(k, _)| *k)
                .collect();
            let inters: HashSet<_> = plan
                .role
                .iter()
                .filter(|(_, v)| **v == Role::Inter)
                .map(|(k, _)| *k)
                .collect();
            assert!(
                holders.is_disjoint(&inters),
                "a hexagon is both holder and intersector"
            );
            assert_eq!(
                plan.poked.len(),
                holders.len(),
                "poked count != triangle count"
            );
            for (v1, v2, apex, _) in &plan.tris {
                let d = [v1, v2, apex]
                    .iter()
                    .map(|(x, y)| (x - lat.cx0).hypot(y - lat.cy0))
                    .fold(f64::INFINITY, f64::min);
                assert!(d >= lat.clear_r, "a triangle sits inside the icon zone");
            }
        }
    }
}
