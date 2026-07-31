use spirv_builder::SpirvBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Targets defined in https://rust-gpu.github.io/rust-gpu/book/platform-support.html
    SpirvBuilder::new("shader", "spirv-unknown-spv1.6")
        // .spirv_metadata(MetadataPrintout::Full)
        .build()?;
    Ok(())
}