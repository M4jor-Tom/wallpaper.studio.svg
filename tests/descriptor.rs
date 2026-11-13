use bgsvg::params::DESCRIPTOR;

#[test]
fn the_descriptor_carries_the_whole_schema() {
    assert!(!DESCRIPTOR.is_empty(), "build.rs must embed descriptor.bin");

    let s = String::from_utf8_lossy(DESCRIPTOR);
    for name in [
        "svg_builder",
        "Parameters",
        "Output",
        "Background",
        "Icon",
        "Hexatri",
        "Ship",
        "Overlay",
        "Matrix",
        "CLOSEOPEN",
        "STARFIELD",
        "ROTATE",
    ] {
        assert!(s.contains(name), "descriptor does not mention {name}");
    }
}
