# FPrime Telemetry Converter - Rust Tool Architecture & Development Plan

## Executive Summary

This document describes the architecture, test strategy, and development plan for a Rust tool that converts FPrime telemetry packets (serialized into Data Product files) back into native data types for construction of time series outputs like HDF5 telemetry history files.

**Key Goals:**
- Parse FPrime Data Product (DP) container files with full fidelity
- Deserialize telemetry records using dictionary-driven type definitions
- Produce HDF5 time series files suitable for analysis and visualization
- Achieve high performance for processing large telemetry archives
- Maintain compatibility with FPrime FSW serialization formats

---

## 1. System Architecture

### 1.1 High-Level Components

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        fprime-tlm-convert (CLI)                         │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │  Dictionary │  │   Parser    │  │    Type     │  │     Output      │ │
│  │   Loader    │  │   Engine    │  │   System    │  │    Writers      │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └────────┬────────┘ │
│         │                │                │                   │         │
│         ▼                ▼                ▼                   ▼         │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │                        Core Library (lib.rs)                        ││
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐ ││
│  │  │ dp_      │ │ tlm_     │ │ serial_  │ │ time_    │ │ hash_     │ ││
│  │  │ container│ │ packet   │ │ buffer   │ │ type     │ │ utils     │ ││
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └───────────┘ ││
│  └─────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           External Crates                               │
│  hdf5-rust │ serde_json │ byteorder │ crc32fast │ sha2 │ clap │ thiserror│
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Module Breakdown

```
fprime-tlm-convert/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Library root, public API
│   ├── main.rs                   # CLI entry point
│   │
│   ├── dictionary/               # Dictionary loading and management
│   │   ├── mod.rs
│   │   ├── json_loader.rs        # JSON dictionary parser
│   │   ├── types.rs              # Dictionary type definitions
│   │   ├── channels.rs           # Channel definitions
│   │   ├── records.rs            # DP record definitions
│   │   └── containers.rs         # DP container definitions
│   │
│   ├── types/                    # FPrime type system implementation
│   │   ├── mod.rs
│   │   ├── primitives.rs         # U8-U64, I8-I64, F32, F64, bool
│   │   ├── time.rs               # TimeType (12 bytes)
│   │   ├── string.rs             # Length-prefixed strings
│   │   ├── array.rs              # Fixed-size arrays
│   │   ├── enum_type.rs          # Enumerated types
│   │   ├── struct_type.rs        # Composite structures
│   │   └── poly_type.rs          # Polymorphic type handling
│   │
│   ├── serial/                   # Serialization infrastructure
│   │   ├── mod.rs
│   │   ├── buffer.rs             # SerialBuffer implementation
│   │   ├── traits.rs             # Serializable/Deserializable traits
│   │   └── endian.rs             # Endianness handling
│   │
│   ├── packets/                  # Packet format implementations
│   │   ├── mod.rs
│   │   ├── descriptor.rs         # Packet type descriptors (APIDs)
│   │   ├── dp_container.rs       # Data Product container (58-byte header)
│   │   ├── tlm_packet.rs         # Telemetry packet format
│   │   └── com_packet.rs         # Communication packet base
│   │
│   ├── hash/                     # Hash/checksum utilities
│   │   ├── mod.rs
│   │   ├── crc32.rs              # CRC32 implementation
│   │   └── sha256.rs             # SHA256 implementation
│   │
│   ├── output/                   # Output format writers
│   │   ├── mod.rs
│   │   ├── hdf5.rs               # HDF5 time series writer
│   │   ├── json.rs               # JSON output (for debugging)
│   │   └── csv.rs                # CSV output (simple analysis)
│   │
│   └── error.rs                  # Error types and handling
│
├── tests/                        # Integration tests
│   ├── dp_container_tests.rs
│   ├── tlm_packet_tests.rs
│   ├── dictionary_tests.rs
│   ├── hdf5_output_tests.rs
│   └── fixtures/                 # Test data files
│       ├── dictionaries/
│       └── binary_samples/
│
├── benches/                      # Performance benchmarks
│   ├── parsing_bench.rs
│   └── hdf5_write_bench.rs
│
└── examples/                     # Usage examples
    ├── parse_dp_file.rs
    └── batch_convert.rs
```

---

## 2. Core Data Structures

### 2.1 Data Product Container Format (from FSW)

