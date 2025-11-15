use std::io::Result;

fn main() -> Result<()> {
    prost_build::Config::new().compile_protos(
        &[
            "../../robot/proto/common.proto",
            "../../robot/proto/network.proto",
        ],
        &["../../robot/proto/"],
    )?;
    Ok(())
}
