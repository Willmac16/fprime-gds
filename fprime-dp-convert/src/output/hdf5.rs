//! HDF5 time series output writer
//!
//! Creates HDF5 files with the following structure:
//!
//! ```text
//! telemetry.h5
//! ├── /metadata
//! │   ├── dictionary_hash
//! │   ├── source_files
//! │   └── creation_time
//! ├── /channels/<component>/<channel>
//! │   ├── time: f64[]
//! │   └── value: <type>[]
//! └── /data_products/<container>
//!     ├── time: f64[]
//!     └── /records/<record>/...
//! ```

use std::collections::HashMap;
use std::path::Path;
use crate::error::Result;
use crate::dictionary::Dictionary;
use crate::packets::dp_container::DpContainer;
use crate::types::time::TimeType;

/// Configuration for HDF5 output
#[derive(Debug, Clone)]
pub struct Hdf5Config {
    /// Chunk size for datasets (number of elements per chunk)
    pub chunk_size: usize,

    /// Compression level (0-9, 0 = no compression)
    pub compression_level: u8,

    /// Whether to include raw binary data
    pub include_raw: bool,

    /// Time format in output
    pub time_format: TimeFormat,
}

impl Default for Hdf5Config {
    fn default() -> Self {
        Self {
            chunk_size: 10000,
            compression_level: 4,
            include_raw: false,
            time_format: TimeFormat::UnixSeconds,
        }
    }
}

/// Time format for HDF5 output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFormat {
    /// Unix timestamp as f64 (seconds.microseconds)
    UnixSeconds,
    /// Separate seconds and microseconds columns
    SeparateFields,
    /// ISO 8601 string format
    Iso8601,
}

/// HDF5 writer for FPrime telemetry data
pub struct Hdf5Writer {
    file: hdf5::File,
    config: Hdf5Config,
    channel_writers: HashMap<u32, ChannelDataset>,
    dp_writers: HashMap<u32, DpDataset>,
}

struct ChannelDataset {
    time_dataset: hdf5::Dataset,
    value_dataset: hdf5::Dataset,
    count: usize,
}

struct DpDataset {
    group: hdf5::Group,
    time_dataset: hdf5::Dataset,
    count: usize,
}

impl Hdf5Writer {
    /// Create a new HDF5 writer
    ///
    /// # Arguments
    ///
    /// * `path` - Output file path
    /// * `dict` - FPrime dictionary for type information
    /// * `config` - Writer configuration
    pub fn new<P: AsRef<Path>>(
        path: P,
        _dict: &Dictionary,
        config: Hdf5Config,
    ) -> Result<Self> {
        let file = hdf5::File::create(path)?;

        // Create top-level groups
        file.create_group("metadata")?;
        file.create_group("channels")?;
        file.create_group("data_products")?;

        Ok(Self {
            file,
            config,
            channel_writers: HashMap::new(),
            dp_writers: HashMap::new(),
        })
    }

    /// Write metadata to the file
    pub fn write_metadata(
        &self,
        dictionary_hash: Option<&str>,
        source_files: &[String],
    ) -> Result<()> {
        let metadata = self.file.group("metadata")?;

        // Write dictionary hash if provided
        if let Some(hash) = dictionary_hash {
            let ds = metadata.new_dataset::<hdf5::types::VarLenUnicode>()
                .create("dictionary_hash")?;
            ds.write_scalar(&hdf5::types::VarLenUnicode::from(hash))?;
        }

        // Write creation time
        let now = chrono::Utc::now().to_rfc3339();
        let ds = metadata.new_dataset::<hdf5::types::VarLenUnicode>()
            .create("creation_time")?;
        ds.write_scalar(&hdf5::types::VarLenUnicode::from(now.as_str()))?;

        // Write source files
        if !source_files.is_empty() {
            // TODO: Write as variable-length string array
        }

        Ok(())
    }

    /// Write a telemetry channel value
    pub fn write_channel_value(
        &mut self,
        channel_id: u32,
        time: &TimeType,
        value: f64, // Simplified - full impl would use FprimeValue
    ) -> Result<()> {
        // Get or create dataset for this channel
        if !self.channel_writers.contains_key(&channel_id) {
            self.create_channel_dataset(channel_id)?;
        }

        let writer = self.channel_writers.get_mut(&channel_id).unwrap();

        // Convert time to f64
        let time_val = time.to_unix_timestamp();

        // Extend datasets
        let new_size = writer.count + 1;
        writer.time_dataset.resize(new_size)?;
        writer.value_dataset.resize(new_size)?;

        // Write values
        writer.time_dataset.write_slice(&[time_val], writer.count..new_size)?;
        writer.value_dataset.write_slice(&[value], writer.count..new_size)?;

        writer.count = new_size;

        Ok(())
    }

    /// Write a data product container
    pub fn write_dp_container(&mut self, container: &DpContainer) -> Result<()> {
        let container_id = container.header.container_id;

        // Get or create dataset for this container
        if !self.dp_writers.contains_key(&container_id) {
            self.create_dp_dataset(container_id)?;
        }

        let writer = self.dp_writers.get_mut(&container_id).unwrap();

        // Convert time to f64
        let time_val = container.header.time_tag.to_unix_timestamp();

        // Extend time dataset
        let new_size = writer.count + 1;
        writer.time_dataset.resize(new_size)?;
        writer.time_dataset.write_slice(&[time_val], writer.count..new_size)?;

        writer.count = new_size;

        // TODO: Write record data based on dictionary

        Ok(())
    }

    /// Finalize and close the HDF5 file
    pub fn finalize(self) -> Result<()> {
        // File is automatically closed when dropped
        // Any flush operations would go here
        Ok(())
    }

    fn create_channel_dataset(&mut self, channel_id: u32) -> Result<()> {
        let channels = self.file.group("channels")?;

        // Create group for this channel
        let group_name = format!("channel_{}", channel_id);
        let group = channels.create_group(&group_name)?;

        // Create resizable datasets
        let time_dataset = group.new_dataset::<f64>()
            .chunk(self.config.chunk_size)
            .deflate(self.config.compression_level)
            .resizable(true)
            .create("time")?;

        let value_dataset = group.new_dataset::<f64>()
            .chunk(self.config.chunk_size)
            .deflate(self.config.compression_level)
            .resizable(true)
            .create("value")?;

        // Set initial size to 0
        time_dataset.resize(0)?;
        value_dataset.resize(0)?;

        // Add attributes
        time_dataset.new_attr::<u32>().create("channel_id")?.write_scalar(&channel_id)?;

        self.channel_writers.insert(channel_id, ChannelDataset {
            time_dataset,
            value_dataset,
            count: 0,
        });

        Ok(())
    }

    fn create_dp_dataset(&mut self, container_id: u32) -> Result<()> {
        let dps = self.file.group("data_products")?;

        // Create group for this container
        let group_name = format!("container_{}", container_id);
        let group = dps.create_group(&group_name)?;

        // Create time dataset
        let time_dataset = group.new_dataset::<f64>()
            .chunk(self.config.chunk_size)
            .deflate(self.config.compression_level)
            .resizable(true)
            .create("time")?;

        time_dataset.resize(0)?;

        // Add attributes
        group.new_attr::<u32>().create("container_id")?.write_scalar(&container_id)?;

        self.dp_writers.insert(container_id, DpDataset {
            group,
            time_dataset,
            count: 0,
        });

        Ok(())
    }
}

use chrono;
