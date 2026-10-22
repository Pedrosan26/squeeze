# squeeze

A command-line tool written in Rust that compresses files into a ZIP archive. It streams each file into the archive, so memory usage stays flat regardless of file size, and it is benchmarked against the macOS `zip` command.

## Usage

```bash
squeeze <file1> <file2> ... -o <output.zip> [--method deflate|stored|zstd]
```

| Method | Description |
|---|---|
| `deflate` (default) | Standard ZIP compression. Opens everywhere. |
| `stored` | No compression, files are just packed into the archive. Opens everywhere. |
| `zstd` | Much faster than deflate and often much smaller, but ZIP archives using zstd require extractor support, so compatibility is not universal. |

Behavior:

- Only the file name is stored inside the ZIP, not the path.
- Duplicate names, files that cannot be opened and invalid names are reported without stopping the run. The ZIP is still created with the valid files, and the program exits with code 1.
- A read or write error in the middle of a file is fatal.
- The output ZIP is not allowed to overwrite one of the input files.


## Benchmark

### Dataset

The dataset is **synthetic**: every file was artificially generated (3.3 GB in total) to cover different levels of compressibility. No real-world files were used.

| Type | Files | Content |
|---|---|---|
| `random_*.bin` | 1, 10, 100 and 1024 MB | Random bytes from `/dev/urandom`. Incompressible, the worst case. |
| `log_*.txt` | 1, 10, 100 and 1024 MB | Simulated log lines (timestamp, level, user, action, path, latency, status). Highly compressible. |
| `data_*.csv` | 1, 10, 100 and 1024 MB | CSV with header `id,name,category,price,qty,date`. Medium compressibility. |
| `many_small/` | 2,000 files of 10 KB | Base64 text of random bytes. Measures per-file overhead. |

The `log` and `csv` files are generated with seed `42` (reproducible) by sampling lines at random from a pool of 50,000 lines.

### Methodology

- **Comparison:** `squeeze <files> -o out.zip --method <m>` against `zip -q <flags> -j out.zip <files>`, with `-0` for stored and `-6` for deflate (the default level of both tools). macOS `zip` does not support zstd, so zstd is measured for squeeze only.
- **Build:** `cargo build --release`.
- **Runs:** 5 per case, all `zip` runs first and then all `squeeze` runs. Reported value is the median of the wall-clock time. Memory is the peak resident set size across runs, measured with `/usr/bin/time -l`.
- **Environment:** a MacBook Air with other applications open (normal use, not an isolated machine).
- **Integrity check:** every archive is checked with `unzip -t`, and the sum of uncompressed sizes in the listing must match the sum of the input file sizes. For zstd only the listing is checked, because the system `unzip` cannot decompress it. zstd round trips are instead covered by the unit tests, which read the archives back with the `zip` crate.

### Results: stored

All results come from the artificially generated dataset described above. Time in seconds (median of 5), archive size in bytes, peak memory in MB.

| File | zip (s) | squeeze (s) | zip size | squeeze size | RAM zip / squeeze |
|---|---:|---:|---:|---:|---:|
| random_1MB | 0.00 | 0.00 | 1,048,754 | 1,048,702 | 2 / 1 |
| random_10MB | 0.03 | 0.00 | 10,485,940 | 10,485,888 | 2 / 1 |
| random_100MB | 0.55 | 0.16 | 104,857,782 | 104,857,730 | 2 / 1 |
| random_1024MB | 5.07 | 1.85 | 1,073,742,008 | 1,073,741,956 | 2 / 1 |
| log_1MB | 0.00 | 0.00 | 1,048,748 | 1,048,696 | 2 / 1 |
| log_10MB | 0.03 | 0.00 | 10,485,934 | 10,485,882 | 2 / 1 |
| log_100MB | 0.28 | 0.09 | 104,857,776 | 104,857,724 | 2 / 1 |
| log_1024MB | 2.82 | 1.03 | 1,073,742,002 | 1,073,741,950 | 2 / 2 |
| csv_1MB | 0.00 | 0.00 | 1,048,750 | 1,048,698 | 2 / 1 |
| csv_10MB | 0.03 | 0.00 | 10,485,936 | 10,485,884 | 2 / 1 |
| csv_100MB | 0.29 | 0.12 | 104,857,778 | 104,857,726 | 2 / 1 |
| csv_1024MB | 2.96 | 0.89 | 1,073,742,004 | 1,073,741,952 | 2 / 1 |
| many_small (2,000 files) | 0.09 | 0.11 | 20,773,808 | 20,669,808 | 3 / 3 |

Stored archives from `zip` are 52 bytes larger per entry (2,000 files: exactly 104,000 bytes more).
### Results: deflate

