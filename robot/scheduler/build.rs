use std::io::Result;

fn main() -> Result<()> {
    // Compile protocol buffers
    prost_build::Config::new().compile_protos(
        &[
            "../../robot/proto/common.proto",
            "../../robot/proto/scheduler.proto",
            "../../robot/proto/vision.proto",
            "../../robot/proto/audio.proto",
            "../../robot/proto/serial.proto",
            "../../robot/proto/network.proto",
        ],
        &["../../robot/proto/"],
    )?;

    Ok(())
}
