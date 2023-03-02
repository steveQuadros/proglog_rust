use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["src/api/v1/log.proto"], &["src/"])?;
    Ok(())
}
