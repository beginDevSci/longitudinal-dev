//! Real NIfTI-1 format parser.
//!
//! NIfTI-1 (Neuroimaging Informatics Technology Initiative) is a binary format
//! for neuroimaging data. This module implements a minimal parser for reading
//! scalar overlay data from NIfTI-1 files.
//!
//! ## Format Specification
//!
//! NIfTI-1 header is 348 bytes:
//! - Bytes 0-3: sizeof_hdr (int32, must be 348)
//! - Bytes 40-55: dim[8] (int16[8], dimension sizes)
//! - Bytes 70-71: datatype (int16)
//! - Bytes 72-73: bitpix (int16, bits per voxel)
//! - Bytes 108-111: vox_offset (float32, offset to data)
//! - Bytes 344-347: magic (4 chars, "n+1\0" or "ni1\0")
//!
//! ## Supported Data Types
//!
//! - NIFTI_TYPE_UINT8 (2): 8-bit unsigned integer
//! - NIFTI_TYPE_INT16 (4): 16-bit signed integer
//! - NIFTI_TYPE_INT32 (8): 32-bit signed integer
//! - NIFTI_TYPE_FLOAT32 (16): 32-bit float
//! - NIFTI_TYPE_FLOAT64 (64): 64-bit double
//!
//! ## Supported File Types
//!
//! - `.nii`: Uncompressed single-file NIfTI
//! - `.nii.gz`: Gzip-compressed single-file NIfTI
//!
//! ## Dimensionality
//!
//! Supports 1D through 4D data:
//! - 1D: Linear vertex data (n_vertices,)
//! - 3D: Volumetric data (x, y, z)
//! - 4D: Time series (x, y, z, time)
//!
//! ## Flattening Order
//!
//! Multi-dimensional data is flattened in row-major (C) order:
//! For 4D data with dims [x, y, z, t], the linear index is:
//! `idx = x + y*dim_x + z*dim_x*dim_y + t*dim_x*dim_y*dim_z`
//!
//! This matches the standard NIfTI storage order.

use crate::error::FormatError;

use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

/// NIfTI scalar overlay data.
#[derive(Debug, Clone)]
pub struct Nifti1Scalar {
    /// Dimension sizes: dim[0] is number of dimensions, dim[1..] are sizes.
    pub dims: [i16; 8],
    /// Data type code (NIFTI_TYPE_* constant).
    pub datatype: i16,
    /// Bits per voxel.
    pub bitpix: i16,
    /// Scalar data values (converted to f32).
    pub data: Vec<f32>,
}

/// NIfTI-1 data type codes.
const NIFTI_TYPE_UINT8: i16 = 2;
const NIFTI_TYPE_INT16: i16 = 4;
const NIFTI_TYPE_INT32: i16 = 8;
const NIFTI_TYPE_FLOAT32: i16 = 16;
const NIFTI_TYPE_FLOAT64: i16 = 64;

/// Check if a file is a valid NIfTI-1 file.
pub fn is_real_nifti1(path: &Path) -> bool {
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    // Check for .nii or .nii.gz extension
    if !filename.ends_with(".nii") && !filename.ends_with(".nii.gz") {
        return false;
    }

    // Try to read and verify header
    if let Ok(bytes) = read_nifti_bytes(path) {
        if bytes.len() >= 348 {
            // Check sizeof_hdr (must be 348)
            let sizeof_hdr = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            if sizeof_hdr == 348 {
                // Check magic
                let magic = &bytes[344..348];
                return magic == b"n+1\0" || magic == b"ni1\0";
            }
        }
    }
    false
}

/// Read NIfTI file bytes, handling gzip compression.
fn read_nifti_bytes(path: &Path) -> Result<Vec<u8>, FormatError> {
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    if filename.ends_with(".gz") {
        let file = File::open(path)?;
        let mut decoder = GzDecoder::new(file);
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes)?;
        Ok(bytes)
    } else {
        Ok(std::fs::read(path)?)
    }
}

/// Read a NIfTI-1 file and return scalar overlay data.
///
/// This function reads the NIfTI-1 header and converts the voxel data to f32.
/// For 3D/4D images, the data is flattened to a 1D array suitable for surface overlays.
pub fn read_real_nifti1(path: &Path) -> Result<Nifti1Scalar, FormatError> {
    let bytes = read_nifti_bytes(path)?;
    read_nifti1_bytes(&bytes)
}

