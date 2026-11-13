//! The WASM build must render the same bytes as native. The sweep is tests/wasm.mjs: wasm-bindgen's glue is JS.
use std::process::Command;

/// Skips loudly without `BGSVG_WASM`: the module comes from `nix build .#bgsvg-wasm`.
#[test]
fn the_wasm_build_renders_the_corpus_byte_for_byte() {
    let Ok(pkg) = std::env::var("BGSVG_WASM") else {
        println!(
            "SKIPPED: BGSVG_WASM is unset, so the wasm build was not swept.\n  \
             nix build .#bgsvg-wasm\n  \
             BGSVG_WASM=$PWD/result/nodejs cargo test --workspace"
        );
        return;
    };

    let sweep = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/wasm.mjs");
    let out = Command::new("node")
        .arg(sweep)
        .env("BGSVG_WASM", &pkg)
        .output()
        .unwrap_or_else(|e| panic!("BGSVG_WASM is set but node could not run {sweep}: {e}"));

    assert!(
        out.status.success(),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    print!("{}", String::from_utf8_lossy(&out.stdout));
}
