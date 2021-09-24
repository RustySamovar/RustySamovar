use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["protobuf/packet_header.proto"], &["protobuf/"])?;
    Ok(())
}
