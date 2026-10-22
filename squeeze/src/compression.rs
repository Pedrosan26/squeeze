use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

pub struct CompressionReport {
    pub added: Vec<PathBuf>,
    pub errors: Vec<(PathBuf, String)>,
}

pub fn compression(
    source_files: &[PathBuf],
    zip_file: &Path,
    compression_method: CompressionMethod,
) -> Result<CompressionReport, Box<dyn Error>> {
    let zip_file_handle = File::create(zip_file)?;
    let mut zip = ZipWriter::new(zip_file_handle);

    let mut report = CompressionReport {
        added: Vec::new(),
        errors: Vec::new(),
    };

    // ZIP entry names must be unique for our CLI.
    let mut entry_names = HashSet::new();

    for source_path in source_files {
        // Store only the filename inside the ZIP.
        let entry_name = match source_path.file_name() {
            Some(name) => name.to_string_lossy().into_owned(),
            None => {
                report.errors.push((
                    source_path.clone(),
                    "Invalid filename".to_string(),
                ));
                continue;
            }
        };

        if entry_names.contains(&entry_name) {
            report.errors.push((
                source_path.clone(),
                format!("Duplicate filename inside ZIP: {}", entry_name),
            ));
            continue;
        }

        // Open (and size) the source BEFORE creating its entry, so that
        // failures here are non-fatal and leave the archive untouched.
        let mut source = match File::open(source_path) {
            Ok(file) => file,
            Err(error) => {
                report.errors.push((
                    source_path.clone(),
                    format!("Could not open file: {}", error),
                ));
                continue;
            }
        };

        let size = match source.metadata() {
            Ok(metadata) => metadata.len(),
            Err(error) => {
                report.errors.push((
                    source_path.clone(),
                    format!("Could not read metadata: {}", error),
                ));
                continue;
            }
        };

        entry_names.insert(entry_name.clone());

        let options = FileOptions::default()
            .compression_method(compression_method)
            .unix_permissions(0o644)
            // Entries of 4 GiB or more require ZIP64.
            .large_file(size >= u32::MAX as u64);

        // From here on, any error leaves a half-written entry in the
        // archive, so it is fatal (same policy as write errors).
        zip.start_file(&entry_name, options)?;
        io::copy(&mut source, &mut zip)?;

        report.added.push(source_path.clone());
    }

    // Finalize the archive, even if some input files failed.
    zip.finish()?;

    Ok(report)
}

#[cfg(test)]
mod lib;