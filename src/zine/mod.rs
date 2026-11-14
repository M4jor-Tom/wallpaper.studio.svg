//! The zine mood: a black print with paper linework and one spot ink.
pub mod halfpipe;
pub mod local;
pub mod park;
pub mod print;
pub mod punk;
pub mod rider;
pub mod snow;
pub mod style;
pub mod summit;
pub mod treeline;

use askama::Template;

use crate::geom::fmt;
use crate::params::rider::Motion;
use crate::params::{Fx, Glyph, LocalLook, ParkLook, PunkLook};
use style::{RAINBOW, shade};

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

/// The zine rules. The sunburst's and the snow's ship on every zine page, used or not (a treeline
/// turns nothing, `NO_SNOW` drops nothing, a trihex ground has neither); a head animation's ship
/// only when it is on. Classes and keyframes start with `z`, so they never meet blueprint's. Each
/// animation rests where it starts, so the closing reduced-motion rule leaves the print intact.
///
/// A head's part animations ship only while its look is `beating()`, the same test that makes the
/// sticker set `--beat`, so no keyframe runs on a beat that is not there.
///
/// A `fill-box` pivot turns each copy of a part about that copy's own box, so it is only used where
/// every copy's box is the same: on art-only parts (`.zeye`, `.zfrost`), and on the gear (`.zspin`),
/// crest spikes (`.zsp`), pompom (`.zpom`) and locks (`.zlock`), whose silhouette boxes equal their
/// art boxes. SWAY turns the whole head, whose art box outreaches its outline's, so it pivots on an
/// explicit neck set inline by `rider::icon`, one point for shadow, outline and art.
pub fn css(glyph: &Glyph) -> String {
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
    if let Some(rig) = glyph.rig() {
        // duration and direction are inline: they follow the rider's rpm
        out.push_str(
            ".zspin{transform-box:fill-box;transform-origin:center;animation:zrot linear infinite}",
        );
        // every head animation runs on the rider's inline --beat
        out.push_str(match rig.motion {
            Motion::Still => "",
            Motion::Bob => concat!(
                ".zbob{animation:zbob var(--beat) ease-in-out infinite}",
                "@keyframes zbob{0%,100%{transform:translateY(0)}50%{transform:translateY(-7px)}}",
            ),
            // about the neck: the origin is inline, in glyph units on the default view-box
            Motion::Sway => concat!(
                ".zsway{animation:zsway calc(var(--beat)*2) ease-in-out infinite alternate}",
                "@keyframes zsway{from{transform:rotate(-8deg)}to{transform:rotate(8deg)}}",
            ),
        });
    }
    match glyph {
        Glyph::Hexatri { .. } | Glyph::Ship => {}
        Glyph::Punk(p) if p.beating() => punk_css(p, &mut out),
        Glyph::Park(p) if p.beating() => park_css(p, &mut out),
        Glyph::Local(l) if l.beating() => local_css(l, &mut out),
        // a still head: nothing on it moves to a beat
        Glyph::Punk(_) | Glyph::Park(_) | Glyph::Local(_) => {}
    }
    out.push_str("@media (prefers-reduced-motion:reduce){*{animation:none!important}}");
    out
}

/// The punk's animations, each only when on. All of them run on the rider's `--beat`.
fn punk_css(p: &PunkLook, out: &mut String) {
    if p.crest_wave {
        out.push_str(concat!(
            ".zwave .zsp{transform-box:fill-box;transform-origin:50% 100%;",
            "animation:zwave var(--beat) ease-in-out infinite;",
            "animation-delay:calc(var(--i)*var(--beat)*-.13)}",
            "@keyframes zwave{0%,100%{transform:rotate(-7deg)}50%{transform:rotate(7deg)}}",
        ));
    }
    if p.rainbow_flow {
        out.push_str(&rainbow_flow());
    }
    if p.chatter {
        out.push_str(concat!(
            ".zchat .zlo{animation:zchat calc(var(--beat)/2) steps(2,jump-none) infinite}",
            "@keyframes zchat{from{transform:translateY(0)}to{transform:translateY(4px)}}",
        ));
    }
    if p.eye_pop {
        out.push_str(concat!(
            ".zpop .zeye{transform-box:fill-box;transform-origin:center;",
            "animation:zpop calc(var(--beat)*4) ease-out infinite}",
            "@keyframes zpop{0%,12%,100%{transform:scale(1)}4%{transform:scale(1.45)}}",
        ));
    }
}