/// Read a NIfTI-1 from in-memory bytes.
///
/// Same format as `read_real_nifti1` but from a byte slice instead of a file.
/// The bytes should already be decompressed if from a .nii.gz file.
pub fn read_nifti1_bytes(bytes: &[u8]) -> Result<Nifti1Scalar, FormatError> {
    if bytes.len() < 348 {
        return Err(FormatError::SizeMismatch {
            expected: 348,
            actual: bytes.len(),
        });
    }

    let mut cursor = Cursor::new(&bytes);

    // Read sizeof_hdr (bytes 0-3)
    let sizeof_hdr = cursor.read_i32::<LittleEndian>()?;
    if sizeof_hdr != 348 {
        return Err(FormatError::InvalidMagic {
            expected: "sizeof_hdr=348",
            found: format!("sizeof_hdr={}", sizeof_hdr),
        });
    }

    // Skip to dim (bytes 40-55)
    cursor.seek(SeekFrom::Start(40))?;
    let mut dims = [0i16; 8];
    for d in &mut dims {
        *d = cursor.read_i16::<LittleEndian>()?;
    }

    // Skip to datatype (bytes 70-71)
    cursor.seek(SeekFrom::Start(70))?;
    let datatype = cursor.read_i16::<LittleEndian>()?;
    let bitpix = cursor.read_i16::<LittleEndian>()?;

    // Skip to vox_offset (bytes 108-111)
    cursor.seek(SeekFrom::Start(108))?;
    let vox_offset = cursor.read_f32::<LittleEndian>()?;

    // Skip to magic (bytes 344-347)
    cursor.seek(SeekFrom::Start(344))?;
    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic)?;

    if &magic != b"n+1\0" && &magic != b"ni1\0" {
        return Err(FormatError::InvalidMagic {
            expected: "n+1 or ni1",
            found: String::from_utf8_lossy(&magic).into(),
        });
    }

    // Calculate total voxel count
    let n_dims = dims[0] as usize;
    let mut total_voxels = 1usize;
    for i in 1..=n_dims.min(7) {
        total_voxels *= dims[i].max(1) as usize;
    }

    // Read voxel data
    let data_offset = if vox_offset > 0.0 {
        vox_offset as usize
    } else {
        348 // Default for single-file NIfTI
    };

    if bytes.len() < data_offset {
        return Err(FormatError::SizeMismatch {
            expected: data_offset,
            actual: bytes.len(),
        });
    }

    let data_bytes = &bytes[data_offset..];

    let data = match datatype {
        NIFTI_TYPE_UINT8 => {
            if data_bytes.len() < total_voxels {
                return Err(FormatError::SizeMismatch {
                    expected: total_voxels,
                    actual: data_bytes.len(),
                });
            }
            data_bytes[..total_voxels]
                .iter()
                .map(|&b| b as f32)
                .collect()
        }
        NIFTI_TYPE_INT16 => {
            let expected_size = total_voxels * 2;
            if data_bytes.len() < expected_size {
                return Err(FormatError::SizeMismatch {
                    expected: expected_size,
                    actual: data_bytes.len(),
                });
            }
            let mut values = Vec::with_capacity(total_voxels);
            let mut cursor = Cursor::new(data_bytes);
            for _ in 0..total_voxels {
                values.push(cursor.read_i16::<LittleEndian>()? as f32);
            }
            values
        }
        NIFTI_TYPE_INT32 => {
            let expected_size = total_voxels * 4;
            if data_bytes.len() < expected_size {
                return Err(FormatError::SizeMismatch {
                    expected: expected_size,
                    actual: data_bytes.len(),
                });
            }
            let mut values = Vec::with_capacity(total_voxels);
            let mut cursor = Cursor::new(data_bytes);
            for _ in 0..total_voxels {
                values.push(cursor.read_i32::<LittleEndian>()? as f32);
            }
            values
        }
        NIFTI_TYPE_FLOAT32 => {
            let expected_size = total_voxels * 4;
            if data_bytes.len() < expected_size {
                return Err(FormatError::SizeMismatch {
                    expected: expected_size,
                    actual: data_bytes.len(),
                });
            }
            let mut values = Vec::with_capacity(total_voxels);
            let mut cursor = Cursor::new(data_bytes);
            for _ in 0..total_voxels {
                values.push(cursor.read_f32::<LittleEndian>()?);
            }
            values
        }
        NIFTI_TYPE_FLOAT64 => {
            let expected_size = total_voxels * 8;
            if data_bytes.len() < expected_size {
                return Err(FormatError::SizeMismatch {
                    expected: expected_size,
                    actual: data_bytes.len(),
                });
            }
            let mut values = Vec::with_capacity(total_voxels);
            let mut cursor = Cursor::new(data_bytes);
            for _ in 0..total_voxels {
                values.push(cursor.read_f64::<LittleEndian>()? as f32);
            }
            values
        }
        _ => {
            return Err(FormatError::InvalidMagic {
                expected: "supported NIfTI datatype (2, 4, 8, 16, 64)",
                found: format!("datatype={}", datatype),
            });
        }
    };

    Ok(Nifti1Scalar {
        dims,
        datatype,
        bitpix,
        data,
    })
}

