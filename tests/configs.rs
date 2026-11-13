use bgsvg::params::{Glyph, Ground, parse, resolve, trihex, valid_configs, validate};
use bgsvg::svg::build_svg;

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
        let ship = scene.glyph == Glyph::Ship;
        assert_eq!(
            svg.contains("<line x1="),
            ship,
            "icon dispatch is wrong: {text}"
        );
        assert_eq!(
            svg.contains("url(#shp"),
            ship,
            "ship facets leaked across icons: {text}"
        );
        let Ground::Trihex { motion, image } = scene.ground;
        assert_eq!(
            svg.contains("<clipPath"),
            image == trihex::Image::Starfield,
            "bg-image dispatch is wrong: {text}"
        );
        assert_eq!(
            svg.contains(r#"class="blind""#),
            motion == trihex::Motion::Closeopen,
            "blinds must exist exactly when closeopen has windows to cover: {text}"
        );
        assert_eq!(
            svg.contains(r#"class="rain""#),
            scene.overlay.is_some(),
            "overlay dispatch is wrong: {text}"
        );
        n += 1;
    }
    assert_eq!(n, 42);
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