```rust
/// Data Product Container Header (58 bytes)
/// Reference: Fw/Dp/DpContainer.hpp
pub struct DpContainerHeader {
    /// Packet descriptor/APID (U16) - always 0x0005 for DP
    pub packet_descriptor: u16,      // offset 0, size 2

    /// Container ID (U32)
    pub container_id: u32,           // offset 2, size 4

    /// Priority (U32)
    pub priority: u32,               // offset 6, size 4

    /// Time tag (12 bytes)
    pub time_tag: TimeType,          // offset 10, size 12

    /// Processing types bitmask (U8)
    pub proc_types: u8,              // offset 22, size 1

    /// User data (32 bytes default)
    pub user_data: [u8; 32],         // offset 23, size 32

    /// Data product state (U8)
    pub dp_state: DpState,           // offset 55, size 1

    /// Data size (U16)
    pub data_size: u16,              // offset 56, size 2
}
// Total header: 58 bytes

/// Full Data Product Container
pub struct DpContainer {
    pub header: DpContainerHeader,
    pub header_hash: HashDigest,     // 4 bytes (CRC32) or 32 bytes (SHA256)
    pub records: Vec<DpRecord>,      // variable, total = header.data_size
    pub data_hash: HashDigest,       // 4 bytes (CRC32) or 32 bytes (SHA256)
}

/// Data Product State
#[repr(u8)]
pub enum DpState {
    Untransmitted = 0,
    Partial = 1,
    Transmitted = 2,
}
```

### 2.2 Time Type (12 bytes)

```rust
/// FPrime Time Type (12 bytes)
/// Reference: Fw/Time/Time.fpp
pub struct TimeType {
    /// Time base (U16)
    pub time_base: TimeBase,         // offset 0, size 2

    /// Time context (U8)
    pub time_context: u8,            // offset 2, size 1

    /// Seconds since epoch (U32)
    pub seconds: u32,                // offset 3, size 4

    /// Microseconds (U32, 0-999999)
    pub useconds: u32,               // offset 7, size 4
}
// Total: 11 bytes (Note: FSW says 12, but layout is 11 - verify!)

#[repr(u16)]
pub enum TimeBase {
    None = 0,
    ProcTime = 1,
    WorkstationTime = 2,
    SpacecraftTime = 3,
    DontCare = 0xFFFF,
}
```

### 2.3 Telemetry Packet Format

```rust
/// Telemetry Packet
/// Reference: Fw/Tlm/TlmPacket.hpp
pub struct TlmPacket {
    /// Packet type (U16) - 0x0001 for telemetry
    pub packet_type: u16,

    /// Telemetry entries
    pub entries: Vec<TlmEntry>,
}

pub struct TlmEntry {
    /// Channel ID (U32)
    pub channel_id: u32,

    /// Timestamp
    pub time: TimeType,

    /// Raw serialized value
    pub data: Vec<u8>,
}
```

### 2.4 Dynamic Type System

```rust
/// Runtime type representation for dictionary-driven deserialization
pub enum FprimeValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
    Enum { name: String, value: i64, repr: String },
    Array(Vec<FprimeValue>),
    Struct(IndexMap<String, FprimeValue>),
}

/// Type definition from dictionary
pub enum TypeDef {
    Primitive(PrimitiveType),
    Enum(EnumDef),
    Array(ArrayDef),
    Struct(StructDef),
    Alias(String),  // Reference to another type
}
```

---

## 3. Dictionary System Design

### 3.1 JSON Dictionary Schema

The tool must parse FPrime JSON dictionaries with the following structure:

```rust
pub struct Dictionary {
    pub metadata: Metadata,
    pub constants: Vec<Constant>,
    pub type_definitions: Vec<TypeDefinition>,
    pub channels: Vec<ChannelDef>,
    pub records: Vec<RecordDef>,
    pub containers: Vec<ContainerDef>,
    pub packets: Vec<PacketDef>,
}

pub struct ChannelDef {
    pub id: u32,
    pub name: String,
    pub component: String,
    pub type_name: String,
    pub format_string: Option<String>,
    pub limits: Option<Limits>,
}

pub struct RecordDef {
    pub id: u32,
    pub name: String,
    pub type_name: String,
    pub is_array: bool,
}
```

### 3.2 Type Resolution

