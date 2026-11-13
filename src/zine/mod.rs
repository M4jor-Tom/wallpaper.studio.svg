//! The zine mood: a black print with paper linework and one spot ink.
pub mod halfpipe;
pub mod print;
pub mod punk;
pub mod rider;
pub mod snow;
pub mod style;
pub mod summit;
pub mod treeline;

use askama::Template;

use crate::geom::fmt;

/// The spot halftone every zine disc is filled with.
#[derive(Template)]
#[template(path = "zine/defs.svg")]
struct Defs<'a> {
    spot: &'a str,
    period: String,
    c: String,
    r: String,
}

pub fn defs(spot: &str, s: f64) -> String {
    Defs {
        spot,
        period: fmt(8.0 * s),
        c: fmt(4.0 * s),
        r: fmt(2.6 * s),
    }
    .render()
    .expect("writing to a String")
}

/// Every zine rule; a rider's only when a zine head is in the scene. Classes and keyframes start
/// with `z`, so they never meet blueprint's. Each animation rests where it starts, so the closing
/// reduced-motion rule leaves the print intact.
pub fn css(rider: bool) -> String {
    let mut out = concat!(
        "@keyframes zrot{to{transform:rotate(360deg)}}",
        // the origin is inline: it follows the head
        ".zrays{transform-box:view-box;animation:zrot 120s linear infinite}",
        // one tile period, set inline per resolution
        "@keyframes zfall{to{transform:var(--zt)}}",
        ".zflurry{animation:zfall 18s linear infinite}",
        ".zblizzard{animation:zfall 6s linear infinite}",
    )
    .to_string();
    if rider {
        // duration and direction are inline: they follow the rider's rpm
        out.push_str(
            ".zspin{transform-box:fill-box;transform-origin:center;animation:zrot linear infinite}",
        );
    }
    out.push_str("@media (prefers-reduced-motion:reduce){*{animation:none!important}}");
    out
}
