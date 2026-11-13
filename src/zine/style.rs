//! The zine palette: one black print, paper, and a single spot ink.
use crate::color::mix;

pub const SHEET: &str = "#111114";
/// lines, snowcaps, snow, the sticker edge
pub const PAPER: &str = "#f3efe6";
/// ink on paper
pub const INK: &str = "#16161a";
pub const HEAD: &str = "#0d0d12";
/// the sunburst's dim wedges
pub const RAYS: &str = "#1c1c21";
/// `zine.spot` when unset
pub const SPOT: &str = "#e63946";
/// front to back
pub const RAINBOW: [&str; 7] = [
    "#ff3b3b", "#ff8a1f", "#ffd400", "#2ed573", "#1e90ff", "#5b4bff", "#c04bff",
];

/// Darken toward black, as the mockups shade a colour's unlit side.
pub fn shade(hex: &str, t: f64) -> String {
    mix(hex, "#000000", t)
}

#[cfg(test)]
mod tests {
    use super::*;

    // the violet spike's nape half in the reference renders
    #[test]
    fn shade_matches_the_reference() {
        assert_eq!(shade("#c04bff", 0.28), "#8a36b8");
    }
}
