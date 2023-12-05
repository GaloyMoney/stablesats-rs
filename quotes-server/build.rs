fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("PROTOC", protobuf_src::protoc());

    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .compile(&["../proto/quotes/quote_service.proto"], &["../proto"])?;
    Ok(())
}