// ============================================================================
// Ground-Truth Constants for NIfTI Test Fixtures
// ============================================================================

/// Path to 3D float32 test fixture (100 vertices, sin wave values).
pub const NIFTI_3D_FLOAT32_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/nifti_real/test_overlay.nii"
);

/// Path to gzipped version of the 3D float32 fixture.
pub const NIFTI_3D_FLOAT32_GZ_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/nifti_real/test_overlay.nii.gz"
);

/// Path to 4D float32 test fixture (5x5x5x3 = 375 voxels).
pub const NIFTI_4D_FLOAT32_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/nifti_real/test_4d.nii"
);

/// Path to INT16 test fixture (50 values).
pub const NIFTI_INT16_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/nifti_real/test_int16.nii"
);

// Ground truth for 3D float32 fixture
/// Expected voxel count for 3D test file.
pub const NIFTI_3D_N_VOXELS: usize = 100;
/// Expected dimensions for 3D test file.
pub const NIFTI_3D_DIMS: [i16; 4] = [3, 100, 1, 1];
/// Sample values from 3D test file: sin(i * 0.1).
pub const NIFTI_3D_VALUE_SAMPLES: &[(usize, f32)] = &[
    (0, 0.0),
    (10, 0.841471),
    (50, -0.958924),
    (99, -0.457536),
];

// Ground truth for 4D float32 fixture
/// Expected voxel count for 4D test file (5*5*5*3).
pub const NIFTI_4D_N_VOXELS: usize = 375;
/// Expected dimensions for 4D test file.
pub const NIFTI_4D_DIMS: [i16; 5] = [4, 5, 5, 5, 3];
/// Sample values from 4D test file: linear index.
pub const NIFTI_4D_VALUE_SAMPLES: &[(usize, f32)] = &[
    (0, 0.0),
    (124, 124.0),  // Last voxel of first time point
    (125, 125.0),  // First voxel of second time point
    (374, 374.0),  // Last voxel
];

// Ground truth for INT16 fixture
/// Expected voxel count for INT16 test file.
pub const NIFTI_INT16_N_VOXELS: usize = 50;
/// Expected dimensions for INT16 test file.
pub const NIFTI_INT16_DIMS: [i16; 2] = [1, 50];
/// Sample values from INT16 test file: i * 100.
pub const NIFTI_INT16_VALUE_SAMPLES: &[(usize, f32)] = &[
    (0, 0.0),
    (25, 2500.0),
    (49, 4900.0),
];

