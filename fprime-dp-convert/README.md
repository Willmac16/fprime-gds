# fprime-dp-convert

A Rust tool for converting FPrime Data Products and telemetry packets into HDF5 time series files for analysis and visualization.

## Features

- Parse FPrime Data Product (DP) container files
- Deserialize telemetry using dictionary-driven type definitions
- Validate data integrity via CRC32 or SHA256 checksums
- Output to HDF5 time series format
- High-performance batch processing

## Installation

### From Source

```bash
# Requires Rust 1.70+ and HDF5 development libraries
cargo install --path .

# Without HDF5 support (inspect/validate only)
cargo install --path . --no-default-features
```

### Dependencies

- **HDF5**: Install via your package manager
  - Ubuntu/Debian: `apt install libhdf5-dev`
  - macOS: `brew install hdf5`
  - Fedora: `dnf install hdf5-devel`

## Usage

### Convert Data Products to HDF5

```bash
fprime-dp-convert convert \
  --dictionary dictionary.json \
  --output telemetry.h5 \
  data_products/*.bin
```

### Inspect a Data Product File

```bash
# Human-readable summary
fprime-dp-convert inspect data_product.bin

# JSON output
fprime-dp-convert inspect data_product.bin --json
```

### Validate File Integrity

```bash
fprime-dp-convert validate data_products/*.bin --hash-type crc32
```

## HDF5 Output Structure

```
telemetry.h5
├── /metadata
│   ├── dictionary_hash
│   ├── source_files
│   └── creation_time
├── /channels/<component>/<channel>
│   ├── time: f64[]       # Unix timestamp
│   └── value: <type>[]   # Channel values
└── /data_products/<container>
    ├── time: f64[]
    └── /records/<record>/...
```

## Binary Formats

This tool implements parsers for FPrime's binary serialization formats:

- **Data Product Container**: 58-byte header + hash + data + hash
- **TimeType**: 11-12 byte timestamp (base, context, seconds, microseconds)
- **Telemetry Packets**: Packetized channel values with timestamps

All formats use big-endian byte order by default.

## Development

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug fprime-dp-convert inspect file.bin

# Build release
cargo build --release
```

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed design documentation.

## License

Apache-2.0

## Related Projects

- [FPrime](https://github.com/nasa/fprime) - Flight software framework
- [FPrime GDS](https://github.com/nasa/fprime-gds) - Ground Data System
