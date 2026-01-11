//! Synthetic NIfTI-like formats for testing.
//!
//! # Purpose
//!
//! This module defines a simplified NIfTI-like format with a custom header
//! and magic bytes (NIF1). Real NIfTI-1 files have a 348-byte header with
//! magic "n+1" or "ni1"; this module uses a minimal header for testing.
//!
//! # When to use
//!
//! - **Tests**: Use `write_test_nifti_scalar` to create gzipped fixtures
//! - **Production**: Use `nifti_real` module for actual NIfTI-1 files
//!
//! # Magic bytes
//!
//! - Synthetic: `NIF1` (custom, 4 bytes)
//! - Real NIfTI-1: `n+1\0` or `ni1\0` at offset 344
//!
//! The loader in `lib.rs` auto-detects which parser to use.

use crate::error::FormatError;

use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct NiftiScalar {
    pub dims: [u16; 8],
    pub data: Vec<f32>,
}

pub fn read_nifti_scalar(path: &Path) -> Result<NiftiScalar, FormatError> {
    let file = File::open(path)?;
    let mut decoder = GzDecoder::new(file);
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;

    let mut cursor = std::io::Cursor::new(&buf);

    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic)?;
    if &magic != b"NIF1" {
        return Err(FormatError::InvalidMagic {
            expected: "NIF1",
            found: String::from_utf8_lossy(&magic).into(),
        });
    }

    let mut dims = [0u16; 8];
    for d in &mut dims {
        *d = cursor.read_u16::<LittleEndian>()?;
    }

    let n_values = dims[1] as usize
        * dims[2] as usize
        * dims[3] as usize
        * dims[4] as usize;

    let header_bytes = 4 + 8 * 2;
    let expected_size = header_bytes + n_values * 4;
    if buf.len() != expected_size {
        return Err(FormatError::SizeMismatch {
            expected: expected_size,
            actual: buf.len(),
        });
    }

    let mut data = Vec::with_capacity(n_values);
    for _ in 0..n_values {
        let v = cursor.read_f32::<LittleEndian>()?;
        data.push(v);
    }

    Ok(NiftiScalar { dims, data })
}

#[cfg(test)]
pub(crate) fn write_test_nifti_scalar(
    path: &Path,
    dims: [u16; 8],
    data: &[f32],
) -> Result<(), FormatError> {
    use byteorder::WriteBytesExt;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::fs;
    use std::io::Write;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut buf = Vec::new();
    buf.write_all(b"NIF1")?;
    for d in &dims {
        buf.write_u16::<LittleEndian>(*d)?;
    }
    for v in data {
        buf.write_f32::<LittleEndian>(*v)?;
    }

    let file = File::create(path)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(&buf)?;
    encoder.finish()?;

    Ok(())
}
