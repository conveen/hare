fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(false)
        .build_server(false)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["../../proto/hare/pagination.proto"], &["../../proto/hare"])?;
    Ok(())
}