```rust
pub struct TypeResolver {
    /// Map from qualified type name to definition
    types: HashMap<String, TypeDef>,

    /// Cache of resolved sizes
    size_cache: HashMap<String, usize>,
}

impl TypeResolver {
    /// Resolve type aliases to concrete types
    pub fn resolve(&self, name: &str) -> Result<&TypeDef, Error>;

    /// Get serialized size of a type
    pub fn size_of(&self, name: &str) -> Result<usize, Error>;

    /// Deserialize a value given its type name
    pub fn deserialize(&self, name: &str, buf: &mut SerialBuffer)
        -> Result<FprimeValue, Error>;
}
```

---

## 4. HDF5 Output Design

### 4.1 HDF5 File Structure

```
telemetry.h5
├── /metadata
│   ├── dictionary_hash: string
│   ├── source_files: string[]
│   ├── creation_time: string
│   └── fprime_version: string
│
├── /channels
│   ├── /<component_name>
│   │   ├── /<channel_name>
│   │   │   ├── time: f64[]          # Unix timestamp (seconds.microseconds)
│   │   │   ├── value: <type>[]      # Channel values
│   │   │   └── @attributes
│   │   │       ├── channel_id: u32
│   │   │       ├── type: string
│   │   │       └── unit: string (if available)
│   │   └── ...
│   └── ...
│
├── /data_products
│   ├── /<container_name>
│   │   ├── time: f64[]
│   │   ├── priority: u32[]
│   │   ├── state: u8[]
│   │   ├── /records
│   │   │   ├── /<record_name>
│   │   │   │   └── ... (type-dependent structure)
│   │   │   └── ...
│   │   └── @attributes
│   │       └── container_id: u32
│   └── ...
│
└── /events (future expansion)
```

### 4.2 HDF5 Writer Interface

```rust
pub struct Hdf5Writer {
    file: hdf5::File,
    channel_datasets: HashMap<u32, ChannelDataset>,
    dp_datasets: HashMap<u32, DpDataset>,
    config: Hdf5Config,
}

pub struct Hdf5Config {
    /// Chunk size for dataset creation
    pub chunk_size: usize,

    /// Compression level (0-9, 0 = none)
    pub compression: u8,

    /// Whether to include raw bytes
    pub include_raw: bool,

    /// Time format (unix, iso8601, fprime_native)
    pub time_format: TimeFormat,
}

impl Hdf5Writer {
    pub fn new(path: &Path, dict: &Dictionary, config: Hdf5Config) -> Result<Self>;
    pub fn write_channel(&mut self, ch: &ChannelData) -> Result<()>;
    pub fn write_dp_container(&mut self, dp: &DpContainer) -> Result<()>;
    pub fn finalize(self) -> Result<()>;
}
```

---

## 5. Testing Strategy

### 5.1 Test Pyramid

```
                    ┌─────────────────┐
                    │   E2E Tests     │  <- Full pipeline tests
                    │   (few, slow)   │
                   ┌┴─────────────────┴┐
                   │ Integration Tests │  <- Module interaction tests
                   │   (moderate)      │
                  ┌┴───────────────────┴┐
                  │    Unit Tests       │  <- Individual function tests
                  │   (many, fast)      │
                 ┌┴─────────────────────┴┐
                 │   Property Tests      │  <- Fuzzing/invariant tests
                 │   (exhaustive)        │
                └───────────────────────┘
```

### 5.2 Unit Test Categories

#### 5.2.1 Primitive Type Tests
```rust
#[cfg(test)]
mod primitive_tests {
    // Test each primitive type serialization/deserialization
    #[test]
    fn test_u8_roundtrip() { ... }
    #[test]
    fn test_u16_big_endian() { ... }
    #[test]
    fn test_u32_big_endian() { ... }
    #[test]
    fn test_i32_negative() { ... }
    #[test]
    fn test_f32_special_values() { ... }  // NaN, Inf, -0
    #[test]
    fn test_f64_precision() { ... }
    #[test]
    fn test_bool_serialization() { ... }
}
```

#### 5.2.2 Time Type Tests
```rust
#[cfg(test)]
mod time_tests {
    #[test]
    fn test_time_type_size() {
        assert_eq!(TimeType::SERIALIZED_SIZE, 12);
    }

    #[test]
    fn test_time_type_roundtrip() { ... }

    #[test]
    fn test_time_base_values() { ... }

    #[test]
    fn test_time_to_unix_conversion() { ... }

    #[test]
    fn test_time_ordering() { ... }
}
```

