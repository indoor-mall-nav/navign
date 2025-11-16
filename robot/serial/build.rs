use std::io::Result;

fn main() -> Result<()> {
    // Compile protocol buffers
    prost_build::Config::new().compile_protos(
        &[
            "../../robot/proto/common.proto",
            "../../robot/proto/serial.proto",
        ],
        &["../../robot/proto/"],
    )?;

    Ok(())
}
