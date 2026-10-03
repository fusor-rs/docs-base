use docs_base_build::{Config, Highlighter};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let guides = docs_base_build::compile(
        &Config {
            root: Path::new("."),
            content: Path::new("docs"),
            navigation: Path::new("navigation.json"),
            references: None,
            base_path: "/manual/",
            repository: Some("https://github.com/fusor-rs/docs-base/blob/main/examples/book/"),
        },
        &Highlighter::default(),
    )?;
    guides.write_assets(Path::new("public"))?;
    fs::write(
        PathBuf::from(env::var("OUT_DIR")?).join("guides.rs"),
        guides.source.to_string(),
    )?;
    fusor_build::compile_app()?;
    Ok(())
}
