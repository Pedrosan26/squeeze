
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
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
        // Read each source before adding its entry.
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

        if !entry_names.insert(entry_name.clone()) {
            report.errors.push((
                source_path.clone(),
                format!("Duplicate filename inside ZIP: {}", entry_name),
            ));
            continue;
        }

        let options = FileOptions::default()
            .compression_method(compression_method)
            .unix_permissions(0o644);

        // Errors writing the archive are fatal.
        zip.start_file(&entry_name, options)?;
        std::io::copy(&mut source, &mut zip)?;

        report.added.push(source_path.clone());
    }

    // Finalize the archive, even if some input files failed.
    zip.finish()?;

    Ok(report)
}
