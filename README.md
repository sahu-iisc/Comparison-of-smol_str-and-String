# small

A lightweight memory profiling and benchmarking tool for evaluating **Small String Optimization (SSO)** libraries in Rust.

---

## Description

In many data processing and deserialization pipelines, dealing with numerous short strings (such as city names, state abbreviations, or postal codes) using the standard heap-allocated `String` incurs significant allocation overhead and memory fragmentation.

`small` demonstrates and profiles heap memory behavior across different string implementations in Rust:
- **`std::string::String`** — Standard 24-byte string with heap allocation for every non-empty value.
- **`smol_str::SmolStr`** — Stack-allocated string up to 23 bytes (inline) or reference-counted heap allocation for larger strings.
- **`smartstring::SmartString`** — Compact inline string representation up to 23 bytes on 64-bit architectures, falling back to heap allocation when capacity is exceeded.

### Key Features
- **Reentrancy-Safe Heap Tracing (`alloc.rs`)**: Uses a custom `GlobalAlloc` implementation that streams allocation (`Alloc`) and deallocation (`Freed`) events to `stderr` using raw unbuffered OS system calls (`kernel32::WriteFile` on Windows and `libc::write` on Unix). This guarantees zero heap allocation overhead during serialization and prevents stack overflow / recursive allocator deadlocks.
- **Configurable Dataset Sampling (`sample.rs`)**: Deserializes JSON datasets (e.g. city/demographic records) into structured records with pluggable string representations (`std`, `smol`, `smart`).
- **Terminal Memory Visualization (`report.rs`)**: Parses line-delimited JSON allocation event logs with automatic encoding detection (UTF-8, UTF-16 LE/BE, and UTF-8 BOM), calculates peak memory usage and total allocated/freed bytes, and renders an ASCII memory curve chart directly in your terminal using `textplots`.

---

## Prerequisites

- **Rust**: Rust 1.85+ (or compatible Rust 2024 edition toolchain) and `cargo`.
  - Install via [rustup](https://rustup.rs/):
    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
- **Operating System**: Windows, macOS, or Linux.

---

## Installation

1. **Clone the repository**:
   ```sh
   git clone https://github.com/<your-username>/small.git
   cd small
   ```

2. **Build the project**:
   ```sh
   cargo build --release
   ```
   The compiled binary will be available at `./target/release/small` (or `.\target\release\small.exe` on Windows).

---

## Usage Examples

`small` provides two main subcommands: `sample` and `report`.

### 1. Profiling Heap Allocations (`sample`)

Run the `sample` subcommand to deserialize a dataset while streaming heap allocation events to `stderr`. Redirect `stderr` (`2>`) to a line-delimited JSON file (`.ldjson`).

#### Using Standard `String` (`std`):
```sh
cargo run -- sample --lib std --path input/cities.json 2> events_std.ldjson
```

#### Using `smol_str` (`smol`):
```sh
cargo run -- sample --lib smol --path input/cities.json 2> events_smol.ldjson
```

#### Using `smartstring` (`smart`):
```sh
cargo run -- sample --lib smart --path input/cities.json 2> events_smart.ldjson
```

*Note on Windows PowerShell:* You can capture `stderr` using:
```powershell
cargo run -- sample --lib std --path input/cities.json 2> events_std.ldjson
```

---

### 2. Generating Allocation Reports & Terminal Plots (`report`)

Analyze the captured allocation event log to view total events, peak heap memory, allocation counts, and an ASCII step chart:

```sh
cargo run -- report events_std.ldjson
```

#### Example Output:
```text
found 3023 events
   122.4 KiB |                                                    
             |                                    ..'''''''''     
             |                              ..''''                
             |                        ..''''                      
             |                  ..''''                            
             |            ..''''                                  
             |      ..''''                                        
         0 B |''''''                                              
             +--------------------------------------------------
             0.0                                          3023.0
     total events | 3023
      peak bytes  | 119.5 KiB
     ----------------------------
     alloc events | 3011
     alloc bytes  | 166.0 KiB
     ----------------------------
     freed events | 12
     freed bytes  | 71.8 KiB
```

Comparing the reports across `--lib std`, `--lib smol`, and `--lib smart` clearly shows the dramatic reduction in allocation count and peak heap memory footprint achieved through small string optimizations.

---

## Project Structure

```text
small/
├── Cargo.toml          # Package configuration and dependencies
├── input/
│   └── cities.json     # Sample dataset for deserialization benchmarks
├── src/
│   ├── main.rs         # CLI entry point and subcommand dispatcher (argh)
│   ├── alloc.rs        # Custom GlobalAlloc memory tracer with platform raw I/O
│   ├── sample.rs       # Dataset deserializer across std/smol/smart string types
│   └── report.rs       # Event parser, metric aggregation, and ASCII chart renderer
└── README.md           # Project documentation
```

---

## Contribution Guidelines

Contributions, improvements, and additional string library comparisons are welcome!

1. **Fork & Branch**: Create a feature branch for your changes (`git checkout -b feature/my-feature`).
2. **Code Style & Formatting**: Ensure code adheres to standard Rust formatting:
   ```sh
   cargo fmt --check
   ```
3. **Linting**: Ensure there are no Clippy warnings:
   ```sh
   cargo clippy --all-targets -- -D warnings
   ```
4. **Verification**: Verify that the project builds and runs cleanly:
   ```sh
   cargo test
   cargo check
   ```
5. **Submit a Pull Request**: Provide a clear description of your changes and any benchmark comparisons.

---

## License

This project is licensed under the MIT License or Apache 2.0 (standard Rust dual-license terms).
