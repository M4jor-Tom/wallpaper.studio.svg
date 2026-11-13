//! Golden corpus: every schema-valid config, kept beside the SVG it renders.
use bgsvg::params::{parse_res, valid_configs};
use bgsvg::render_to_string;
use sha2::{Digest, Sha512};
use std::collections::BTreeMap;
use std::path::Path;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");
const JSON_SUFFIX: &str = "_parameters.json";
const SVG_SUFFIX: &str = "_background.svg";

type Corpus = BTreeMap<String, (Vec<u8>, Vec<u8>)>;

type Found = BTreeMap<String, (String, Vec<u8>, Option<Vec<u8>>)>;

fn sha(blob: &[u8]) -> String {
    Sha512::digest(blob)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn text(blob: &[u8]) -> String {
    String::from_utf8_lossy(blob).trim().to_string()
}

fn names_in(dir: std::fs::ReadDir) -> Vec<String> {
    dir.map(|e| {
        e.expect("a readable entry")
            .file_name()
            .to_string_lossy()
            .into_owned()
    })
    .collect()
}

fn corpus() -> Corpus {
    let (w, h) = parse_res("").expect("the empty resolution is the 1080p default");
    let configs = valid_configs(0);
    let mut out = Corpus::new();
    for cfg in &configs {
        let line = serde_json::to_string(cfg).expect("a config must serialise");
        let svg = render_to_string(&line, w, h).unwrap_or_else(|e| panic!("{line}: {e}"));
        roxmltree::Document::parse(&svg).unwrap_or_else(|e| panic!("{line}: {e}"));
        // on disk: exactly what `bgsvg --configs` prints plus a newline; the name hashes these bytes
        let json = format!("{line}\n").into_bytes();
        out.insert(sha(&json), (json, svg.into_bytes()));
    }
    assert_eq!(
        out.len(),
        configs.len(),
        "two configs serialised to the same JSON"
    );
    out
}

fn first_diff(old: &[u8], new: &[u8]) -> String {
    let i = old
        .iter()
        .zip(new)
        .position(|(a, b)| a != b)
        .unwrap_or(old.len().min(new.len()));
    let window = |b: &[u8]| text(&b[i.saturating_sub(30)..(i + 40).min(b.len())]);
    format!(
        "      first differs at byte {i}, {} -> {} bytes\n      was ...{}...\n      now ...{}...",
        old.len(),
        new.len(),
        window(old),
        window(new),
    )
}

fn scan() -> (Found, Vec<String>) {
    let (mut found, mut bad) = (Found::new(), Vec::new());
    let Ok(dir) = std::fs::read_dir(ROOT) else {
        return (
            found,
            vec!["tests/golden does not exist; regenerate it".into()],
        );
    };
    let mut entries = names_in(dir);
    entries.sort();

    for entry in entries {
        let d = Path::new(ROOT).join(&entry);
        let tag = format!("{}...", &entry[..entry.len().min(16)]);
        let Ok(names) = std::fs::read_dir(&d) else {
            bad.push(format!("{entry}: stray file in the corpus root"));
            continue;
        };
        let mut names = names_in(names);
        names.sort();

        let (mut jsons, mut svgs) = (Vec::new(), Vec::new());
        let mut blobs = BTreeMap::new();
        for n in &names {
            let p = d.join(n);
            let stem = p
                .is_file()
                .then(|| {
                    n.strip_suffix(JSON_SUFFIX)
                        .or_else(|| n.strip_suffix(SVG_SUFFIX))
                })
                .flatten();
            let Some(stem) = stem else {
                bad.push(format!("{tag}/{n}: not a golden file"));
                continue;
            };
            let blob = std::fs::read(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            blobs.insert(n.clone(), blob);
            if stem != sha(&blobs[n]) {
                bad.push(format!(
                    "{tag}/{}...: contents do not hash to its own name (edited by hand?)",
                    &n[..n.len().min(16)]
                ));
            }
            if n.ends_with(JSON_SUFFIX) {
                jsons.push(n.clone());
            } else {
                svgs.push(n.clone());
            }
        }

        if svgs.len() != 1 {
            bad.push(format!(
                "{tag}: holds {} SVGs, expected the 1 it is named after",
                svgs.len()
            ));
        } else if svgs[0].strip_suffix(SVG_SUFFIX) != Some(entry.as_str()) {
            bad.push(format!(
                "{tag}: the SVG here does not hash to this directory"
            ));
        }
        if jsons.len() > 1 {
            let configs: Vec<String> = jsons.iter().map(|n| text(&blobs[n])).collect();
            bad.push(format!(
                "{tag}: {} configs render this one SVG, so an axis stopped changing the picture: {}",
                jsons.len(),
                configs.join(" ")
            ));
        } else if jsons.is_empty() {
            bad.push(format!("{tag}: holds no config"));
        }

        let svg = (svgs.len() == 1).then(|| blobs[&svgs[0]].clone());
        for n in jsons {
            let hash = n
                .strip_suffix(JSON_SUFFIX)
                .expect("a json name")
                .to_string();
            found.insert(hash, (entry.clone(), blobs[&n].clone(), svg.clone()));
        }
    }
    (found, bad)
}

fn verify() -> Vec<String> {
    let want = corpus();
    let (found, mut bad) = scan();

    let mut wanted: Vec<_> = want.iter().collect();
    wanted.sort_by(|a, b| (a.1).0.cmp(&(b.1).0));
    let mut stored_configs: Vec<_> = found.iter().collect();
    stored_configs.sort_by(|a, b| (a.1).1.cmp(&(b.1).1));

    for (json_hash, (json_blob, svg_blob)) in wanted {
        let Some((entry, _, stored)) = found.get(json_hash.as_str()) else {
            bad.push(format!("missing: {}", text(json_blob)));
            continue;
        };
        match stored {
            Some(s) if s != svg_blob => bad.push(format!(
                "changed: {}\n{}",
                text(json_blob),
                first_diff(s, svg_blob)
            )),
            _ if *entry != sha(svg_blob) => bad.push(format!(
                "changed: {}\n      {}... -> {}...",
                text(json_blob),
                &entry[..entry.len().min(16)],
                &sha(svg_blob)[..16]
            )),
            _ => {}
        }
    }
    for (json_hash, (_, json_blob, _)) in stored_configs {
        if !want.contains_key(json_hash.as_str()) {
            bad.push(format!("stale: {}", text(json_blob)));
        }
    }

    bad
}

fn report(bad: &[String]) -> String {
    format!(
        "golden: {} problem(s)\n  {}\n\nif the picture was meant to change:\n  \
         cargo test --test golden -- --ignored regenerate_the_corpus",
        bad.len(),
        bad.join("\n  ")
    )
}

/// Verify and regenerate must not interleave (`--include-ignored` runs both). Poison is ignored: a failing verify poisons it.
static CORPUS: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn the_corpus_is_unchanged() {
    let _lock = CORPUS.lock().unwrap_or_else(|e| e.into_inner());
    let bad = verify();
    assert!(bad.is_empty(), "{}", report(&bad));
}

/// Rewrites all 42 goldens. `#[ignore]`d: a moved golden is a regression until proven otherwise (ADR 0009).
#[test]
#[ignore = "rewrites the corpus; run it only after an intended visual change"]
fn regenerate_the_corpus() {
    let _lock = CORPUS.lock().unwrap_or_else(|e| e.into_inner());
    let want = corpus();
    std::fs::remove_dir_all(ROOT).expect("the corpus directory must be removable");
    for (json_hash, (json_blob, svg_blob)) in &want {
        let svg_hash = sha(svg_blob);
        let d = Path::new(ROOT).join(&svg_hash);
        std::fs::create_dir_all(&d).expect("the corpus directory must be writable");
        for (name, blob) in [
            (format!("{json_hash}{JSON_SUFFIX}"), json_blob),
            (format!("{svg_hash}{SVG_SUFFIX}"), svg_blob),
        ] {
            std::fs::write(d.join(&name), blob).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }
    let bad = verify();
    assert!(bad.is_empty(), "{}", report(&bad));
}
