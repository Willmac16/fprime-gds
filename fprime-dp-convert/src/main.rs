//! FPrime Data Product Converter CLI
//!
//! Convert FPrime Data Products and telemetry to HDF5 time series.
//!
//! # Usage
//!
//! ```bash
//! fprime-dp-convert convert --dictionary dict.json --output output.h5 input1.bin input2.bin
//! ```

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::{Context, Result};

use fprime_tlm::{Dictionary, DpContainer};
use fprime_tlm::packets::dp_container::HashDigest;

#[derive(Parser)]
#[command(name = "fprime-dp-convert")]
#[command(author, version, about = "Convert FPrime Data Products to HDF5 time series")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert data product files to HDF5
    Convert {
        /// Path to FPrime JSON dictionary
        #[arg(short, long)]
        dictionary: PathBuf,

        /// Output HDF5 file path
        #[arg(short, long)]
        output: PathBuf,

        /// Input data product files
        #[arg(required = true)]
        inputs: Vec<PathBuf>,

        /// Hash type: "crc32" or "sha256"
        #[arg(long, default_value = "crc32")]
        hash_type: String,

        /// Compression level (0-9)
        #[arg(long, default_value = "4")]
        compression: u8,
    },

    /// Inspect a data product file
    Inspect {
        /// Input data product file
        input: PathBuf,

        /// Hash type: "crc32" or "sha256"
        #[arg(long, default_value = "crc32")]
        hash_type: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Validate data product file integrity
    Validate {
        /// Input data product files
        #[arg(required = true)]
        inputs: Vec<PathBuf>,

        /// Hash type: "crc32" or "sha256"
        #[arg(long, default_value = "crc32")]
        hash_type: String,
    },
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Convert {
            dictionary,
            output,
            inputs,
            hash_type,
            compression,
        } => {
            cmd_convert(&dictionary, &output, &inputs, &hash_type, compression)
        }
        Commands::Inspect {
            input,
            hash_type,
            json,
        } => {
            cmd_inspect(&input, &hash_type, json)
        }
        Commands::Validate { inputs, hash_type } => {
            cmd_validate(&inputs, &hash_type)
        }
    }
}

fn cmd_convert(
    dictionary_path: &PathBuf,
    output_path: &PathBuf,
    inputs: &[PathBuf],
    hash_type: &str,
    _compression: u8,
) -> Result<()> {
    let hash_size = parse_hash_size(hash_type)?;

    // Load dictionary
    println!("Loading dictionary: {}", dictionary_path.display());
    let dict = Dictionary::from_json_file(dictionary_path)
        .context("Failed to load dictionary")?;

    println!("Processing {} input files...", inputs.len());

    // Process each input file
    let mut containers = Vec::new();
    for input in inputs {
        println!("  Reading: {}", input.display());
        let data = std::fs::read(input)
            .with_context(|| format!("Failed to read {}", input.display()))?;

        let container = DpContainer::parse(&data, hash_size)
            .with_context(|| format!("Failed to parse {}", input.display()))?;

        if !container.hash_valid {
            eprintln!("  WARNING: Hash validation failed for {}", input.display());
        }

        containers.push(container);
    }

    // Write HDF5 output
    #[cfg(feature = "hdf5")]
    {
        use fprime_tlm::output::hdf5::{Hdf5Writer, Hdf5Config};

        println!("Writing HDF5: {}", output_path.display());

        let config = Hdf5Config {
            compression_level: _compression,
            ..Default::default()
        };

        let mut writer = Hdf5Writer::new(output_path, &dict, config)
            .context("Failed to create HDF5 file")?;

        writer.write_metadata(None, &[])?;

        for container in &containers {
            writer.write_dp_container(container)?;
        }

        writer.finalize()?;
    }

    #[cfg(not(feature = "hdf5"))]
    {
        eprintln!("HDF5 support not enabled. Rebuild with --features hdf5");
        std::process::exit(1);
    }

    println!("Done! Processed {} containers.", containers.len());
    Ok(())
}

fn cmd_inspect(input: &PathBuf, hash_type: &str, json: bool) -> Result<()> {
    let hash_size = parse_hash_size(hash_type)?;

    let data = std::fs::read(input)
        .with_context(|| format!("Failed to read {}", input.display()))?;

    let container = DpContainer::parse(&data, hash_size)
        .with_context(|| format!("Failed to parse {}", input.display()))?;

    if json {
        let output = fprime_tlm::output::json::dp_container_to_json(&container, true)?;
        println!("{}", output);
    } else {
        println!("Data Product Container");
        println!("======================");
        println!("Container ID:  {}", container.header.container_id);
        println!("Priority:      {}", container.header.priority);
        println!("Time:          {}", container.header.time_tag);
        println!("State:         {:?}", container.header.dp_state);
        println!("Data Size:     {} bytes", container.header.data_size);
        println!("Hash Valid:    {}", container.hash_valid);
        println!("Records:       {}", container.records.len());
    }

    Ok(())
}

fn cmd_validate(inputs: &[PathBuf], hash_type: &str) -> Result<()> {
    let hash_size = parse_hash_size(hash_type)?;

    let mut valid_count = 0;
    let mut invalid_count = 0;

    for input in inputs {
        let data = std::fs::read(input)
            .with_context(|| format!("Failed to read {}", input.display()))?;

        match DpContainer::parse(&data, hash_size) {
            Ok(container) => {
                if container.hash_valid {
                    println!("VALID:   {}", input.display());
                    valid_count += 1;
                } else {
                    println!("INVALID: {} (hash mismatch)", input.display());
                    invalid_count += 1;
                }
            }
            Err(e) => {
                println!("ERROR:   {} ({})", input.display(), e);
                invalid_count += 1;
            }
        }
    }

    println!("\nSummary: {} valid, {} invalid", valid_count, invalid_count);

    if invalid_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn parse_hash_size(hash_type: &str) -> Result<usize> {
    match hash_type.to_lowercase().as_str() {
        "crc32" => Ok(HashDigest::CRC32_SIZE),
        "sha256" => Ok(HashDigest::SHA256_SIZE),
        _ => {
            anyhow::bail!("Invalid hash type '{}'. Use 'crc32' or 'sha256'", hash_type);
        }
    }
}
