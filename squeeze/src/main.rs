mod compression;

use std::env;
use std::path::PathBuf;
use std::process;
use zip::CompressionMethod;

fn print_usage() {
    println!(
        "Usage: squeeze <file1> <file2> ... -o <output.zip> \
         [--method deflate|stored|zstd]"
    );
}

fn parse_method(value: &str) -> Option<CompressionMethod> {
    match value {
        "deflate" => Some(CompressionMethod::Deflated),
        "stored" => Some(CompressionMethod::Stored),
        "zstd" => Some(CompressionMethod::Zstd),
        _ => None,
    }
}

fn main() {
    let mut args = env::args().skip(1).peekable();

    let mut source_files: Vec<PathBuf> = Vec::new();
    let mut output_file: Option<PathBuf> = None;
    let mut compression_method = CompressionMethod::Deflated;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                return;
            }

            "-o" | "--output" => {
                let Some(value) = args.next() else {
                    eprintln!("Error: missing output ZIP path.");
                    process::exit(2);
                };

                output_file = Some(PathBuf::from(value));
            }

            "--method" => {
                let Some(value) = args.next() else {
                    eprintln!("Error: missing compression method.");
                    process::exit(2);
                };

                let Some(method) = parse_method(&value) else {
                    eprintln!("Error: unknown compression method '{}'.", value);
                    print_usage();
                    process::exit(2);
                };

                compression_method = method;
            }

            _ if arg.starts_with('-') => {
                eprintln!("Error: unknown option '{}'.", arg);
                print_usage();
                process::exit(2);
            }

            _ => source_files.push(PathBuf::from(arg)),
        }
    }

    if source_files.is_empty() {
        eprintln!("Error: provide at least one source file.");
        print_usage();
        process::exit(2);
    }

    let Some(output_file) = output_file else {
        eprintln!("Error: specify an output ZIP with -o.");
        print_usage();
        process::exit(2);
    };

    // Prevent accidentally overwriting one of the input files.
    if output_file.exists() {
        let output_canonical = output_file.canonicalize().ok();

        if let Some(output_canonical) = output_canonical {
            for source in &source_files {
                if source.canonicalize().ok().as_ref()
                    == Some(&output_canonical)
                {
                    eprintln!(
                        "Error: output ZIP cannot overwrite input '{}'.",
                        source.display()
                    );
                    process::exit(2);
                }
            }
        }
    }

    println!("Creating ZIP: {}", output_file.display());

    match compression::compression(
        &source_files,
        &output_file,
        compression_method,
    ) {
        Ok(report) => {
            println!("\nCompression finished.");
            println!("Files added: {}", report.added.len());

            for file in &report.added {
                println!("  + {}", file.display());
            }

            if !report.errors.is_empty() {
                eprintln!("\nFiles that could not be added:");

                for (file, error) in &report.errors {
                    eprintln!("  - {}: {}", file.display(), error);
                }

                process::exit(1);
            }

            println!("ZIP created successfully!");
        }

        Err(error) => {
            eprintln!("Fatal ZIP error: {}", error);
            process::exit(1);
        }
    }
}