# squeeze

A command-line tool written in Rust that compresses files into a ZIP archive. It streams each file into the archive, so memory usage stays flat regardless of file size. It is benchmarked against the macOS `zip` command (stored and deflate) and the standalone `zstd` command-line tool (zstd).

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

- **stored and deflate:** `squeeze <files> -o out.zip --method <m>` against `zip -q <flags> -j out.zip <files>`, with `-0` for stored and `-6` for deflate (the default level of both tools).
- **zstd:** macOS `zip` does not support zstd, so squeeze is compared against the standalone `zstd` CLI (`zstd -q -3 -T1 -f <file> -o out.zst`). Level 3 is the default of both the CLI and the `zip` crate, and `-T1` keeps the CLI single-threaded. This comparison covers single files only (not `many_small`). The CLI writes a `.zst` file rather than a ZIP, so squeeze's sizes include the ZIP container overhead (about 120 bytes per file).
- **Build:** `cargo build --release`.
- **Runs:** 5 per case, all runs of one tool and then all runs of the other. Reported value is the median of the wall-clock time. Memory is the peak resident set size across runs, measured with `/usr/bin/time -l`.
- **Environment:** a MacBook Air with other applications open (normal use, not an isolated machine).
- **Integrity check:** every ZIP is checked with `unzip -t`, and the sum of uncompressed sizes in the listing must match the sum of the input file sizes. For zstd inside a ZIP only the listing is checked, because the system `unzip` cannot decompress it; zstd round trips are instead covered by the unit tests, which read the archives back with the `zip` crate. Files produced by the `zstd` CLI are checked with `zstd -t`.

### Results: stored

All results come from the artificially generated dataset described above. Time in seconds (median of 5), archive size in bytes, peak memory in MB.

| File | zip (s) | squeeze (s) | zip size | squeeze size | RAM zip / squeeze |
|---|---:|---:|---:|---:|---:|
| random_1MB | 0.00 | 0.00 | 1,048,754 | 1,048,702 | 2 / 1 |
| random_10MB | 0.03 | 0.00 | 10,485,940 | 10,485,888 | 2 / 1 |
| random_100MB | 0.32 | 0.13 | 104,857,782 | 104,857,730 | 2 / 1 |
| random_1024MB | 2.84 | 1.46 | 1,073,742,008 | 1,073,741,956 | 2 / 2 |
| log_1MB | 0.00 | 0.00 | 1,048,748 | 1,048,696 | 2 / 1 |
| log_10MB | 0.04 | 0.01 | 10,485,934 | 10,485,882 | 2 / 1 |
| log_100MB | 0.28 | 0.07 | 104,857,776 | 104,857,724 | 2 / 1 |
| log_1024MB | 3.05 | 1.30 | 1,073,742,002 | 1,073,741,950 | 2 / 1 |
| csv_1MB | 0.00 | 0.00 | 1,048,750 | 1,048,698 | 2 / 1 |
| csv_10MB | 0.03 | 0.00 | 10,485,936 | 10,485,884 | 2 / 1 |
| csv_100MB | 0.32 | 0.17 | 104,857,778 | 104,857,726 | 2 / 1 |
| csv_1024MB | 2.86 | 1.12 | 1,073,742,004 | 1,073,741,952 | 2 / 2 |
| many_small (2,000 files) | 0.09 | 0.10 | 20,773,808 | 20,669,808 | 3 / 3 |

Stored archives from `zip` are 52 bytes larger per entry (2,000 files: exactly 104,000 bytes more).

### Results: deflate

| File | zip (s) | squeeze (s) | zip size | squeeze size | RAM zip / squeeze |
|---|---:|---:|---:|---:|---:|
| random_1MB | 0.01 | 0.01 | 1,048,914 | 1,048,872 | 2 / 2 |
| random_10MB | 0.16 | 0.15 | 10,487,540 | 10,487,543 | 2 / 2 |
| random_100MB | 1.61 | 1.59 | 104,874,457 | 104,874,250 | 2 / 2 |
| random_1024MB | 16.61 | 16.82 | 1,073,915,710 | 1,073,911,076 | 2 / 2 |
| log_1MB | 0.01 | 0.01 | 165,785 | 159,287 | 2 / 2 |
| log_10MB | 0.12 | 0.12 | 1,649,309 | 1,585,174 | 2 / 2 |
| log_100MB | 1.19 | 1.19 | 16,474,341 | 15,838,895 | 2 / 2 |
| log_1024MB | 12.32 | 12.32 | 168,664,260 | 162,159,067 | 2 / 2 |
| csv_1MB | 0.02 | 0.02 | 326,556 | 326,151 | 2 / 2 |
| csv_10MB | 0.27 | 0.24 | 3,256,357 | 3,252,529 | 2 / 2 |
| csv_100MB | 2.72 | 2.38 | 32,546,520 | 32,507,341 | 2 / 2 |
| csv_1024MB | 28.01 | 24.63 | 333,256,763 | 332,865,864 | 2 / 2 |
| many_small (2,000 files) | 0.28 | 0.30 | 15,806,301 | 15,745,911 | 3 / 5 |

