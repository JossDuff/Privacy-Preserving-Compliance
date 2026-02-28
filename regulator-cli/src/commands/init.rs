use anyhow::{bail, Result};
use std::path::Path;

const NARGO_TOML_TEMPLATE: &str = r#"[package]
name = "{name}"
type = "bin"
authors = [""]

[dependencies]
"#;

const MAIN_NR_TEMPLATE: &str = r#"fn main(x: u64, y: pub u64) {
    assert(x != y);
}

#[test]
fn test_main() {
    main(1, 2);
}
"#;

pub async fn run(name: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        bail!("directory already exists: {name}");
    }

    std::fs::create_dir_all(project_dir.join("src"))?;

    std::fs::write(
        project_dir.join("Nargo.toml"),
        NARGO_TOML_TEMPLATE.replace("{name}", name),
    )?;

    std::fs::write(project_dir.join("src/main.nr"), MAIN_NR_TEMPLATE)?;

    println!("created compliance definition project: {name}/");

    Ok(())
}