#### 5.2.3 Data Product Container Tests
```rust
#[cfg(test)]
mod dp_container_tests {
    #[test]
    fn test_header_size() {
        assert_eq!(DpContainerHeader::SIZE, 58);
    }

    #[test]
    fn test_header_field_offsets() {
        assert_eq!(DpContainerHeader::PACKET_DESCRIPTOR_OFFSET, 0);
        assert_eq!(DpContainerHeader::ID_OFFSET, 2);
        assert_eq!(DpContainerHeader::PRIORITY_OFFSET, 6);
        // ... etc
    }

    #[test]
    fn test_parse_valid_container() { ... }

    #[test]
    fn test_crc32_validation() { ... }

    #[test]
    fn test_sha256_validation() { ... }

    #[test]
    fn test_invalid_header_hash() { ... }

    #[test]
    fn test_invalid_data_hash() { ... }
}
```

#### 5.2.4 Dictionary Tests
```rust
#[cfg(test)]
mod dictionary_tests {
    #[test]
    fn test_load_simple_dictionary() { ... }

    #[test]
    fn test_type_resolution() { ... }

    #[test]
    fn test_alias_resolution() { ... }

    #[test]
    fn test_struct_size_calculation() { ... }

    #[test]
    fn test_nested_struct_resolution() { ... }

    #[test]
    fn test_enum_value_lookup() { ... }
}
```

### 5.3 Integration Tests

#### 5.3.1 Cross-Validation with Python GDS
```rust
/// Generate test data using Python GDS, parse with Rust, compare results
#[test]
fn test_python_gds_compatibility() {
    // 1. Use Python to serialize known values
    // 2. Parse with Rust implementation
    // 3. Assert values match
}
```

#### 5.3.2 Round-Trip Tests
```rust
#[test]
fn test_dp_container_roundtrip() {
    let original = create_test_dp_container();
    let serialized = original.serialize();
    let parsed = DpContainer::parse(&serialized)?;
    assert_eq!(original, parsed);
}
```

### 5.4 Property-Based Tests (using proptest)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_u32_roundtrip(value: u32) {
        let mut buf = SerialBuffer::new(4);
        buf.serialize_u32(value, Endianness::Big)?;
        buf.reset_deser();
        let result = buf.deserialize_u32(Endianness::Big)?;
        prop_assert_eq!(value, result);
    }

    #[test]
    fn prop_time_type_valid(
        base in 0u16..=3,
        ctx in any::<u8>(),
        secs in any::<u32>(),
        usecs in 0u32..1_000_000,
    ) {
        let time = TimeType {
            time_base: TimeBase::from(base),
            time_context: ctx,
            seconds: secs,
            useconds: usecs,
        };
        let serialized = time.serialize();
        prop_assert_eq!(serialized.len(), 12);
    }
}
```

### 5.5 Fuzz Testing

```rust
// In fuzz/fuzz_targets/parse_dp.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Should not panic, even on malformed input
    let _ = DpContainer::parse(data);
});
```

### 5.6 Test Fixtures

Create binary test fixtures from:

1. **FPrime Reference Implementation**: Generate known-good binaries using FSW
2. **Python GDS**: Use `data_product_writer.py` to create test files
3. **Hand-crafted Edge Cases**: Manually construct boundary condition tests

```
tests/fixtures/
├── dictionaries/
│   ├── simple_dictionary.json      # Minimal dictionary
│   ├── complex_dictionary.json     # All type variants
│   └── ref_mission_dictionary.json # Realistic mission dictionary
│
├── binary_samples/
│   ├── dp_container_simple.bin     # Single record DP
│   ├── dp_container_multi.bin      # Multiple records
│   ├── dp_container_nested.bin     # Nested struct records
│   ├── tlm_packet_basic.bin        # Basic telemetry
│   └── malformed/
│       ├── truncated_header.bin
│       ├── bad_crc.bin
│       └── oversized_data.bin
│
└── expected_outputs/
    ├── dp_container_simple.json    # Expected parsed values
    └── ...