/// Tolerance for value comparisons.
pub const NIFTI_VALUE_TOLERANCE: f32 = 1e-5;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn approx_eq(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn test_read_nifti1_3d_float32() {
        let path = PathBuf::from(NIFTI_3D_FLOAT32_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", NIFTI_3D_FLOAT32_PATH);
            return;
        }

        let nifti = read_real_nifti1(&path).expect("Failed to read NIfTI-1");

        // Verify dimensions
        assert_eq!(nifti.dims[0], NIFTI_3D_DIMS[0], "Expected {} dimensions", NIFTI_3D_DIMS[0]);
        assert_eq!(nifti.dims[1], NIFTI_3D_DIMS[1], "Expected {} in dim1", NIFTI_3D_DIMS[1]);

        // Verify data count
        assert_eq!(nifti.data.len(), NIFTI_3D_N_VOXELS, "Expected {} values", NIFTI_3D_N_VOXELS);

        // Verify datatype
        assert_eq!(nifti.datatype, NIFTI_TYPE_FLOAT32, "Expected FLOAT32 datatype");

        // Verify sample values
        for (idx, expected) in NIFTI_3D_VALUE_SAMPLES {
            assert!(
                approx_eq(nifti.data[*idx], *expected, NIFTI_VALUE_TOLERANCE),
                "Value {} mismatch: expected {}, got {}",
                idx,
                expected,
                nifti.data[*idx]
            );
        }
    }

    #[test]
    fn test_read_nifti1_3d_gzipped() {
        let path = PathBuf::from(NIFTI_3D_FLOAT32_GZ_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", NIFTI_3D_FLOAT32_GZ_PATH);
            return;
        }

        let nifti = read_real_nifti1(&path).expect("Failed to read gzipped NIfTI-1");

        // Verify same data as uncompressed version
        assert_eq!(nifti.data.len(), NIFTI_3D_N_VOXELS, "Expected {} values", NIFTI_3D_N_VOXELS);
        assert_eq!(nifti.datatype, NIFTI_TYPE_FLOAT32, "Expected FLOAT32 datatype");

        // Verify sample values match uncompressed
        for (idx, expected) in NIFTI_3D_VALUE_SAMPLES {
            assert!(
                approx_eq(nifti.data[*idx], *expected, NIFTI_VALUE_TOLERANCE),
                "Value {} mismatch: expected {}, got {}",
                idx,
                expected,
                nifti.data[*idx]
            );
        }
    }

    #[test]
    fn test_read_nifti1_4d() {
        let path = PathBuf::from(NIFTI_4D_FLOAT32_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", NIFTI_4D_FLOAT32_PATH);
            return;
        }

        let nifti = read_real_nifti1(&path).expect("Failed to read 4D NIfTI-1");

        // Verify dimensions
        assert_eq!(nifti.dims[0], NIFTI_4D_DIMS[0], "Expected {} dimensions", NIFTI_4D_DIMS[0]);
        for i in 1..=4 {
            assert_eq!(
                nifti.dims[i], NIFTI_4D_DIMS[i as usize],
                "dim[{}] mismatch", i
            );
        }

        // Verify data count
        assert_eq!(nifti.data.len(), NIFTI_4D_N_VOXELS, "Expected {} values", NIFTI_4D_N_VOXELS);

        // Verify sample values
        for (idx, expected) in NIFTI_4D_VALUE_SAMPLES {
            assert!(
                approx_eq(nifti.data[*idx], *expected, NIFTI_VALUE_TOLERANCE),
                "Value {} mismatch: expected {}, got {}",
                idx,
                expected,
                nifti.data[*idx]
            );
        }
    }

    #[test]
    fn test_read_nifti1_int16() {
        let path = PathBuf::from(NIFTI_INT16_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", NIFTI_INT16_PATH);
            return;
        }

        let nifti = read_real_nifti1(&path).expect("Failed to read INT16 NIfTI-1");

        // Verify dimensions
        assert_eq!(nifti.dims[0], NIFTI_INT16_DIMS[0], "Expected {} dimensions", NIFTI_INT16_DIMS[0]);
        assert_eq!(nifti.dims[1], NIFTI_INT16_DIMS[1], "Expected {} in dim1", NIFTI_INT16_DIMS[1]);

        // Verify data count
        assert_eq!(nifti.data.len(), NIFTI_INT16_N_VOXELS, "Expected {} values", NIFTI_INT16_N_VOXELS);

        // Verify datatype
        assert_eq!(nifti.datatype, NIFTI_TYPE_INT16, "Expected INT16 datatype");

        // Verify sample values (converted to f32)
        for (idx, expected) in NIFTI_INT16_VALUE_SAMPLES {
            assert!(
                approx_eq(nifti.data[*idx], *expected, NIFTI_VALUE_TOLERANCE),
                "Value {} mismatch: expected {}, got {}",
                idx,
                expected,
                nifti.data[*idx]
            );
        }
    }

    #[test]
    fn test_is_real_nifti1() {
        let path = PathBuf::from(NIFTI_3D_FLOAT32_PATH);
        if !path.exists() {
            return;
        }

        assert!(is_real_nifti1(&path), "Should detect as real NIfTI-1");
    }

    #[test]
    fn test_is_real_nifti1_gzipped() {
        let path = PathBuf::from(NIFTI_3D_FLOAT32_GZ_PATH);
        if !path.exists() {
            return;
        }

        assert!(is_real_nifti1(&path), "Should detect gzipped file as real NIfTI-1");
    }
}
