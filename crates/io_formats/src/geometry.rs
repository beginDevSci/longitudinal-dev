use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

use crate::error::FormatError;
use crate::statistics::Hemisphere;

/// Format-agnostic surface mesh representation.
#[derive(Debug, Clone)]
pub struct BrainGeometry {
    /// Hemisphere this surface belongs to.
    pub hemisphere: Hemisphere,
    /// Vertex positions in world space.
    pub vertices: Vec<[f32; 3]>,
    /// Vertex normals.
    pub normals: Vec<[f32; 3]>,
    /// Triangle indices into `vertices`.
    pub indices: Vec<[u32; 3]>,
}

impl BrainGeometry {
    pub const MAGIC: &'static [u8; 4] = b"BRG1";
    pub const VERSION: u32 = 1;
    pub const SUPPORTED_FLAGS_MASK: u32 = 0b0000_0111;

    /// Parse a `BrainGeometry` from the custom BRG1 binary format.
    pub fn from_bytes(bytes: &[u8], hemisphere: Hemisphere) -> Result<Self, FormatError> {
        let mut cursor = Cursor::new(bytes);

        // Magic
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic)?;
        if &magic != Self::MAGIC {
            return Err(FormatError::InvalidMagic {
                expected: "BRG1",
                found: String::from_utf8_lossy(&magic).into(),
            });
        }

        // Version
        let version = cursor.read_u32::<LittleEndian>()?;
        if version != Self::VERSION {
            return Err(FormatError::UnsupportedVersion {
                expected: Self::VERSION,
                found: version,
            });
        }

        // Flags
        let flags = cursor.read_u32::<LittleEndian>()?;
        if (flags & !Self::SUPPORTED_FLAGS_MASK) != 0 {
            return Err(FormatError::UnsupportedFlags { flags });
        }

        let n_vertices = cursor.read_u32::<LittleEndian>()? as usize;
        let n_faces = cursor.read_u32::<LittleEndian>()? as usize;

        let header_bytes = 20;
        let expected_size = header_bytes + n_vertices * 24 + n_faces * 12;
        if bytes.len() != expected_size {
            return Err(FormatError::SizeMismatch {
                expected: expected_size,
                actual: bytes.len(),
            });
        }

        let mut vertices = Vec::with_capacity(n_vertices);
        for _ in 0..n_vertices {
            let x = cursor.read_f32::<LittleEndian>()?;
            let y = cursor.read_f32::<LittleEndian>()?;
            let z = cursor.read_f32::<LittleEndian>()?;
            vertices.push([x, y, z]);
        }

        let mut normals = Vec::with_capacity(n_vertices);
        for _ in 0..n_vertices {
            let nx = cursor.read_f32::<LittleEndian>()?;
            let ny = cursor.read_f32::<LittleEndian>()?;
            let nz = cursor.read_f32::<LittleEndian>()?;
            normals.push([nx, ny, nz]);
        }

        let mut indices = Vec::with_capacity(n_faces);
        for _ in 0..n_faces {
            let i = cursor.read_u32::<LittleEndian>()?;
            let j = cursor.read_u32::<LittleEndian>()?;
            let k = cursor.read_u32::<LittleEndian>()?;
            indices.push([i, j, k]);
        }

        Ok(BrainGeometry {
            hemisphere,
            vertices,
            normals,
            indices,
        })
    }
}