### Results: zstd (standalone `zstd` CLI vs squeeze)

| File | zstd CLI (s) | squeeze (s) | zstd CLI size | squeeze size | RAM CLI / squeeze |
|---|---:|---:|---:|---:|---:|
| random_1MB | 0.00 | 0.00 | 1,048,613 | 1,048,735 | 7 / 4 |
| random_10MB | 0.00 | 0.00 | 10,486,014 | 10,486,137 | 26 / 5 |
| random_100MB | 0.06 | 0.07 | 104,860,014 | 104,860,139 | 54 / 5 |
| random_1024MB | 0.79 | 1.27 | 1,073,766,414 | 1,073,766,541 | 55 / 5 |
| log_1MB | 0.00 | 0.00 | 180,066 | 180,182 | 5 / 4 |
| log_10MB | 0.02 | 0.02 | 1,622,036 | 1,622,212 | 16 / 5 |
| log_100MB | 0.16 | 0.19 | 15,993,159 | 15,992,974 | 39 / 5 |
| log_1024MB | 1.65 | 1.99 | 163,484,584 | 163,423,793 | 41 / 5 |
| csv_1MB | 0.00 | 0.00 | 304,072 | 304,190 | 6 / 4 |
| csv_10MB | 0.03 | 0.03 | 2,540,325 | 2,494,867 | 17 / 5 |
| csv_100MB | 0.25 | 0.28 | 24,773,991 | 24,248,744 | 40 / 5 |
| csv_1024MB | 2.67 | 2.97 | 253,150,128 | 247,514,129 | 42 / 5 |
| many_small (2,000 files) | n/a | 0.19 | n/a | 15,637,808 | n/a / 6 |

### Summary

| Data | stored (vs `zip -0`) | deflate (vs `zip -6`) | zstd (vs `zstd -3 -T1`) |
|---|---|---|---|
| `random` | squeeze ~2.0 to 2.5x faster on 100 MB and up | Tie | Time inconclusive (overlapping ranges), size equal |
| `log` | squeeze ~2.3 to 4.0x faster on 100 MB and up | Tie on speed, squeeze ~3.9% smaller | CLI ~19 to 21% faster, size equal |
| `csv` | squeeze ~1.9 to 2.6x faster on 100 MB and up | squeeze ~11 to 13% faster on 10 MB and up, ~0.1% smaller | CLI ~11 to 12% faster, squeeze ~1.8 to 2.2% smaller on 10 MB and up |
| `many_small` | Tie | Tie, squeeze ~0.4% smaller | No CLI baseline |
| Peak memory | squeeze 1 to 2 MB vs `zip` 2 MB | Both 2 MB | squeeze 5 MB vs CLI 39 to 55 MB on 100 MB and up (~8 to 11x less) |

Spread between runs was narrow for most deflate and zstd cases. The stored results and the 1 GB random zstd runs were noisier (for example, squeeze stored on `random_1024MB` ranged from 0.89 to 2.16 s, and the zstd CLI on the same file from 0.67 to 2.12 s).

### Limitations

- **Synthetic data:** `log` and `csv` come from a pool of only 50,000 lines, so they are more repetitive than real data (the log file compresses almost 10:1). The comparison between tools is valid, but the absolute ratios and speeds may differ on real data.
- **Non-isolated environment:** the runs were made with other apps open on a fanless MacBook Air, which can throttle under long loads. Absolute times are not comparable across machines or sessions.
- **Stored speed varies between runs:** an earlier run measured `zip` stored on `random_1024MB` at 5.07 s versus 2.84 s here, so the stored speed ratios should be read as approximate. The direction (squeeze faster) was the same in both runs.
- **zstd comparison scope:** the baseline is the standalone `zstd` CLI at level 3 on a single thread, on single files only. It writes `.zst` rather than ZIP, and its version was not recorded.
- **zstd inside a ZIP:** the benchmark integrity check only covers the archive listing. Extraction was verified through the unit tests, not with a system tool.
- **Unexplained differences:** the size advantage on logs, the speed advantage on CSV and stored, and the ~2% size advantage on CSV against the zstd CLI are probably due to different implementations or library versions, but this was not measured.

## Tests

```bash
cargo test
```

Six unit tests cover a round trip with every method on a 1 MB file (larger than the copy buffer), compression sizes, multiple files, an empty file, a missing file (non-fatal) and duplicate names. A regression test catches the empty-entry bug that appears if a file is read fully before being streamed.

## Build

```bash
cargo build --release
```
