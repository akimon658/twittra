use std::fs;

use anyhow::Result;

fn main() -> Result<()> {
    let (_, openapi) = app::create_app();

    fs::write("openapi.json", openapi.to_pretty_json()?)?;

    Ok(())
}