```

---

## 6. Development Phases

### Phase 1: Foundation (Weeks 1-2)
**Goal**: Core serialization infrastructure

**Deliverables**:
- [ ] Project scaffolding with Cargo workspace
- [ ] `serial/buffer.rs` - SerialBuffer with offset tracking
- [ ] `serial/endian.rs` - Big/little endian support
- [ ] `types/primitives.rs` - All primitive type serialization
- [ ] `types/time.rs` - TimeType implementation
- [ ] `hash/crc32.rs` - CRC32 validation
- [ ] Comprehensive unit tests for all primitives

**Tests**:
- Primitive roundtrip tests
- Endianness verification
- TimeType serialization tests
- CRC32 known-value tests

### Phase 2: Container Parsing (Weeks 3-4)
**Goal**: Parse Data Product containers

**Deliverables**:
- [ ] `packets/dp_container.rs` - Header parsing
- [ ] `hash/sha256.rs` - SHA256 support
- [ ] Header and data hash validation
- [ ] Basic record extraction (raw bytes)

**Tests**:
- Header field offset verification
- Hash validation (both CRC32 and SHA256)
- Error handling for malformed containers
- Integration test with Python-generated DPs

### Phase 3: Dictionary & Types (Weeks 5-7)
**Goal**: Dynamic type system from dictionary

**Deliverables**:
- [ ] `dictionary/json_loader.rs` - Parse JSON dictionaries
- [ ] `dictionary/types.rs` - Type definition structures
- [ ] `types/enum_type.rs` - Enum deserialization
- [ ] `types/array.rs` - Array deserialization
- [ ] `types/struct_type.rs` - Struct deserialization
- [ ] `types/string.rs` - Length-prefixed strings
- [ ] Type resolution with alias support

**Tests**:
- Dictionary loading tests
- Type resolution tests
- Complex nested struct tests
- Property tests for type system

### Phase 4: Record Deserialization (Weeks 8-9)
**Goal**: Full DP record parsing

**Deliverables**:
- [ ] Integrate dictionary types with DP parsing
- [ ] Record ID to type mapping
- [ ] Array record handling
- [ ] Full DpContainer::parse() with typed records

**Tests**:
- End-to-end DP parsing tests
- Cross-validation with Python GDS
- Fuzz testing for parser robustness

### Phase 5: Telemetry Packets (Week 10)
**Goal**: Support packetized telemetry

**Deliverables**:
- [ ] `packets/tlm_packet.rs` - Telemetry packet parsing
- [ ] Channel ID to type mapping
- [ ] Packet template support

**Tests**:
- Telemetry packet parsing tests
- Multi-channel packet tests

### Phase 6: HDF5 Output (Weeks 11-13)
**Goal**: Time series HDF5 generation

**Deliverables**:
- [ ] `output/hdf5.rs` - HDF5 file creation
- [ ] Channel dataset writing
- [ ] DP container dataset writing
- [ ] Metadata and attributes
- [ ] Chunking and compression support

**Tests**:
- HDF5 file structure verification
- Data integrity tests (read back and verify)
- Performance benchmarks for large files

### Phase 7: CLI & Polish (Weeks 14-15)
**Goal**: Production-ready CLI tool

**Deliverables**:
- [ ] `main.rs` - CLI with clap
- [ ] Batch processing support
- [ ] Progress reporting
- [ ] Error reporting and diagnostics
- [ ] JSON and CSV output formats
- [ ] Documentation and examples

**Tests**:
- CLI argument parsing tests
- End-to-end workflow tests
- Performance benchmarks

### Phase 8: Hardening (Week 16)
**Goal**: Production readiness

**Deliverables**:
- [ ] Fuzz testing campaign
- [ ] Memory safety audit
- [ ] Performance optimization
- [ ] CI/CD pipeline
- [ ] Release packaging

---

## 7. Performance Considerations

### 7.1 Design for Performance

1. **Zero-copy parsing where possible**: Use `&[u8]` slices instead of copying
2. **Streaming processing**: Don't load entire files into memory
3. **Parallel processing**: Use rayon for batch file processing
4. **Efficient HDF5 writes**: Buffer writes, use chunked datasets

### 7.2 Benchmarks

```rust
// benches/parsing_bench.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_dp_header_parse(c: &mut Criterion) {
    let data = include_bytes!("fixtures/dp_container.bin");
    c.bench_function("dp_header_parse", |b| {
        b.iter(|| DpContainerHeader::parse(data))
    });
}

