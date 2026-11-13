use std::fmt::Write as _;

/// Mix two `#rrggbb` colours. `round_ties_even`: Python's `round` is half-to-even.
pub fn mix(h1: &str, h2: &str, t: f64) -> String {
    let channel = |h: &str, i: usize| {
        u8::from_str_radix(&h[i..i + 2], 16).expect("palette literals are valid hex") as f64
    };
    let mut out = String::from("#");
    for k in 0..3 {
        let i = 1 + 2 * k;
        let v = channel(h1, i) + (channel(h2, i) - channel(h1, i)) * t;
        write!(out, "{:02x}", v.round_ties_even() as u8).expect("writing to a String");
    }
    out
}

pub fn hex_rgba(s: &str) -> Result<(String, f64), String> {
    let t = s.trim();
    let bad = || format!("bad colour '{s}': use #rrggbb or #rrggbbaa");
    let digits = t.strip_prefix('#').ok_or_else(bad)?;
    let is_hex = |c: &u8| c.is_ascii_hexdigit();
    if !(digits.len() == 6 || digits.len() == 8) || !digits.bytes().all(|c| is_hex(&c)) {
        return Err(bad());
    }
    let (rgb, a) = digits.split_at(6);
    let alpha = if a.is_empty() {
        0xff
    } else {
        u8::from_str_radix(a, 16).map_err(|_| bad())?
    };
    Ok((format!("#{}", rgb.to_lowercase()), alpha as f64 / 255.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    // half-to-even: no palette colour lands on a tie, so only this test catches f64::round.
    #[test]
    fn mix_rounds_ties_to_even() {
        assert_eq!(mix("#000000", "#050505", 0.5), "#020202");
    }

    #[test]
    fn hex_rgba_splits_colour_and_alpha() {
        assert_eq!(hex_rgba("#8899AA").unwrap(), ("#8899aa".into(), 1.0));
        assert_eq!(hex_rgba(" #8899aa80 ").unwrap().1, 0x80 as f64 / 255.0);
        assert!(hex_rgba("395e53").is_err());
        assert!(hex_rgba("#395e5").is_err());
        assert!(hex_rgba("#395e53ccc").is_err());
    }
}
