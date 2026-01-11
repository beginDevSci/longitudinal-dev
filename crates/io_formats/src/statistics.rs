use byteorder::{LittleEndian, ReadBytesExt};
use serde::Deserialize;
use std::io::{Cursor, Read};

use crate::error::FormatError;

/// Hemisphere identifier used in file formats and metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Hemisphere {
    Left,
    Right,
}

impl Hemisphere {
    /// Return the hemisphere prefix used in file names ("lh" or "rh").
    pub fn as_str(&self) -> &'static str {
        match self {
            Hemisphere::Left => "lh",
            Hemisphere::Right => "rh",
        }
    }
}

/// BLMM analysis type used in file naming and metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Analysis {
    #[serde(rename = "des1")]
    Design1,
    #[serde(rename = "des2")]
    Design2,
    #[serde(rename = "compare")]
    Compare,
}

impl Analysis {
    /// Return the analysis code used in file names.
    pub fn as_str(&self) -> &'static str {
        match self {
            Analysis::Design1 => "des1",
            Analysis::Design2 => "des2",
            Analysis::Compare => "compare",
        }
    }
}

/// Statistic type used in BLMM outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Statistic {
    #[serde(rename = "beta")]
    Beta,
    #[serde(rename = "conT")]
    TStat,
    #[serde(rename = "conTlp")]
    LogP,
    #[serde(rename = "sigma2")]
    Sigma2,
    /// Chi-squared statistic (model comparison only)
    #[serde(rename = "Chi2")]
    Chi2,
    /// -log10(p) for Chi-squared (model comparison only)
    #[serde(rename = "Chi2lp")]
    Chi2lp,
}

impl Statistic {
    /// Return the statistic code used in file names.
    pub fn as_str(&self) -> &'static str {
        match self {
            Statistic::Beta => "beta",
            Statistic::TStat => "conT",
            Statistic::LogP => "conTlp",
            Statistic::Sigma2 => "sigma2",
            Statistic::Chi2 => "Chi2",
            Statistic::Chi2lp => "Chi2lp",
        }
    }

    /// Returns true if this statistic is only available for the Compare analysis.
    pub fn is_compare_only(&self) -> bool {
        matches!(self, Statistic::Chi2 | Statistic::Chi2lp)
    }

    /// Returns true if this statistic is available for Design analyses (des1/des2).
    pub fn is_design_stat(&self) -> bool {
        !self.is_compare_only()
    }
}

/// Flat statistics storage: [volume0_vertex0, volume0_vertex1, ..., volume1_vertex0, ...]
#[derive(Debug, Clone)]
pub struct StatisticData {
    pub values: Vec<f32>,
    pub n_vertices: usize,
    pub n_volumes: usize,
    pub global_min: f32,
    pub global_max: f32,
    pub volume_ranges: Vec<(f32, f32)>,
    pub nan_count: u32,
}

impl StatisticData {
    /// Parse statistical data from the custom BRS1 binary format.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FormatError> {
        let mut cursor = Cursor::new(bytes);

        // Magic
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic)?;
        if &magic != b"BRS1" {
            return Err(FormatError::InvalidMagic {
                expected: "BRS1",
                found: String::from_utf8_lossy(&magic).into(),
            });
        }

        let version = cursor.read_u32::<LittleEndian>()?;
        if version != 1 {
            return Err(FormatError::UnsupportedVersion {
                expected: 1,
                found: version,
            });
        }

        let flags = cursor.read_u32::<LittleEndian>()?;
        // Bits 3-31 must be zero
        let supported_mask: u32 = 0b0000_0111;
        if (flags & !supported_mask) != 0 {
            return Err(FormatError::UnsupportedFlags { flags });
        }

        let n_vertices = cursor.read_u32::<LittleEndian>()? as usize;
        let n_volumes = cursor.read_u32::<LittleEndian>()? as usize;

        let global_min = cursor.read_f32::<LittleEndian>()?;
        let global_max = cursor.read_f32::<LittleEndian>()?;
        let nan_count = cursor.read_u32::<LittleEndian>()?;

        let mut volume_ranges = Vec::with_capacity(n_volumes);
        for _ in 0..n_volumes {
            let vmin = cursor.read_f32::<LittleEndian>()?;
            let vmax = cursor.read_f32::<LittleEndian>()?;
            volume_ranges.push((vmin, vmax));
        }

        let header_bytes = 32 + n_volumes * 8;
        let expected_values = n_vertices * n_volumes;
        let expected_size = header_bytes + expected_values * 4;
        if bytes.len() != expected_size {
            return Err(FormatError::SizeMismatch {
                expected: expected_size,
                actual: bytes.len(),
            });
        }

        let mut values = Vec::with_capacity(expected_values);
        for _ in 0..expected_values {
            let v = cursor.read_f32::<LittleEndian>()?;
            values.push(v);
        }

        Ok(StatisticData {
            values,
            n_vertices,
            n_volumes,
            global_min,
            global_max,
            volume_ranges,
            nan_count,
        })
    }

    #[inline]
    pub fn get(&self, volume: usize, vertex: usize) -> Option<f32> {
        if volume < self.n_volumes && vertex < self.n_vertices {
            let idx = volume * self.n_vertices + vertex;
            Some(self.values[idx])
        } else {
            None
        }
    }

    #[inline]
    pub fn volume_slice(&self, volume: usize) -> Option<&[f32]> {
        if volume >= self.n_volumes {
            return None;
        }
        let start = volume * self.n_vertices;
        let end = start + self.n_vertices;
        Some(&self.values[start..end])
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatisticMetadata {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub hemisphere: Hemisphere,
    pub analysis: Analysis,
    /// Name of the colormap to use for visualization.
    /// Interpreted by higher-level crates such as `neuro_surface`.
    pub colormap: String,
    pub symmetric: bool,
    pub suggested_threshold: Option<f32>,
    pub nan_handling: NanHandling,
    pub volumes: Vec<VolumeLabel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VolumeLabel {
    pub index: u32,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NanHandling {
    #[default]
    Transparent,
    Gray,
    Zero,
}
