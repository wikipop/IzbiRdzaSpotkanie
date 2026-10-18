//! Project automation, run as `cargo xtask <command>` from `backend/`.

mod compose;
mod keys;

use std::path::PathBuf;

const USAGE: &str = "usage:
  cargo xtask gen-compose [--domain izbi.beer] [--expose-slots pn-09,sr-10]
  cargo xtask gen-keys [--kid k1] [--force]";

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((command, options)) = args.split_first() else {
        anyhow::bail!("{USAGE}");
    };
    match command.as_str() {
        "gen-compose" => compose::run(&backend_dir(), options),
        "gen-keys" => keys::run(&backend_dir(), options),
        other => anyhow::bail!("unknown command `{other}`\n{USAGE}"),
    }
}

/// The `backend/` directory, regardless of where `cargo xtask` is invoked.
fn backend_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives inside backend/")
        .to_path_buf()
}

/// Returns the value following `--name` in `options`, if present.
fn option<'a>(options: &'a [String], name: &str) -> anyhow::Result<Option<&'a str>> {
    match options.iter().position(|arg| arg == name) {
        Some(index) => options
            .get(index + 1)
            .map(|value| Some(value.as_str()))
            .ok_or_else(|| anyhow::anyhow!("{name} needs a value")),
        None => Ok(None),
    }
}

fn flag(options: &[String], name: &str) -> bool {
    options.iter().any(|arg| arg == name)
}