/// The park rider's animations, each only when on. All of them run on the rider's `--beat`.
fn park_css(p: &ParkLook, out: &mut String) {
    if p.pompom {
        out.push_str(concat!(
            ".zpom{transform-box:fill-box;transform-origin:50% 100%;",
            "animation:zpom var(--beat) ease-in-out infinite}",
            "@keyframes zpom{0%,100%{transform:rotate(-16deg)}50%{transform:rotate(16deg)}}",
        ));
    }
    if p.lens_flow {
        // the lens gradient's stops, not a filter: each runs a seventh of the cycle behind the last
        out.push_str(concat!(
            ".zlensflow stop{animation:zlensflow calc(var(--beat)*8) linear infinite;",
            "animation-delay:calc(var(--i)*var(--beat)*-8/7)}",
        ));
        out.push_str(&cycle("zlensflow", "stop-color", RAINBOW));
    }
    fx_css(p.fx, out);
}

/// The mountain local's animations, each only when on, on the rider's `--beat`.
fn local_css(l: &LocalLook, out: &mut String) {
    if l.beard_wind {
        // each lock flicks about its root, .18 of a beat behind the one before
        out.push_str(concat!(
            ".zlock{transform-box:fill-box;transform-origin:50% 0;",
            "animation:zlock var(--beat) ease-in-out infinite;",
            "animation-delay:calc(var(--i)*var(--beat)*-.18)}",
            "@keyframes zlock{0%,100%{transform:rotate(-9deg)}50%{transform:rotate(9deg)}}",
        ));
    }
    if l.frost {
        out.push_str(concat!(
            ".zfrost{transform-box:fill-box;transform-origin:center;",
            "animation:zfrost calc(var(--beat)*2) ease-in-out infinite;",
            "animation-delay:calc(var(--i)*var(--beat)*-.29)}",
            "@keyframes zfrost{0%,100%{transform:scale(.4);opacity:.4}50%{transform:scale(1.4);opacity:1}}",
        ));
    }
    fx_css(l.fx, out);
}

/// Shared by the park rider and the local. They rest at opacity 0 in the markup; only these
/// keyframes raise them.
fn fx_css(fx: Fx, out: &mut String) {
    if fx.glint {
        out.push_str(concat!(
            ".zgl{animation:zglint calc(var(--beat)*4) ease-in-out infinite}",
            "@keyframes zglint{0%,55%{transform:translateX(0);opacity:0}",
            "62%{opacity:1}100%{transform:translateX(130px);opacity:0}}",
        ));
    }
    if fx.breath {
        out.push_str(concat!(
            ".zbreath{animation:zpuff calc(var(--beat)*2) ease-out infinite}",
            "@keyframes zpuff{0%{opacity:0;transform:translate(-6px,4px)}30%{opacity:1}",
            "100%{opacity:0;transform:translate(10px,-10px)}}",
        ));
    }
}

/// Keyframes that walk `prop` once round `colours`, back to the first, evenly spaced.
fn cycle(name: &str, prop: &str, colours: [impl AsRef<str>; 7]) -> String {
    let stops: String = (0..=7)
        .map(|k| {
            format!(
                "{}%{{{prop}:{}}}",
                fmt(k as f64 * 100.0 / 7.0),
                colours[k % 7].as_ref()
            )
        })
        .collect();
    format!("@keyframes {name}{{{stops}}}")
}

/// The crest's colours travel round it once every 8 beats. Each spike's lit half cycles the
/// rainbow and its shade half the shaded rainbow, a seventh of the cycle behind the spike before
/// it, so at t = 0 (and at rest) every spike shows its own colour. Fill keyframes, not a
/// `hue-rotate` filter: measured, the filter costs frames.
fn rainbow_flow() -> String {
    [
        ".zflow .zsp>*{animation:zflow calc(var(--beat)*8) linear infinite;",
        "animation-delay:calc(var(--i)*var(--beat)*-8/7)}",
        ".zflow .zsp>:last-child{animation-name:zflowshade}",
        &cycle("zflow", "fill", RAINBOW),
        &cycle(
            "zflowshade",
            "fill",
            RAINBOW.map(|c| shade(c, punk::NAPE_SHADE)),
        ),
    ]
    .concat()
}
