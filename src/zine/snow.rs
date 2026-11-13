//! Falling snow: one seeded tile, slid down by exactly one period so the loop is seamless.
//! The markup is the tile, so it does not grow with the frame.
use askama::Template;

use crate::geom::fmt;
use crate::params::Snow;
use crate::rng::PyRandom;
use crate::zine::print::Frame;
use crate::zine::style::PAPER;

struct Flake {
    x: String,
    y: String,
    d: String,
    o: String,
}

struct Streak {
    x1: String,
    y1: String,
    x2: String,
    y2: String,
    o: String,
}

/// No `transform` attribute on the rect: under reduced motion it rests where it starts.
#[derive(Template)]
#[template(path = "zine/snow.svg")]
struct Layer {
    tw: String,
    th: String,
    paper: &'static str,
    flakes: Vec<Flake>,
    streaks: Vec<Streak>,
    sw: String,
    class: &'static str,
    y: String,
    rw: String,
    rh: String,
    style: String,
}

pub fn snow(f: &Frame, seed: u32, kind: Snow) -> String {
    let s = f.s;
    let mut rng = PyRandom::new(&format!("snow:{seed}"));
    let (mut flakes, mut streaks) = (Vec::new(), Vec::new());
    // a tile period, and how far one loop carries the rect: down one period, or left one and down two
    let (tw, th, class, (dx, dy)) = match kind {
        Snow::NoSnow => return String::new(),
        Snow::Flurry => {
            for _ in 0..55 {
                let d = rng.uniform(1.6, 2.9);
                // inside the tile: a flake cut by its edge would show half a square
                let (x, y) = (rng.uniform(0.0, 360.0 - d), rng.uniform(0.0, 360.0 - d));
                let o = rng.uniform(0.4, 0.9);
                flakes.push(Flake {
                    x: fmt(x * s),
                    y: fmt(y * s),
                    d: fmt(d * s),
                    o: fmt(o),
                });
            }
            (360.0, 360.0, "zflurry", (0.0, 360.0))
        }
        Snow::Blizzard => {
            for _ in 0..340 {
                // a (-5, 9) streak, about 10 long, slanted with the drift
                let (x, y) = (rng.uniform(5.0, 360.0), rng.uniform(0.0, 711.0));
                let o = rng.uniform(0.4, 0.9);
                streaks.push(Streak {
                    x1: fmt(x * s),
                    y1: fmt(y * s),
                    x2: fmt((x - 5.0) * s),
                    y2: fmt((y + 9.0) * s),
                    o: fmt(o),
                });
            }
            (360.0, 720.0, "zblizzard", (-360.0, 720.0))
        }
    };
    Layer {
        tw: fmt(tw * s),
        th: fmt(th * s),
        paper: PAPER,
        flakes,
        streaks,
        sw: fmt(1.2 * s),
        class,
        // the frame plus the one period the loop carries in from above (and from the right)
        y: fmt(-dy * s),
        rw: fmt(f.w - dx * s),
        rh: fmt(f.h + dy * s),
        style: format!("--zt:translate({}px,{}px)", fmt(dx * s), fmt(dy * s)),
    }
    .render()
    .expect("writing to a String")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snow_markup_does_not_grow_with_resolution() {
        let flakes = |w, h| {
            snow(&Frame::new(w, h), 0, Snow::Flurry)
                .matches("<rect")
                .count()
        };
        assert_eq!(flakes(3840, 2160), flakes(1280, 720));
        assert_eq!(
            flakes(1280, 720),
            55 + 1,
            "55 flakes and the rect they fill"
        );
    }

    #[test]
    fn snow_rests_where_it_starts() {
        for kind in [Snow::Flurry, Snow::Blizzard] {
            let out = snow(&Frame::new(1920, 1080), 0, kind);
            assert!(
                out.contains("class=\"z"),
                "the snow is not animated: {kind:?}"
            );
            assert!(
                !out.contains(" transform="),
                "the snow rests displaced: {kind:?}"
            );
        }
        assert_eq!(snow(&Frame::new(1920, 1080), 0, Snow::NoSnow), "");
    }
}
