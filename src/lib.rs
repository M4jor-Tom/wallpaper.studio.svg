//! Animated SVG background builder — trihexagonal blueprint.

pub mod geom;
pub mod icon;
pub mod matrix;
pub mod params;
pub mod rng;
pub mod style;
pub mod svg;
pub mod trihex;

use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Schema(serde_json::Error),
    Invalid(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{e}"),
            Error::Schema(e) => write!(f, "{e}"),
            Error::Invalid(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Schema(e)
    }
}

pub fn load(json: &str) -> Result<(params::Parameters, params::Scene), Error> {
    let p = params::parse(json)?;
    params::validate(&p)?;
    let scene = params::resolve(&p);
    Ok((p, scene))
}

/// Rejects zero sizes, which `Lattice::new` would divide by; the CLI path gets the same check from `parse_res`.
pub fn render_to_string(json: &str, w: u32, h: u32) -> Result<String, Error> {
    if w == 0 || h == 0 {
        return Err(Error::Invalid("width and height must be non-zero".into()));
    }
    let (_, scene) = load(json)?;
    Ok(svg::build_svg(w, h, &scene))
}

fn render(text: &str) -> Result<(), Error> {
    let (p, scene) = load(text)?;
    let sink = p.output.as_ref().and_then(|o| o.sink.as_ref());

    if let Some(params::output::Sink::Stdout(s)) = sink {
        let (w, h) = params::parse_res(&s.resolution)?;
        print!("{}", svg::build_svg(w, h, &scene));
        return Ok(());
    }
    if let Some(params::output::Sink::File(f)) = sink {
        let (w, h) = params::parse_res(&f.resolution)?;
        std::fs::write(&f.path, svg::build_svg(w, h, &scene))?;
        println!("{}", f.path);
        return Ok(());
    }

    let (dir, resolutions): (&str, &[String]) = match sink {
        Some(params::output::Sink::Directory(d)) => (&d.path, &d.resolutions),
        _ => ("", &[]),
    };
    let dir = if dir.is_empty() { "out" } else { dir };
    std::fs::create_dir_all(dir)?;
    let sizes: Vec<(u32, u32)> = if resolutions.is_empty() {
        vec![params::parse_res("")?]
    } else {
        resolutions
            .iter()
            .map(|r| params::parse_res(r))
            .collect::<Result<_, _>>()?
    };
    for (w, h) in sizes {
        let path = format!("{dir}/trihex-{}-{w}x{h}.svg", scene.slug());
        std::fs::write(&path, svg::build_svg(w, h, &scene))?;
        println!("{path}");
    }
    Ok(())
}

pub fn run(args: &[String]) -> i32 {
    let (label, text) = match args {
        [] => ("(defaults)".to_string(), Ok("{}".to_string())),
        [a] if a == "--configs" => {
            for c in params::valid_configs(0) {
                println!(
                    "{}",
                    serde_json::to_string(&c).expect("a config must serialise")
                );
            }
            return 0;
        }
        [a] if a == "--descriptor" => {
            use std::io::Write;
            return match std::io::stdout().write_all(params::DESCRIPTOR) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("--descriptor: {e}");
                    2
                }
            };
        }
        [a] => (a.clone(), std::fs::read_to_string(a).map_err(Error::Io)),
        _ => {
            eprintln!("usage: bgsvg [config.json | --configs | --descriptor]");
            return 2;
        }
    };
    match text.and_then(|t| render(&t)) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{label}: {e}");
            2
        }
    }
}
