//! No SVG markup in Rust source outside `#[cfg(test)]`: templates own every `<tag`.

use std::path::{Path, PathBuf};

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("a readable directory") {
        let path = entry.expect("a readable entry").path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn sources() -> Vec<PathBuf> {
    let mut out = Vec::new();
    rust_sources(Path::new("src"), &mut out);
    out
}

#[test]
fn purity_walks_subfolders() {
    assert!(sources().contains(&Path::new("src").join("blueprint").join("trihex.rs")));
}

/// A smoke test, not a proof -- `format!("{x}<polygon")` would slip through.
#[test]
fn no_svg_markup_in_rust_sources() {
    for path in sources() {
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
