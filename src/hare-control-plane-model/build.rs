fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize)]")
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&["../../proto/hare/control_plane.proto"], &["../../proto/hare"])?;
    Ok(())
}
