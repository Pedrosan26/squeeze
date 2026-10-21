
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::CompressionMethod;
use zip::write::FileOptions;

pub fn compression(
    source_file: &str,
    zip_file: &str,
    compression_method: CompressionMethod,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting compression...");

    let mut source = File::open(source_file)?;
    let mut buffer = Vec::new();
    source.read_to_end(&mut buffer)?;

    let zip_path = Path::new(zip_file);
    let zip_file_handle = File::create(zip_path)?;

    let mut zip = zip::ZipWriter::new(zip_file_handle);

    let options = FileOptions::default()
        .compression_method(compression_method)
        .unix_permissions(0o644);

    zip.start_file(source_file, options)?;
    zip.write_all(&buffer)?;
    zip.finish()?;

    println!("Compression completed successfully!");
    println!(
        "File '{}' has been compressed to '{}'",
        source_file,
        zip_file
    );

    Ok(())
}
