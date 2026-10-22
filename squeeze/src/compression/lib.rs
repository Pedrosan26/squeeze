use super::*;
use std::fs;
use std::io::Read;
use zip::ZipArchive;

const ALL_METHODS: [CompressionMethod; 3] = [
    CompressionMethod::Stored,
    CompressionMethod::Deflated,
    CompressionMethod::Zstd,
];

/// Directorio temporal único que se borra al salir de scope.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir()
            .join(format!("squeeze_test_{}_{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Datos deterministas: mitad texto repetitivo, mitad pseudoaleatorio.
/// `len` > 8 KB para ejercitar el streaming con varios bloques.
fn sample_data(len: usize, seed: u64) -> Vec<u8> {
    let mut data = Vec::with_capacity(len);
    let mut state = seed;
    while data.len() < len {
        if data.len() < len / 2 {
            data.extend_from_slice(b"linea de log repetitiva user=42 status=200\n");
        } else {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            data.push((state >> 33) as u8);
        }
    }
    data.truncate(len);
    data
}

fn write_file(dir: &TempDir, name: &str, data: &[u8]) -> PathBuf {
    let path = dir.path(name);
    fs::write(&path, data).unwrap();
    path
}

fn read_entry(zip_path: &Path, name: &str) -> (Vec<u8>, CompressionMethod, u64) {
    let mut archive = ZipArchive::new(File::open(zip_path).unwrap()).unwrap();
    let mut entry = archive.by_name(name).unwrap();
    let method = entry.compression();
    let compressed_size = entry.compressed_size();
    let mut content = Vec::new();
    entry.read_to_end(&mut content).unwrap();
    (content, method, compressed_size)
}

#[test]
fn roundtrip_with_every_method() {
    let dir = TempDir::new("roundtrip");
    // 1 MB: mucho mayor que el buffer de io::copy (8 KB).
    let data = sample_data(1024 * 1024, 7);
    let source = write_file(&dir, "datos.bin", &data);

    for method in ALL_METHODS {
        let zip_path = dir.path(&format!("out_{:?}.zip", method));
        let report = compression(&[source.clone()], &zip_path, method).unwrap();

        assert_eq!(report.added, vec![source.clone()], "{:?}", method);
        assert!(report.errors.is_empty(), "{:?}", method);

        let (content, used_method, _) = read_entry(&zip_path, "datos.bin");
        assert_eq!(used_method, method, "método registrado en el ZIP");
        // Regresión: con streaming mal hecho las entradas quedaban vacías.
        assert_eq!(content.len(), data.len(), "{:?}", method);
        assert_eq!(content, data, "{:?}", method);
    }
}

#[test]
fn stored_does_not_compress_but_others_do() {
    let dir = TempDir::new("sizes");
    let data = vec![b'a'; 512 * 1024]; // altamente compresible
    let source = write_file(&dir, "ceros.txt", &data);

    for method in ALL_METHODS {
        let zip_path = dir.path(&format!("out_{:?}.zip", method));
        compression(&[source.clone()], &zip_path, method).unwrap();
        let (_, _, compressed) = read_entry(&zip_path, "ceros.txt");

        if method == CompressionMethod::Stored {
            assert_eq!(compressed, data.len() as u64);
        } else {
            assert!(
                compressed < data.len() as u64 / 10,
                "{:?} debería comprimir mucho (quedó en {} bytes)",
                method,
                compressed
            );
        }
    }
}

#[test]
fn multiple_files_keep_names_and_contents() {
    let dir = TempDir::new("multi");
    let files: Vec<(String, Vec<u8>)> = (0..5)
        .map(|i| (format!("f{}.dat", i), sample_data(20_000 + i * 1000, i as u64)))
        .collect();
    let paths: Vec<PathBuf> = files
        .iter()
        .map(|(name, data)| write_file(&dir, name, data))
        .collect();

    for method in ALL_METHODS {
        let zip_path = dir.path(&format!("multi_{:?}.zip", method));
        let report = compression(&paths, &zip_path, method).unwrap();
        assert_eq!(report.added.len(), 5);

        for (name, data) in &files {
            let (content, _, _) = read_entry(&zip_path, name);
            assert_eq!(&content, data, "{} con {:?}", name, method);
        }
    }
}

#[test]
fn empty_file_is_supported() {
    let dir = TempDir::new("empty");
    let source = write_file(&dir, "vacio.txt", b"");

    for method in ALL_METHODS {
        let zip_path = dir.path(&format!("empty_{:?}.zip", method));
        let report = compression(&[source.clone()], &zip_path, method).unwrap();
        assert!(report.errors.is_empty(), "{:?}", method);

        let (content, _, _) = read_entry(&zip_path, "vacio.txt");
        assert!(content.is_empty(), "{:?}", method);
    }
}

#[test]
fn missing_file_is_reported_and_does_not_stop_the_rest() {
    let dir = TempDir::new("missing");
    let ok = write_file(&dir, "ok.txt", b"hola");
    let missing = dir.path("no_existe.txt");
    let zip_path = dir.path("out.zip");

    let report =
        compression(&[missing.clone(), ok.clone()], &zip_path, CompressionMethod::Deflated)
            .unwrap();

    assert_eq!(report.added, vec![ok]);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.errors[0].0, missing);
    assert!(report.errors[0].1.contains("Could not open file"));

    // El ZIP final es válido y contiene solo el archivo bueno.
    let archive = ZipArchive::new(File::open(&zip_path).unwrap()).unwrap();
    assert_eq!(archive.len(), 1);
}

#[test]
fn duplicate_filenames_are_rejected() {
    let dir = TempDir::new("dups");
    fs::create_dir_all(dir.path("a")).unwrap();
    fs::create_dir_all(dir.path("b")).unwrap();
    let first = write_file(&dir, "a/mismo.txt", b"primero");
    let second = write_file(&dir, "b/mismo.txt", b"segundo");
    let zip_path = dir.path("out.zip");

    let report =
        compression(&[first.clone(), second.clone()], &zip_path, CompressionMethod::Stored)
            .unwrap();

    assert_eq!(report.added, vec![first]);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.errors[0].0, second);
    assert!(report.errors[0].1.contains("Duplicate filename"));

    let (content, _, _) = read_entry(&zip_path, "mismo.txt");
    assert_eq!(content, b"primero");
}