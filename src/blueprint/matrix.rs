use askama::Template;

use crate::blueprint::style::{MATRIX_FRAC, MATRIX_GLYPHS, MATRIX_HEAD_STEP, MATRIX_KF, MATRIX_S};
use crate::color::hex_rgba;
use crate::geom::{Lattice, fmt};
use crate::rng::cell_rng;

/// `ch` is emitted unescaped: safe only because `MATRIX_GLYPHS` excludes `< > & " '`.
struct RainCell {
    x: String,
    y: String,
    style: String,
    o: String,
    ch: char,
}

struct RainColumn {
    style: String,
    cells: Vec<RainCell>,
}

#[derive(askama::Template)]
#[template(path = "blueprint/matrix.svg")]
struct Matrix {
    fs: String,
    rgb: String,
    style: String,
    columns: Vec<RainColumn>,
}

/// Character rain at `angle` degrees (0 = down, clockwise). NOTHING MOVES: glyphs are fixed, the lighting travels.
/// Cells are rotated in Rust, so glyphs stay upright with no transform. Each glyph's fill-opacity is its t=0
/// value, the frame reduced motion freezes. Columns use `cell_rng` keyed by slot, so the overlay never moves a hexagon.
pub fn pat_matrix(w: u32, h: u32, lat: &Lattice, seed: u32, angle: f64, color: &str) -> String {
    let (rgb, alpha) = hex_rgba(color).expect("validate() already accepted the colour");
    let (hold, out_at) = MATRIX_KF;
    let rad = angle.to_radians();
    let (co, si) = (rad.cos(), rad.sin());
    // the band -- P across x, T along
    let (band_p, band_t) = (
        w as f64 * co.abs() + h as f64 * si.abs(),
        w as f64 * si.abs() + h as f64 * co.abs(),
    );
    let fs = lat.u / 28.0;
    let (pitch, step) = (fs * 1.6, fs * 1.05);
    let (cx0, cy0) = (lat.cx0, lat.cy0);
    let n = (band_t / step) as i64 + 2; // cells spanning the band, head to tail

    let mut columns = Vec::new();
    for i in 0..(band_p / pitch) as i32 + 1 {
        let mut g = cell_rng("rain", seed, 0, i);
        if g.random() > MATRIX_FRAC {
            continue;
        }
        let dx = i as f64 * pitch - band_p / 2.0;
        let dur = g.uniform(MATRIX_S.0, MATRIX_S.1);
        let head = g.randrange(n as u32) as i64; // the cell lit at t=0
        let cells = (0..n)
            .map(|j| {
                let dy = j as f64 * step - band_t / 2.0;
                // rem_euclid: `head - j` is mostly negative, Python's `%` never is
                let pct = 100.0 * (head - j).rem_euclid(n) as f64 / n as f64;
                let o = if pct <= hold {
                    alpha * (1.0 - (1.0 - MATRIX_HEAD_STEP) * pct / hold)
                } else if pct <= out_at {
                    alpha * MATRIX_HEAD_STEP * (1.0 - (pct - hold) / (out_at - hold))
                } else {
                    0.0
                };
                RainCell {
                    x: fmt(cx0 + dx * co - dy * si),
                    y: fmt(cy0 + dx * si + dy * co),
                    style: format!(
                        "animation-delay:-{}s",
                        fmt(dur * (head + n - j) as f64 / n as f64)
                    ),
                    o: fmt(o),
                    ch: g.choice(MATRIX_GLYPHS) as char,
                }
            })
            .collect();
        columns.push(RainColumn {
            style: format!("--d:{}s", fmt(dur)),
            cells,
        });
    }

    Matrix {
        fs: fmt(fs),
        rgb,
        style: format!("--o:{};--t:{}", fmt(alpha), fmt(alpha * MATRIX_HEAD_STEP)),
        columns,
    }
    .render()
    .expect("writing to a String")
}

#[cfg(test)]
mod tests {
    use crate::blueprint::style::{MATRIX_COLOR, MATRIX_GLYPHS};
    use crate::geom::fmt;
    use crate::params::{Glyph, Rain, Scene, trihex};
    use crate::svg::build_svg;

    fn scene(overlay: Option<Rain>) -> Scene {
        Scene {
            seed: 3,
            motion: trihex::Motion::Lights,
            image: trihex::Image::Starfield,
            glyph: Glyph::Hexatri { rotate: true },
            overlay,
        }
    }

    fn polys(s: &str) -> Vec<&str> {
        s.match_indices("<polygon")
            .map(|(i, _)| &s[i..=i + s[i..].find('>').expect("a closing bracket")])
            .collect()
    }

