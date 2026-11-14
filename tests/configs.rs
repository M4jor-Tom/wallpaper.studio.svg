use bgsvg::mood::Mood;
use bgsvg::params::{Glyph, Ground, parse, resolve, trihex, valid_configs, validate};
use bgsvg::svg::build_svg;
use bgsvg::{load, render_to_string};
use serde_json::json;

#[test]
fn every_valid_config_dispatches_correctly() {
    let mut n = 0;
    for cfg in valid_configs(1) {
        let text = serde_json::to_string(&cfg).unwrap();
        let p = parse(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
        validate(&p).unwrap_or_else(|e| panic!("{text}: {e}"));
        let scene = resolve(&p);
        let svg = build_svg(640, 360, &scene);

        assert!(svg.starts_with("<svg") && svg.contains("prefers-reduced-motion"));
        // read off the config itself, so `uses` is checked against the input rather than against `scene`
        let has = |axis: &str, item: &str| cfg[axis].get(item).is_some();
        assert_eq!(
            scene.uses(Mood::Zine),
            has("background", "summit")
                || has("background", "treeline")
                || has("background", "halfpipe")
                || has("icon", "punk")
                || has("icon", "park"),
            "{text}"
        );
        assert_eq!(
            scene.uses(Mood::Blueprint),
            has("background", "trihex")
                || has("icon", "hexatri")
                || has("icon", "ship")
                || has("overlay", "matrix"),
            "{text}"
        );
        let ship = scene.glyph == Glyph::Ship;
        // zine hatching draws lines clipped to their faces, so these two only tell on a blueprint page
        if !scene.uses(Mood::Zine) {
            assert_eq!(
                svg.contains("<line x1="),
                ship,
                "icon dispatch is wrong: {text}"
            );
            assert_eq!(
                svg.contains("<clipPath"),
                matches!(
                    scene.ground,
                    Ground::Trihex {
                        image: trihex::Image::Starfield,
                        ..
                    }
                ),
                "bg-image dispatch is wrong: {text}"
            );
        }
        assert_eq!(
            svg.contains("url(#shp"),
            ship,
            "ship facets leaked across icons: {text}"
        );
        assert_eq!(
            svg.contains(r#"class="blind""#),
            matches!(
                scene.ground,
                Ground::Trihex {
                    motion: trihex::Motion::Closeopen,
                    ..
                }
            ),
            "blinds must exist exactly when closeopen has windows to cover: {text}"
        );
        assert_eq!(
            svg.contains(r#"class="rain""#),
            scene.overlay.is_some(),
            "overlay dispatch is wrong: {text}"
        );
        assert_eq!(
            svg.contains("@keyframes zrot"),
            scene.uses(Mood::Zine),
            "zine CSS must appear exactly when a zine item does: {text}"
        );
        assert_eq!(
            svg.contains("url(#ink)"),
            matches!(scene.glyph, Glyph::Hexatri { .. } | Glyph::Ship),
            "only blueprint glyphs take the drop shadow: {text}"
        );
        assert!(
            !svg.contains("<filter") || scene.uses(Mood::Blueprint),
            "zine output must never carry a filter: {text}"
        );
        n += 1;
    }
    assert_eq!(n, 116);
}

#[test]
fn an_empty_config_renders_the_defaults() {
    let from_empty = build_svg(640, 360, &resolve(&parse("{}").unwrap()));
    let spelled_out = build_svg(
        640,
        360,
        &resolve(
            &parse(
                r#"{"seed":0,"background":{"trihex":{"motion":"STATIC","image":"NONE"}},"icon":{"hexatri":{"motion":"ROTATE"}}}"#,
            )
            .unwrap(),
        ),
    );
    assert_eq!(from_empty, spelled_out);
}

#[test]
fn an_inert_zine_block_changes_nothing() {
    let render = |cfg: &serde_json::Value| {
        let (_, scene) = load(&cfg.to_string()).unwrap_or_else(|e| panic!("{cfg}: {e}"));
        (scene.uses(Mood::Zine), build_svg(640, 360, &scene))
    };
    let mut n = 0;
    for mut cfg in valid_configs(0) {
        let (zine, plain) = render(&cfg);
        if zine {
            continue;
        }
        cfg["zine"] = json!({"spot": "#22c8e5"});
        assert_eq!(render(&cfg).1, plain, "the zine block moved {cfg}");
        n += 1;
    }
    assert!(n > 0, "no blueprint-only config to try");
}

fn summit_rider(rider: serde_json::Value) -> String {
    let cfg = json!({"background": {"summit": {}}, "icon": {"punk": {"rider": rider}}});
    render_to_string(&cfg.to_string(), 640, 360).unwrap_or_else(|e| panic!("{cfg}: {e}"))
}

#[test]
fn counter_clockwise_without_spin_changes_nothing() {
    assert_eq!(
        summit_rider(json!({"spin_rpm": 0})),
        summit_rider(json!({"spin_rpm": 0, "counter_clockwise": true}))
    );
}

#[test]
fn bpm_without_any_head_animation_changes_nothing() {
    assert_eq!(summit_rider(json!({})), summit_rider(json!({"bpm": 150})));
}

#[test]
fn a_typed_spot_reaches_the_svg_normalised() {
    let svg = render_to_string(
        r##"{"background":{"summit":{}},"zine":{"spot":" #22C8E5 "}}"##,
        640,
        360,
    )
    .unwrap();
    assert!(svg.contains("#22c8e5") && !svg.contains("22C8E5"));
}
