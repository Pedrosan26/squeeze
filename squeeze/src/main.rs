mod compression;

use std::io::{self,Read,Write};
use zip::CompressionMethod;

fn main() {
    let mut source_file = String::new();
    let mut zip_file = String::new();
    let mut compression_choice = String::new();

    println!("++++++");

    println!("Enter the path of the source file to compress:");
    io::stdin().read_line(&mut source_file).expect("Failed to read source file path");
    let source_file = source_file.trim();

    println!("Enter the destination path for the ZIP file:");
    io::stdin().read_line(&mut zip_file).expect("Failed to read destination file path");
    let zip_file = zip_file.trim();

    println!("Choose a compression method:");
    println!("1) Deflated  2) Stored  3) Bzip2  4) Zstd");
    io::stdin().read_line(&mut compression_choice).expect("Failed to read compression choice");
    let compression_choice = compression_choice.trim();

    let compression_method = match compression_choice {
        "1" => CompressionMethod::Deflated,
        "2" => CompressionMethod::Stored,
        "3" => CompressionMethod::Bzip2,
        "4" => CompressionMethod::Zstd,
        _ => {
            eprintln!("Invalid choice, defaulting to Deflated.");
            CompressionMethod::Deflated
        }
    };

    match compression::compression(source_file, zip_file, compression_method) {
        Ok(_) => println!("Compression successful."),
        Err(e) => eprintln!("An error occurred: {}", e),
    }
}