    fn columns(s: &str) -> Vec<&str> {
        s.match_indices("<g style=\"--d:")
            .map(|(i, _)| {
                let rest = &s[i..];
                let open = rest.find("\">").expect("an open tag") + 2;
                let close = rest.find("</g>").expect("a closing tag");
                &rest[open..close]
            })
            .collect()
    }

    fn glyphs(s: &str) -> Vec<u8> {
        s.match_indices("</text>")
            .map(|(i, _)| s.as_bytes()[i - 1])
            .collect()
    }

    fn nums(s: &str, key: &str, end: char) -> Vec<f64> {
        s.match_indices(key)
            .map(|(i, _)| {
                let rest = &s[i + key.len()..];
                rest[..rest.find(end).expect("a terminator")]
                    .parse()
                    .expect("a number")
            })
            .collect()
    }

    #[test]
    fn the_rain_lights_a_column_without_moving_anything() {
        for (w, h) in [(1920u32, 1080u32), (1080, 1920)] {
            let plain = build_svg(w, h, &scene(None));
            for angle in [0.0, 90.0, 181.0, 359.5] {
                let svg = build_svg(
                    w,
                    h,
                    &scene(Some(Rain {
                        angle,
                        color: MATRIX_COLOR.to_string(),
                    })),
                );
                assert_eq!(
                    polys(&svg),
                    polys(&plain),
                    "the overlay moved the lattice at {} deg",
                    fmt(angle)
                );

                // the halo is the first `<circle` AFTER the layer: stars are circles too
                let start = svg.find("class=\"matrix\"").expect("the rain layer");
                let rest = &svg[start..];
                let layer = &rest[..rest.find("<circle").expect("the halo")];
                assert!(
                    !layer.contains("transform"),
                    "a rain glyph carries a transform at {} deg",
                    fmt(angle)
                );
                let glyphs = glyphs(&svg);
                assert!(
                    !glyphs.is_empty() && glyphs.iter().all(|c| MATRIX_GLYPHS.contains(c)),
                    "a glyph is off the set"
                );

                let cols = columns(&svg);
                assert!(!cols.is_empty(), "no rain columns were placed");
                let cells = cols[0].matches("<text").count();
                assert!(
                    cols.iter().all(|col| col.matches("<text").count() == cells),
                    "every column must span the whole band, or a head would stop mid-canvas"
                );
                for col in &cols {
                    let o = nums(col, "fill-opacity=\"", '"');
                    let lit = o.iter().filter(|v| **v != 0.0).count();
                    assert!(
                        0 < lit && lit < o.len(),
                        "a column must be part lit and part dark -- that gap is the trail"
                    );
                    let top = o.iter().copied().fold(f64::MIN, f64::max);
                    assert_eq!(
                        o.iter().filter(|v| **v == top).count(),
                        1,
                        "a column must have exactly one head"
                    );
                    let head = o.iter().position(|v| *v == top).expect("the head");
                    let n = o.len() as i64;
                    let run: Vec<f64> = (0..lit as i64)
                        .map(|k| o[(head as i64 - k).rem_euclid(n) as usize])
                        .collect();
                    assert!(
                        run.iter().all(|v| *v != 0.0) && run.windows(2).all(|w| w[0] >= w[1]),
                        "the trail must be one contiguous run fading back from the head"
                    );
                }

                let d = nums(cols[0], "animation-delay:-", 's');
                let gaps: Vec<f64> = d.windows(2).map(|w| w[0] - w[1]).collect();
                let lo = gaps.iter().copied().fold(f64::MAX, f64::min);
                let hi = gaps.iter().copied().fold(f64::MIN, f64::max);
                assert!(
                    lo > 0.0 && hi - lo <= 0.02,
                    "glyph delays must step by one constant cell-time down the column"
                );

                let kf = &svg[svg.find("@keyframes rain{").expect("the rain keyframes")..];
                assert!(
                    !kf[..kf.find("}}").expect("the end of the rule")].contains("translate"),
                    "the rain keyframes must animate opacity only -- characters never move"
                );
                assert!(
                    svg.contains(".rain{animation:rain var(--d) linear infinite}"),
                    "columns must take their speed from the inherited --d"
                );
            }

            let svg = build_svg(
                w,
                h,
                &Scene {
                    seed: 0,
                    motion: trihex::Motion::Static,
                    image: trihex::Image::None,
                    glyph: Glyph::Hexatri { rotate: true },
                    overlay: Some(Rain {
                        angle: 0.0,
                        color: "#8899aa80".to_string(),
                    }),
                },
            );
            assert!(
                svg.contains("fill=\"#8899aa\"") && !svg.contains("#8899aa80"),
                "overlay.matrix.color is not applied"
            );
            assert!(
                svg.contains(&format!("--o:{};", fmt(0x80 as f64 / 255.0))),
                "the colour's alpha never reaches the glyphs"
            );
        }
    }
}
