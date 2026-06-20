# lewrep2 🏎️💨

`lewrep2` is a high-performance, cross-platform CLI text search utility built in Rust that recursively scans directories for patterns with absolute minimal overhead. By leveraging the industry-proven, production-grade ecosystem that powers ripgrep (`grep-searcher`, `grep-regex`, and `ignore`, and my own open source colour crate `lewcolour`), `lewrep2` drops traditional initialization bloat to deliver near-instantaneous search results.

Engineered to seamlessly handle complex pipelines, it fully supports standard UNIX piping constraints, allowing you to fluidly pass data into and out of core system utilities like `grep`, `ripgrep`, `tree`, `cat`, `ls`, and more.

## 📊 Performance Showdown (.txt Scan Rematch)

Measured using high-precision execution telemetry (`lewtime`) on an Apple Silicon architecture:

| Metric | lewrep2 | ripgrep (rg) | The Verdict |
| :--- | :--- | :--- | :--- |
| **Total Process Time** | 0.052s | 0.068s | `lewrep2` is ~23% Faster on micro-scans |
| **Memory Footprint** | 2.56 MB | 6.09 MB | `lewrep2` uses less than half the RAM |

## Why is it so fast?

* **Production-Grade Engine Room:** Built directly on top of the `grep-searcher` and `grep-regex` crates—the exact same underlying hardware-accelerated libraries developed to power `ripgrep`.
* **Zero-Copy Architecture Available:** Utilizes optimized buffer strategies and optional native OS memory mapping (`Mmap`) to stream file chunks efficiently.
* **Smart Traversal:** Blazes through file structures using the `ignore` crate to automatically respect `.gitignore` rules, concurrently driven by a parallel `rayon` thread pool.

* lewrep2 uses a custom zero‑overhead colour engine (lewcolour) designed specifically     for high‑performance terminal output.

## 🚀 Features & Flags

* `lewrep2 <pattern> [path]` - Standard blazing fast search.
* `-i`, `--ignore-case` - Case-insensitive matching.
* `-n`, `--line-number` - Displays specific line number locations for hits.
* `-v`, `--invert-match` - Inverts the search (selects non-matching lines).
* `-l` - Filenames-only short-circuit mode (exits the millisecond a match is found in a file).
* `-I` - Explicit ignore configurations / overrides.
* `-A` - Context control (displays lines trailing after a match).
* `-d` - Deletes orange ANSI colour for text and replaces it with white text (Grep format).
* `-c` - Counts only the amount of times a certain word is mentioned in text.
* `-u` - restricted mode can scan `.gitignore`, binaries, hidden files.
* `-T` - shows tree input for your text searches (exclusive to lewrep2).
* `-X` - explains the search (uses its own regex engine) (exclusive to lewrep2).
* `-h` - removes the filename of your search.
* `--Hide` - hides `.gitignore`, `binary`, and hidden files (opposite flag to `-u`).
* `--vscode` - searches text via `.vscode`. 

## 🐛 Bug Reports

If there is a bug in the code that I've missed or that you notice while running your own benchmarks, please let me know in the **GitHub Bug Reports / Issues** tab immediately so I can patch it!

## 💡 Feedback & Contributions

Got feature requests, optimizations, or ideas to make `lewrep2` even faster? Feel free to open a GitHub Issue or submit a Pull Request. Open-source feedback is highly encouraged!

## 📄 License

This project is licensed under the MIT License - see the `LICENSE` file for details.

Developed with 🦀 by **xlewis1**.

## P.S
this program uses grep_regex, grep-searcher, walkbuilder, ignore which all belong to burntsushi

## 🛠️ Installation & Building

Since `lewrep2` is 100% cross-platform at the source level, you can compile it natively for macOS, Linux, or Windows.

```bash
# Clone the repository
git clone [https://github.com/xlewis1/LewRep2.git](https://github.com/xlewis1/LewRep2.git)
cd LewRep2

# Build a hyper-optimized release binary
cargo build --release
or use RUSTFLAGS="-C target-cpu=native" cargo build --release for maximum power. 