fn bench_large_dp_file(c: &mut Criterion) {
    // 100MB DP file
    c.bench_function("large_dp_parse", |b| {
        b.iter(|| parse_dp_file("large_dp.bin"))
    });
}
```

### 7.3 Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Header parse | < 100ns | Fixed size, simple |
| Record parse (simple) | < 1μs | Single primitive |
| Record parse (complex) | < 10μs | Nested struct |
| DP file (1MB) | < 10ms | Including hash validation |
| HDF5 write (1M points) | < 1s | Chunked, compressed |

---

## 8. Error Handling Strategy

### 8.1 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Buffer underflow: needed {needed} bytes, had {available}")]
    BufferUnderflow { needed: usize, available: usize },

    #[error("Invalid packet descriptor: expected {expected:#x}, got {got:#x}")]
    InvalidDescriptor { expected: u16, got: u16 },

    #[error("Hash validation failed: expected {expected:?}, computed {computed:?}")]
    HashMismatch { expected: Vec<u8>, computed: Vec<u8> },

    #[error("Unknown type: {0}")]
    UnknownType(String),

    #[error("Dictionary error: {0}")]
    Dictionary(#[from] DictionaryError),

    #[error("HDF5 error: {0}")]
    Hdf5(#[from] hdf5::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### 8.2 Error Recovery

- **Soft errors**: Log warning, continue processing (e.g., unknown channel ID)
- **Hard errors**: Stop processing, report location (e.g., hash mismatch)
- **Configurable**: `--strict` mode for zero tolerance

---

## 9. Dependencies

### 9.1 Core Dependencies

```toml
[dependencies]
# Serialization
byteorder = "1.5"           # Endian-aware byte handling
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"          # Dictionary parsing

# Hashing
crc32fast = "1.3"           # CRC32 (fast SIMD implementation)
sha2 = "0.10"               # SHA256

# HDF5
hdf5 = "0.8"                # HDF5 bindings

# CLI
clap = { version = "4.0", features = ["derive"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"              # For CLI error handling

# Utilities
indexmap = "2.0"            # Ordered maps for struct fields
log = "0.4"
env_logger = "0.10"

[dev-dependencies]
proptest = "1.0"            # Property-based testing
criterion = "0.5"           # Benchmarking
tempfile = "3.0"            # Temporary files for tests
```

### 9.2 Optional Features

```toml
[features]
default = ["hdf5"]
hdf5 = ["dep:hdf5"]
python = ["pyo3"]           # Python bindings (future)
parallel = ["rayon"]        # Parallel batch processing
```

---

## 10. Future Considerations

### 10.1 Potential Extensions

1. **Python bindings** via PyO3 for GDS integration
2. **Event log parsing** (similar to telemetry)
3. **Real-time streaming** from TCP socket
4. **Parquet output** for big data workflows
5. **Web assembly** build for browser-based analysis

### 10.2 Compatibility Matrix

| FPrime Version | Support Status |
|---------------|----------------|
| v3.x | Primary target |
| v2.x | Best effort |
| v1.x | Not planned |

---

## 11. Success Criteria

### 11.1 Functional Requirements

- [ ] Parse all FPrime primitive types correctly
- [ ] Parse Data Product containers with CRC32 and SHA256
- [ ] Load and resolve JSON dictionaries
- [ ] Deserialize all record types (primitives, enums, arrays, structs)
- [ ] Generate valid HDF5 files readable by h5py/HDFView
- [ ] Handle malformed input gracefully (no panics)

### 11.2 Non-Functional Requirements

- [ ] Parse 1MB DP file in < 50ms
- [ ] Write 10M telemetry points to HDF5 in < 30s
- [ ] Memory usage < 2x file size during processing
- [ ] Zero unsafe code in core parsing logic
- [ ] 90%+ test coverage on parsing modules

### 11.3 Validation

- [ ] Cross-validate 100% of test cases against Python GDS
- [ ] Process real mission data successfully
- [ ] Pass 24-hour fuzz testing campaign with no crashes

---

## Appendix A: FSW Reference Files

Key FSW source files for reference:

| File | Purpose |
|------|---------|
| `Fw/Types/BasicTypes.h` | Core type definitions |
| `Fw/Types/Serializable.hpp` | Serialization interface |
| `Fw/Dp/DpContainer.hpp` | DP container format |
| `Fw/Time/Time.hpp` | Time type |
| `Fw/Tlm/TlmPacket.hpp` | Telemetry packets |
| `Utils/Hash/Hash.hpp` | Hash interface |
| `default/config/FpConfig.fpp` | Type configuration |
| `default/config/DpCfg.fpp` | DP configuration |

## Appendix B: GDS Reference Files

Key GDS source files for cross-validation:

| File | Purpose |
|------|---------|
| `executables/data_product_writer.py` | DP parsing reference |
| `common/decoders/ch_decoder.py` | Channel decoder |
| `common/decoders/pkt_decoder.py` | Packet decoder |
| `common/models/serialize/` | Type system |
| `common/loaders/json_loader.py` | Dictionary loading |
