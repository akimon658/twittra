use std::fs;

use anyhow::Result;

fn main() -> Result<()> {
    let (_, openapi) = app::setup_openapi_routes()?;

    fs::write("api/openapi.json", openapi.to_pretty_json()?)?;

    Ok(())
}