| File | zip (s) | squeeze (s) | zip size | squeeze size | RAM zip / squeeze |
|---|---:|---:|---:|---:|---:|
| random_1MB | 0.01 | 0.01 | 1,048,914 | 1,048,872 | 2 / 2 |
| random_10MB | 0.16 | 0.16 | 10,487,540 | 10,487,543 | 2 / 2 |
| random_100MB | 2.76 | 2.77 | 104,874,457 | 104,874,250 | 2 / 2 |
| random_1024MB | 16.80 | 16.90 | 1,073,915,710 | 1,073,911,076 | 2 / 2 |
| log_1MB | 0.01 | 0.01 | 165,785 | 159,287 | 2 / 2 |
| log_10MB | 0.12 | 0.12 | 1,649,309 | 1,585,174 | 2 / 2 |
| log_100MB | 1.19 | 1.19 | 16,474,341 | 15,838,895 | 2 / 2 |
| log_1024MB | 12.23 | 12.25 | 168,664,260 | 162,159,067 | 2 / 2 |
| csv_1MB | 0.02 | 0.02 | 326,556 | 326,151 | 2 / 2 |
| csv_10MB | 0.27 | 0.24 | 3,256,357 | 3,252,529 | 2 / 2 |
| csv_100MB | 2.75 | 2.39 | 32,546,520 | 32,507,341 | 2 / 2 |
| csv_1024MB | 28.74 | 24.91 | 333,256,763 | 332,865,864 | 2 / 2 |
| many_small (2,000 files) | 0.29 | 0.30 | 15,806,301 | 15,745,911 | 3 / 6 |

### Results: zstd (squeeze only)

| File | Time (s) | Size | RAM (MB) |
|---|---:|---:|---:|
| random_1MB | 0.00 | 1,048,735 | 4 |
| random_10MB | 0.00 | 10,486,137 | 5 |
| random_100MB | 0.11 | 104,860,139 | 5 |
| random_1024MB | 0.93 | 1,073,766,541 | 5 |
| log_1MB | 0.00 | 180,182 | 4 |
| log_10MB | 0.02 | 1,622,212 | 5 |
| log_100MB | 0.19 | 15,992,974 | 5 |
| log_1024MB | 2.04 | 163,423,793 | 5 |
| csv_1MB | 0.00 | 304,190 | 4 |
| csv_10MB | 0.03 | 2,494,867 | 5 |
| csv_100MB | 0.28 | 24,248,744 | 5 |
| csv_1024MB | 3.00 | 247,514,129 | 5 |
| many_small (2,000 files) | 0.19 | 15,637,808 | 8 |

### Summary

| Data | stored | deflate (vs `zip -6`) | zstd (vs squeeze deflate) |
|---|---|---|---|
| `random` | squeeze ~2.7 to 3.4x faster | Tie | ~18x faster on 1 GB, size equal |
| `log` | squeeze ~2.7 to 3.1x faster | Tie on speed, squeeze ~3.9% smaller | ~6x faster on 1 GB, ~0.8% larger |
| `csv` | squeeze ~2.4 to 3.3x faster on files of 100 MB and up | squeeze ~11 to 13% faster, ~0.1% smaller | ~8x faster on 1 GB, ~26% smaller |
| `many_small` | Tie | Tie | ~1.6x faster, ~0.7% smaller |

Run-to-run spread was narrow in almost every case. The exceptions were occasional `zip` outliers, for example 16.39 to 20.74 s on `random_1024MB` with deflate.

### Limitations

- **Synthetic data:** `log` and `csv` come from a pool of only 50,000 lines, so they are more repetitive than real data (the log file compresses almost 10:1). The comparison between tools is valid, but the absolute ratios and speeds may differ on real data.
- **Non-isolated environment:** the runs were made with other apps open on a fanless MacBook Air, which can throttle under long loads. Absolute times are not comparable across machines or sessions.
- **No zstd baseline:** macOS `zip` does not support zstd, so the zstd rows are compared against squeeze's own deflate, not against another tool.
- **zstd compatibility:** the zstd integrity check at benchmark time only covers the archive listing. Extraction was verified through the unit tests, not with a system tool.
- **Unexplained differences:** the size advantage on logs and the speed advantage on CSV and stored are probably due to different implementations, but this was not measured.

## Tests

```bash
cargo test
```

Six unit tests cover a round trip with every method on a 1 MB file (larger than the copy buffer), compression sizes, multiple files, an empty file, a missing file (non-fatal) and duplicate names. A regression test catches the empty-entry bug that appears if a file is read fully before being streamed.

## Build

```bash
cargo build --release
```
