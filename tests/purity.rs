//! No SVG markup in Rust source outside `#[cfg(test)]`: templates own every `<tag`.

/// A smoke test, not a proof -- `format!("{x}<polygon")` would slip through.
#[test]
fn no_svg_markup_in_rust_sources() {
    for entry in std::fs::read_dir("src").expect("src/ exists") {
        let path = entry.expect("a readable entry").path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let src = std::fs::read_to_string(&path).expect("valid utf-8");
        let production = match src.find("#[cfg(test)]") {
            Some(i) => &src[..i],
            None => &src[..],
        };
        assert!(
            !production.contains("\"<"),
            "{} contains SVG markup outside its #[cfg(test)] module; \
             move it into templates/ instead of building it in Rust",
            path.display()
        );
    }
}
