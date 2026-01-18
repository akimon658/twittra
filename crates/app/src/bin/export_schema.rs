use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_, openapi) = app::setup_openapi_routes();

    fs::write("api/openapi.json", openapi.to_pretty_json()?)?;

    Ok(())
}
