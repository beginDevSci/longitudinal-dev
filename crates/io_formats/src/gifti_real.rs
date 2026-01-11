//! Real GIFTI format parser.
//!
//! GIFTI (Geometry Format for the Imaging Technologies and Neuroscience community)
//! is an XML-based format for brain surface data.
//!
//! This module supports:
//! - ASCII-encoded data arrays
//! - Base64Binary encoded data
//! - GZipBase64Binary encoded data
//!
//! ## Supported DataArray intents
//!
//! - `NIFTI_INTENT_POINTSET` (1008): Vertex coordinates
//! - `NIFTI_INTENT_TRIANGLE` (1009): Triangle indices
//! - `NIFTI_INTENT_SHAPE` (2005): Per-vertex scalar data (curvature, etc.)
//! - `NIFTI_INTENT_TIME_SERIES` (2001): Functional time series data
//! - `NIFTI_INTENT_LABEL` (1002): Per-vertex parcellation labels

use crate::error::FormatError;
use crate::geometry::BrainGeometry;
use crate::parcellation::{Parcellation, Region};
use crate::statistics::Hemisphere;

use base64::Engine;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// GIFTI encoding types
#[derive(Debug, Clone, Copy, PartialEq)]
enum GiftiEncoding {
    Ascii,
    Base64Binary,
    GZipBase64Binary,
}

/// GIFTI data types
#[derive(Debug, Clone, Copy)]
enum GiftiDataType {
    Float32,
    Int32,
    UInt8,
}


/// Parsed GIFTI DataArray
struct GiftiDataArray {
    intent: i32,
    #[allow(dead_code)]
    data_type: GiftiDataType,
    dim0: usize,
    dim1: Option<usize>,
    encoding: GiftiEncoding,
    data: Vec<u8>,
}

/// Check if a file starts with GIFTI XML declaration.
pub fn is_real_gifti(path: &Path) -> bool {
    if let Ok(file) = File::open(path) {
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        if reader.read_line(&mut line).is_ok() {
            return line.contains("<?xml") || line.contains("<GIFTI");
        }
    }
    false
}

/// Read a real GIFTI surface file containing POINTSET and TRIANGLE data.
pub fn read_real_gifti_surface(path: &Path) -> Result<BrainGeometry, FormatError> {
    let content = std::fs::read_to_string(path)?;
    let hemisphere = infer_hemisphere(path);
    read_gifti_surface_from_str(&content, hemisphere)
}

/// Read a GIFTI surface from in-memory bytes.
///
/// Same format as `read_real_gifti_surface` but from a byte slice instead of a file.
pub fn read_gifti_surface_bytes(bytes: &[u8], hemisphere: Hemisphere) -> Result<BrainGeometry, FormatError> {
    let content = String::from_utf8_lossy(bytes);
    read_gifti_surface_from_str(&content, hemisphere)
}

/// Internal helper to parse GIFTI surface from string content.
fn read_gifti_surface_from_str(content: &str, hemisphere: Hemisphere) -> Result<BrainGeometry, FormatError> {
    let arrays = parse_gifti_data_arrays(content)?;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for array in arrays {
        match array.intent {
            1008 => {
                // NIFTI_INTENT_POINTSET
                vertices = parse_float32_vertices(&array)?;
            }
            1009 => {
                // NIFTI_INTENT_TRIANGLE
                indices = parse_int32_triangles(&array)?;
            }
            _ => {}
        }
    }

    if vertices.is_empty() {
        return Err(FormatError::InvalidMagic {
            expected: "GIFTI surface with POINTSET",
            found: "no vertex data found".into(),
        });
    }

    // Compute normals (empty if no triangles)
    let normals = if indices.is_empty() {
        vec![[0.0f32; 3]; vertices.len()]
    } else {
        compute_vertex_normals(&vertices, &indices)
    };

    Ok(BrainGeometry {
        hemisphere,
        vertices,
        normals,
        indices,
    })
}

/// Read a real GIFTI functional file containing scalar data arrays.
///
/// Returns a vector of volumes, where each volume is a Vec<f32> of per-vertex values.
pub fn read_real_gifti_func(path: &Path) -> Result<Vec<Vec<f32>>, FormatError> {
    let content = std::fs::read_to_string(path)?;
    read_gifti_func_from_str(&content)
}

/// Read GIFTI functional data from in-memory bytes.
///
/// Same format as `read_real_gifti_func` but from a byte slice instead of a file.
pub fn read_gifti_func_bytes(bytes: &[u8]) -> Result<Vec<Vec<f32>>, FormatError> {
    let content = String::from_utf8_lossy(bytes);
    read_gifti_func_from_str(&content)
}

/// Internal helper to parse GIFTI functional data from string content.
fn read_gifti_func_from_str(content: &str) -> Result<Vec<Vec<f32>>, FormatError> {
    let arrays = parse_gifti_data_arrays(content)?;

    let mut volumes = Vec::new();

    for array in &arrays {
        // Accept SHAPE (2005) or TIME_SERIES (2001) intents
        if array.intent == 2005 || array.intent == 2001 {
            let values = decode_float32_array(array)?;
            volumes.push(values);
        }
    }

    if volumes.is_empty() {
        return Err(FormatError::InvalidMagic {
            expected: "GIFTI functional with SHAPE or TIME_SERIES",
            found: "no scalar data found".into(),
        });
    }

    Ok(volumes)
}

// ============================================================================
// GIFTI Label Parsing
// ============================================================================

/// Read a GIFTI label file (.label.gii) containing parcellation data.
///
/// GIFTI label files contain:
/// - A `LabelTable` element with region definitions (key, name, RGBA)
/// - A `DataArray` with `NIFTI_INTENT_LABEL` (1002) containing per-vertex labels
///
/// # Arguments
/// * `path` - Path to the .label.gii file
///
/// # Returns
/// A `Parcellation` structure with per-vertex labels and region definitions.
pub fn read_gifti_label(path: &Path) -> Result<Parcellation, FormatError> {
    let content = std::fs::read_to_string(path)?;
    let hemisphere = infer_hemisphere(path);
    let mut parcellation = read_gifti_label_from_str(&content, hemisphere)?;

    // Extract atlas name from file path (e.g., "lh.aparc.label.gii" -> "aparc")
    parcellation.atlas_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            // Handle patterns like "lh.aparc.label" -> "aparc"
            let s = s.strip_suffix(".label").unwrap_or(s);
            if let Some(pos) = s.find('.') {
                s[pos + 1..].to_string()
            } else {
                s.to_string()
            }
        });

    Ok(parcellation)
}

/// Read a GIFTI label file from in-memory bytes.
pub fn read_gifti_label_bytes(bytes: &[u8], hemisphere: Hemisphere) -> Result<Parcellation, FormatError> {
    let content = String::from_utf8_lossy(bytes);
    read_gifti_label_from_str(&content, hemisphere)
}

/// Internal helper to parse GIFTI label from string content.
fn read_gifti_label_from_str(content: &str, hemisphere: Hemisphere) -> Result<Parcellation, FormatError> {
    // Parse the LabelTable for region definitions
    let regions = parse_label_table(content)?;

    // Parse the DataArray for per-vertex labels
    let arrays = parse_gifti_data_arrays(content)?;

    let mut labels = Vec::new();
    for array in &arrays {
        if array.intent == 1002 {
            // NIFTI_INTENT_LABEL
            labels = decode_int32_labels(array)?;
            break;
        }
    }

    if labels.is_empty() {
        return Err(FormatError::InvalidMagic {
            expected: "GIFTI label with NIFTI_INTENT_LABEL",
            found: "no label data found".into(),
        });
    }

    Ok(Parcellation {
        hemisphere,
        labels,
        regions,
        atlas_name: None,
    })
}

/// Parse the LabelTable element from GIFTI XML.
///
/// LabelTable format:
/// ```xml
/// <LabelTable>
///   <Label Key="0" Red="0.0" Green="0.0" Blue="0.0" Alpha="0.0">unknown</Label>
///   <Label Key="1" Red="0.8" Green="0.4" Blue="0.2" Alpha="1.0">bankssts</Label>
/// </LabelTable>
/// ```
fn parse_label_table(content: &str) -> Result<Vec<Region>, FormatError> {
    let mut regions = Vec::new();

    // Find the LabelTable section
    let table_start = content.find("<LabelTable>").ok_or(FormatError::InvalidMagic {
        expected: "<LabelTable>",
        found: "element not found".into(),
    })?;

    let table_end = content[table_start..].find("</LabelTable>").ok_or(FormatError::InvalidMagic {
        expected: "</LabelTable>",
        found: "element not found".into(),
    })? + table_start;

    let table_xml = &content[table_start..table_end];

    // Parse each Label element
    let mut pos = 0;
    while let Some(label_start_offset) = table_xml[pos..].find("<Label ") {
        let label_start = pos + label_start_offset;

        // Find the closing tag
        if let Some(end_offset) = table_xml[label_start..].find("</Label>") {
            let label_end = label_start + end_offset + "</Label>".len();
            let label_xml = &table_xml[label_start..label_end];

            if let Ok(region) = parse_single_label(label_xml) {
                regions.push(region);
            }

            pos = label_end;
        } else {
            // Self-closing tag: <Label ... />
            if let Some(end_offset) = table_xml[label_start..].find("/>") {
                let label_end = label_start + end_offset + 2;
                let label_xml = &table_xml[label_start..label_end];

                if let Ok(region) = parse_single_label(label_xml) {
                    regions.push(region);
                }

                pos = label_end;
            } else {
                break;
            }
        }
    }

    Ok(regions)
}

/// Parse a single Label element.
fn parse_single_label(xml: &str) -> Result<Region, FormatError> {
    // Extract attributes
    let key = extract_label_attr_u32(xml, "Key")?;
    let red = extract_label_attr_f32(xml, "Red").unwrap_or(0.0);
    let green = extract_label_attr_f32(xml, "Green").unwrap_or(0.0);
    let blue = extract_label_attr_f32(xml, "Blue").unwrap_or(0.0);
    let alpha = extract_label_attr_f32(xml, "Alpha").unwrap_or(1.0);

    // Convert normalized floats to u8
    let rgba = [
        (red * 255.0).round() as u8,
        (green * 255.0).round() as u8,
        (blue * 255.0).round() as u8,
        (alpha * 255.0).round() as u8,
    ];

    // Extract the label name (content between > and </Label> or empty for self-closing)
    let name = if xml.contains("</Label>") {
        let start = xml.find('>').map(|i| i + 1).unwrap_or(0);
        let end = xml.find("</Label>").unwrap_or(xml.len());
        xml[start..end].trim().to_string()
    } else {
        // Self-closing tag, no name content
        format!("region_{}", key)
    };

    Ok(Region::new(key, name, rgba))
}

/// Extract a u32 attribute from a Label element.
fn extract_label_attr_u32(xml: &str, name: &str) -> Result<u32, FormatError> {
    let pattern = format!("{}=\"", name);
    if let Some(start) = xml.find(&pattern) {
        let start = start + pattern.len();
        if let Some(end) = xml[start..].find('"') {
            let value = &xml[start..start + end];
            return value.parse().map_err(|_| FormatError::InvalidMagic {
                expected: "integer attribute",
                found: value.to_string(),
            });
        }
    }
    Err(FormatError::InvalidMagic {
        expected: "attribute",
        found: format!("{} not found", name),
    })
}

/// Extract an f32 attribute from a Label element.
fn extract_label_attr_f32(xml: &str, name: &str) -> Result<f32, FormatError> {
    let pattern = format!("{}=\"", name);
    if let Some(start) = xml.find(&pattern) {
        let start = start + pattern.len();
        if let Some(end) = xml[start..].find('"') {
            let value = &xml[start..start + end];
            return value.parse().map_err(|_| FormatError::InvalidMagic {
                expected: "float attribute",
                found: value.to_string(),
            });
        }
    }
    Err(FormatError::InvalidMagic {
        expected: "attribute",
        found: format!("{} not found", name),
    })
}

/// Decode int32 labels from a DataArray.
fn decode_int32_labels(array: &GiftiDataArray) -> Result<Vec<u32>, FormatError> {
    let count = array.dim0;

    match array.encoding {
        GiftiEncoding::Ascii => {
            let text = String::from_utf8_lossy(&array.data);
            text.split_whitespace()
                .map(|s| s.parse::<i32>().map(|v| v as u32))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| FormatError::InvalidMagic {
                    expected: "int values",
                    found: e.to_string(),
                })
        }
        GiftiEncoding::Base64Binary | GiftiEncoding::GZipBase64Binary => {
            if array.data.len() != count * 4 {
                return Err(FormatError::SizeMismatch {
                    expected: count * 4,
                    actual: array.data.len(),
                });
            }

            let mut values = Vec::with_capacity(count);
            for chunk in array.data.chunks_exact(4) {
                let bytes: [u8; 4] = chunk.try_into().unwrap();
                values.push(i32::from_le_bytes(bytes) as u32);
            }
            Ok(values)
        }
    }
}

/// Check if a file is a GIFTI label file.
pub fn is_gifti_label(path: &Path) -> bool {
    // Check both extension pattern and file content
    let is_label_ext = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.contains(".label.gii") || n.ends_with(".label.gii"))
        .unwrap_or(false);

    if is_label_ext {
        return true;
    }

    // Also check for NIFTI_INTENT_LABEL in the file content
    if let Ok(content) = std::fs::read_to_string(path) {
        return content.contains("NIFTI_INTENT_LABEL") || content.contains("Intent=\"1002\"");
    }

    false
}

/// Parse all DataArray elements from GIFTI XML content.
fn parse_gifti_data_arrays(content: &str) -> Result<Vec<GiftiDataArray>, FormatError> {
    let mut arrays = Vec::new();

    // Find each DataArray element by looking for the end tag
    let mut pos = 0;
    while let Some(start_offset) = content[pos..].find("<DataArray") {
        let start = pos + start_offset;

        // Find the matching closing tag
        if let Some(end_offset) = content[start..].find("</DataArray>") {
            let array_end = start + end_offset + "</DataArray>".len();
            let array_xml = &content[start..array_end];

            match parse_single_data_array(array_xml) {
                Ok(array) => arrays.push(array),
                Err(e) => {
                    // Log parsing errors for debugging but continue trying other arrays
                    log::debug!("Failed to parse DataArray: {:?}", e);
                }
            }

            pos = array_end;
        } else {
            // No closing tag found, break
            break;
        }
    }

    Ok(arrays)
}

/// Parse a single DataArray element.
fn parse_single_data_array(xml: &str) -> Result<GiftiDataArray, FormatError> {
    // Extract attributes - Note: GIFTI uses Intent= (not intent=)
    let intent = extract_attr_i32(xml, "Intent")?;
    let data_type = extract_data_type(xml)?;
    let encoding = extract_encoding(xml)?;
    let dim0 = extract_attr_usize(xml, "Dim0")?;
    let dim1 = extract_attr_usize(xml, "Dim1").ok();

    // Extract data content
    let data = extract_data_content(xml, encoding)?;

    Ok(GiftiDataArray {
        intent,
        data_type,
        dim0,
        dim1,
        encoding,
        data,
    })
}

/// Extract an integer attribute value.
fn extract_attr_i32(xml: &str, name: &str) -> Result<i32, FormatError> {
    let pattern = format!("{}=\"", name);
    if let Some(start) = xml.find(&pattern) {
        let start = start + pattern.len();
        if let Some(end) = xml[start..].find('"') {
            let value = &xml[start..start + end];
            // Handle NIFTI intent names
            return match value {
                "NIFTI_INTENT_LABEL" => Ok(1002),
                "NIFTI_INTENT_POINTSET" => Ok(1008),
                "NIFTI_INTENT_TRIANGLE" => Ok(1009),
                "NIFTI_INTENT_SHAPE" => Ok(2005),
                "NIFTI_INTENT_TIME_SERIES" => Ok(2001),
                _ => value.parse().map_err(|_| FormatError::InvalidMagic {
                    expected: "integer attribute",
                    found: value.to_string(),
                }),
            };
        }
    }
    Err(FormatError::InvalidMagic {
        expected: "attribute",
        found: format!("{} not found", name),
    })
}

/// Extract a usize attribute value.
fn extract_attr_usize(xml: &str, name: &str) -> Result<usize, FormatError> {
    let pattern = format!("{}=\"", name);
    if let Some(start) = xml.find(&pattern) {
        let start = start + pattern.len();
        if let Some(end) = xml[start..].find('"') {
            let value = &xml[start..start + end];
            return value.parse().map_err(|_| FormatError::InvalidMagic {
                expected: "integer attribute",
                found: value.to_string(),
            });
        }
    }
    Err(FormatError::InvalidMagic {
        expected: "attribute",
        found: format!("{} not found", name),
    })
}

/// Extract the DataType attribute.
fn extract_data_type(xml: &str) -> Result<GiftiDataType, FormatError> {
    let pattern = "DataType=\"";
    if let Some(start) = xml.find(pattern) {
        let start = start + pattern.len();
        if let Some(end) = xml[start..].find('"') {
            let value = &xml[start..start + end];
            return match value {
                "NIFTI_TYPE_FLOAT32" => Ok(GiftiDataType::Float32),
                "NIFTI_TYPE_INT32" => Ok(GiftiDataType::Int32),
                "NIFTI_TYPE_UINT8" => Ok(GiftiDataType::UInt8),
                _ => Err(FormatError::InvalidMagic {
                    expected: "supported GIFTI data type",
                    found: value.to_string(),
                }),
            };
        }
    }
    Err(FormatError::InvalidMagic {
        expected: "DataType",
        found: "attribute not found".into(),
    })
}

/// Extract the Encoding attribute.
fn extract_encoding(xml: &str) -> Result<GiftiEncoding, FormatError> {
    let pattern = "Encoding=\"";
    if let Some(start) = xml.find(pattern) {
        let start = start + pattern.len();
        if let Some(end) = xml[start..].find('"') {
            let value = &xml[start..start + end];
            return match value {
                "ASCII" => Ok(GiftiEncoding::Ascii),
                "Base64Binary" => Ok(GiftiEncoding::Base64Binary),
                "GZipBase64Binary" => Ok(GiftiEncoding::GZipBase64Binary),
                _ => Err(FormatError::InvalidMagic {
                    expected: "supported GIFTI encoding",
                    found: value.to_string(),
                }),
            };
        }
    }
    Err(FormatError::InvalidMagic {
        expected: "Encoding",
        found: "attribute not found".into(),
    })
}

/// Extract the Data element content.
fn extract_data_content(xml: &str, encoding: GiftiEncoding) -> Result<Vec<u8>, FormatError> {
    let start = xml.find("<Data>").ok_or(FormatError::InvalidMagic {
        expected: "<Data>",
        found: "element not found".into(),
    })? + 6;

    let end = xml.find("</Data>").ok_or(FormatError::InvalidMagic {
        expected: "</Data>",
        found: "element not found".into(),
    })?;

    let content = xml[start..end].trim();

    match encoding {
        GiftiEncoding::Ascii => {
            // For ASCII, store the text content directly
            Ok(content.as_bytes().to_vec())
        }
        GiftiEncoding::Base64Binary => {
            // Decode base64
            let engine = base64::engine::general_purpose::STANDARD;
            engine
                .decode(content)
                .map_err(|e| FormatError::InvalidMagic {
                    expected: "valid base64",
                    found: e.to_string(),
                })
        }
        GiftiEncoding::GZipBase64Binary => {
            // Decode base64, then decompress gzip
            let engine = base64::engine::general_purpose::STANDARD;
            let compressed = engine
                .decode(content)
                .map_err(|e| FormatError::InvalidMagic {
                    expected: "valid base64",
                    found: e.to_string(),
                })?;

            let mut decoder = GzDecoder::new(&compressed[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        }
    }
}

/// Decode a float32 array from DataArray data.
fn decode_float32_array(array: &GiftiDataArray) -> Result<Vec<f32>, FormatError> {
    let count = array.dim0 * array.dim1.unwrap_or(1);

    match array.encoding {
        GiftiEncoding::Ascii => {
            let text = String::from_utf8_lossy(&array.data);
            let values: Result<Vec<f32>, _> = text
                .split_whitespace()
                .map(|s| s.parse::<f32>())
                .collect();
            values.map_err(|e| FormatError::InvalidMagic {
                expected: "float values",
                found: e.to_string(),
            })
        }
        GiftiEncoding::Base64Binary | GiftiEncoding::GZipBase64Binary => {
            // Data is already decoded binary
            if array.data.len() != count * 4 {
                return Err(FormatError::SizeMismatch {
                    expected: count * 4,
                    actual: array.data.len(),
                });
            }

            let mut values = Vec::with_capacity(count);
            for chunk in array.data.chunks_exact(4) {
                let bytes: [u8; 4] = chunk.try_into().unwrap();
                values.push(f32::from_le_bytes(bytes));
            }
            Ok(values)
        }
    }
}

/// Parse vertices from a POINTSET array.
fn parse_float32_vertices(array: &GiftiDataArray) -> Result<Vec<[f32; 3]>, FormatError> {
    let values = decode_float32_array(array)?;

    if values.len() % 3 != 0 {
        return Err(FormatError::InvalidMagic {
            expected: "vertex count divisible by 3",
            found: format!("{} values", values.len()),
        });
    }

    let mut vertices = Vec::with_capacity(values.len() / 3);
    for chunk in values.chunks_exact(3) {
        vertices.push([chunk[0], chunk[1], chunk[2]]);
    }
    Ok(vertices)
}

/// Parse triangle indices from a TRIANGLE array.
fn parse_int32_triangles(array: &GiftiDataArray) -> Result<Vec<[u32; 3]>, FormatError> {
    let count = array.dim0 * array.dim1.unwrap_or(1);

    let values: Vec<i32> = match array.encoding {
        GiftiEncoding::Ascii => {
            let text = String::from_utf8_lossy(&array.data);
            text.split_whitespace()
                .map(|s| s.parse::<i32>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| FormatError::InvalidMagic {
                    expected: "int values",
                    found: e.to_string(),
                })?
        }
        GiftiEncoding::Base64Binary | GiftiEncoding::GZipBase64Binary => {
            if array.data.len() != count * 4 {
                return Err(FormatError::SizeMismatch {
                    expected: count * 4,
                    actual: array.data.len(),
                });
            }

            let mut values = Vec::with_capacity(count);
            for chunk in array.data.chunks_exact(4) {
                let bytes: [u8; 4] = chunk.try_into().unwrap();
                values.push(i32::from_le_bytes(bytes));
            }
            values
        }
    };

    if values.len() % 3 != 0 {
        return Err(FormatError::InvalidMagic {
            expected: "triangle count divisible by 3",
            found: format!("{} values", values.len()),
        });
    }

    let mut triangles = Vec::with_capacity(values.len() / 3);
    for chunk in values.chunks_exact(3) {
        triangles.push([chunk[0] as u32, chunk[1] as u32, chunk[2] as u32]);
    }
    Ok(triangles)
}

/// Compute vertex normals from geometry.
fn compute_vertex_normals(vertices: &[[f32; 3]], faces: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; vertices.len()];

    for [i0, i1, i2] in faces {
        let v0 = vertices[*i0 as usize];
        let v1 = vertices[*i1 as usize];
        let v2 = vertices[*i2 as usize];

        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];

        for idx in [*i0 as usize, *i1 as usize, *i2 as usize] {
            normals[idx][0] += n[0];
            normals[idx][1] += n[1];
            normals[idx][2] += n[2];
        }
    }

    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-10 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        }
    }

    normals
}

/// Infer hemisphere from path.
fn infer_hemisphere(path: &Path) -> Hemisphere {
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        if stem.starts_with("lh.") || stem.starts_with("lh_") || stem == "lh" {
            return Hemisphere::Left;
        }
        if stem.starts_with("rh.") || stem.starts_with("rh_") || stem == "rh" {
            return Hemisphere::Right;
        }
    }
    Hemisphere::Left
}

// ============================================================================
// Ground-Truth Constants for GIFTI Test Fixtures
// ============================================================================

/// Path to ASCII GIFTI shape fixture (10 vertices, scalar values).
pub const GIFTI_SHAPE_ASCII_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/gifti_real/test_shape.gii"
);

/// Path to GZipBase64Binary GIFTI shape fixture.
pub const GIFTI_SHAPE_B64_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/gifti_real/test_shape_b64.gii"
);

/// Path to ASCII GIFTI surface fixture (cube: 8 vertices, 12 triangles).
pub const GIFTI_SURFACE_ASCII_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/gifti_real/test_surface.surf.gii"
);

/// Path to GZipBase64Binary GIFTI surface fixture.
pub const GIFTI_SURFACE_B64_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/gifti_real/test_surface_b64.surf.gii"
);

/// Path to GIFTI functional fixture (10 vertices, 3 time points).
pub const GIFTI_FUNC_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/gifti_real/test_func.func.gii"
);

// Ground truth for shape fixtures
/// Expected vertex count for shape test file.
pub const GIFTI_SHAPE_N_VERTICES: usize = 10;

// Ground truth for surface fixtures
/// Expected vertex count for surface test file (cube).
pub const GIFTI_SURFACE_N_VERTICES: usize = 8;
/// Expected triangle count for surface test file.
pub const GIFTI_SURFACE_N_TRIANGLES: usize = 12;
/// Sample vertex coordinates from surface test file.
pub const GIFTI_SURFACE_VERTEX_SAMPLES: &[(usize, [f32; 3])] = &[
    (0, [-1.0, -1.0, 1.0]),
    (7, [-1.0, 1.0, -1.0]),
];
/// Sample triangle indices from surface test file.
pub const GIFTI_SURFACE_TRIANGLE_SAMPLES: &[(usize, [u32; 3])] = &[
    (0, [0, 1, 2]),
    (11, [0, 7, 4]),
];

// Ground truth for functional fixtures
/// Expected vertex count for functional test file.
pub const GIFTI_FUNC_N_VERTICES: usize = 10;
/// Expected time point count for functional test file.
pub const GIFTI_FUNC_N_TIMEPOINTS: usize = 3;
/// Sample values from functional test file: sin(vertex * 0.5 + timepoint).
pub const GIFTI_FUNC_VALUE_SAMPLES: &[(usize, usize, f32)] = &[
    // (timepoint, vertex, value)
    (0, 0, 0.000000),
    (0, 5, 0.598472),
    (0, 9, -0.977530),
    (1, 0, 0.841471),
    (1, 5, -0.350783),
    (2, 0, 0.909297),
    (2, 9, 0.215120),
];

/// Tolerance for value comparisons.
pub const GIFTI_VALUE_TOLERANCE: f32 = 1e-5;
/// Tolerance for coordinate comparisons.
pub const GIFTI_COORD_TOLERANCE: f32 = 1e-4;

// Ground truth for label fixtures
/// Path to ASCII GIFTI label fixture (100 vertices, 5 regions).
pub const GIFTI_LABEL_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/gifti_real/lh.aparc.label.gii"
);
/// Expected vertex count for label test file.
pub const GIFTI_LABEL_N_VERTICES: usize = 100;
/// Expected region count for label test file.
pub const GIFTI_LABEL_N_REGIONS: usize = 5;
/// Sample label values: (vertex_index, expected_region_id).
pub const GIFTI_LABEL_SAMPLES: &[(usize, u32)] = &[
    (0, 1),   // First vertex, region 1
    (25, 2),  // Quarter way, region 2
    (50, 3),  // Halfway, region 3
    (75, 4),  // Three quarters, region 4
    (99, 0),  // Last vertex, region 0 (unknown)
];
/// Sample region definitions: (id, name, approximate RGB).
pub const GIFTI_LABEL_REGION_SAMPLES: &[(u32, &str, [u8; 3])] = &[
    (0, "unknown", [0, 0, 0]),
    (1, "bankssts", [25, 100, 40]),
    (2, "caudalanteriorcingulate", [125, 100, 160]),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn approx_eq(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() < tol
    }

    fn coords_approx_eq(a: [f32; 3], b: [f32; 3], tol: f32) -> bool {
        approx_eq(a[0], b[0], tol) && approx_eq(a[1], b[1], tol) && approx_eq(a[2], b[2], tol)
    }

    #[test]
    fn test_read_ascii_gifti_shape() {
        let path = PathBuf::from(GIFTI_SHAPE_ASCII_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", GIFTI_SHAPE_ASCII_PATH);
            return;
        }

        let volumes = read_real_gifti_func(&path).expect("Failed to read ASCII GIFTI");
        assert_eq!(volumes.len(), 1, "Expected 1 data array");
        assert_eq!(volumes[0].len(), GIFTI_SHAPE_N_VERTICES, "Expected {} vertices", GIFTI_SHAPE_N_VERTICES);

        // Verify values (0.0, 0.1, 0.2, ..., 0.9)
        for i in 0..GIFTI_SHAPE_N_VERTICES {
            let expected = i as f32 * 0.1;
            assert!(
                approx_eq(volumes[0][i], expected, GIFTI_VALUE_TOLERANCE),
                "Value {} mismatch: expected {}, got {}",
                i,
                expected,
                volumes[0][i]
            );
        }
    }

    #[test]
    fn test_read_b64_gifti_shape() {
        let path = PathBuf::from(GIFTI_SHAPE_B64_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", GIFTI_SHAPE_B64_PATH);
            return;
        }

        let volumes = read_real_gifti_func(&path).expect("Failed to read GZipBase64Binary GIFTI");
        assert_eq!(volumes.len(), 1, "Expected 1 data array");
        assert_eq!(volumes[0].len(), GIFTI_SHAPE_N_VERTICES, "Expected {} vertices", GIFTI_SHAPE_N_VERTICES);

        // Verify values match ASCII version
        for i in 0..GIFTI_SHAPE_N_VERTICES {
            let expected = i as f32 * 0.1;
            assert!(
                approx_eq(volumes[0][i], expected, GIFTI_VALUE_TOLERANCE),
                "Value {} mismatch: expected {}, got {}",
                i,
                expected,
                volumes[0][i]
            );
        }
    }

    #[test]
    fn test_read_ascii_gifti_surface() {
        let path = PathBuf::from(GIFTI_SURFACE_ASCII_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", GIFTI_SURFACE_ASCII_PATH);
            return;
        }

        let geom = read_real_gifti_surface(&path).expect("Failed to read ASCII GIFTI surface");

        // Verify counts
        assert_eq!(geom.vertices.len(), GIFTI_SURFACE_N_VERTICES, "Expected {} vertices", GIFTI_SURFACE_N_VERTICES);
        assert_eq!(geom.indices.len(), GIFTI_SURFACE_N_TRIANGLES, "Expected {} triangles", GIFTI_SURFACE_N_TRIANGLES);
        assert_eq!(geom.normals.len(), GIFTI_SURFACE_N_VERTICES, "Expected {} normals", GIFTI_SURFACE_N_VERTICES);

        // Verify sample vertices
        for (idx, expected_coords) in GIFTI_SURFACE_VERTEX_SAMPLES {
            assert!(
                coords_approx_eq(geom.vertices[*idx], *expected_coords, GIFTI_COORD_TOLERANCE),
                "Vertex {} mismatch: expected {:?}, got {:?}",
                idx,
                expected_coords,
                geom.vertices[*idx]
            );
        }

        // Verify sample triangles
        for (idx, expected_tri) in GIFTI_SURFACE_TRIANGLE_SAMPLES {
            assert_eq!(
                geom.indices[*idx], *expected_tri,
                "Triangle {} mismatch: expected {:?}, got {:?}",
                idx, expected_tri, geom.indices[*idx]
            );
        }
    }

    #[test]
    fn test_read_b64_gifti_surface() {
        let path = PathBuf::from(GIFTI_SURFACE_B64_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", GIFTI_SURFACE_B64_PATH);
            return;
        }

        let geom = read_real_gifti_surface(&path).expect("Failed to read GZipBase64Binary GIFTI surface");

        // Verify counts match ASCII version
        assert_eq!(geom.vertices.len(), GIFTI_SURFACE_N_VERTICES);
        assert_eq!(geom.indices.len(), GIFTI_SURFACE_N_TRIANGLES);

        // Verify sample vertices
        for (idx, expected_coords) in GIFTI_SURFACE_VERTEX_SAMPLES {
            assert!(
                coords_approx_eq(geom.vertices[*idx], *expected_coords, GIFTI_COORD_TOLERANCE),
                "Vertex {} mismatch: expected {:?}, got {:?}",
                idx,
                expected_coords,
                geom.vertices[*idx]
            );
        }
    }

    #[test]
    fn test_read_gifti_func() {
        let path = PathBuf::from(GIFTI_FUNC_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", GIFTI_FUNC_PATH);
            return;
        }

        let volumes = read_real_gifti_func(&path).expect("Failed to read GIFTI functional");

        // Verify counts
        assert_eq!(volumes.len(), GIFTI_FUNC_N_TIMEPOINTS, "Expected {} time points", GIFTI_FUNC_N_TIMEPOINTS);
        for (t, vol) in volumes.iter().enumerate() {
            assert_eq!(vol.len(), GIFTI_FUNC_N_VERTICES, "Time point {} should have {} vertices", t, GIFTI_FUNC_N_VERTICES);
        }

        // Verify sample values
        for (timepoint, vertex, expected) in GIFTI_FUNC_VALUE_SAMPLES {
            let actual = volumes[*timepoint][*vertex];
            assert!(
                approx_eq(actual, *expected, GIFTI_VALUE_TOLERANCE),
                "Value at t={}, v={} mismatch: expected {}, got {}",
                timepoint,
                vertex,
                expected,
                actual
            );
        }
    }

    #[test]
    fn test_is_real_gifti() {
        let path = PathBuf::from(GIFTI_SHAPE_ASCII_PATH);
        if !path.exists() {
            return;
        }

        assert!(is_real_gifti(&path), "Should detect as real GIFTI");
    }

    #[test]
    fn test_read_gifti_label() {
        let path = PathBuf::from(GIFTI_LABEL_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", GIFTI_LABEL_PATH);
            return;
        }

        let parcellation = read_gifti_label(&path).expect("Failed to read GIFTI label");

        // Verify counts
        assert_eq!(
            parcellation.labels.len(),
            GIFTI_LABEL_N_VERTICES,
            "Expected {} vertices",
            GIFTI_LABEL_N_VERTICES
        );
        assert_eq!(
            parcellation.regions.len(),
            GIFTI_LABEL_N_REGIONS,
            "Expected {} regions",
            GIFTI_LABEL_N_REGIONS
        );

        // Verify sample labels
        for (vertex, expected_label) in GIFTI_LABEL_SAMPLES {
            assert_eq!(
                parcellation.labels[*vertex], *expected_label,
                "Label at vertex {} mismatch: expected {}, got {}",
                vertex, expected_label, parcellation.labels[*vertex]
            );
        }

        // Verify sample regions
        for (id, expected_name, _expected_rgb) in GIFTI_LABEL_REGION_SAMPLES {
            let region = parcellation.get_region(*id).expect(&format!("Region {} not found", id));
            assert_eq!(
                region.name, *expected_name,
                "Region {} name mismatch: expected {}, got {}",
                id, expected_name, region.name
            );
        }

        // Verify hemisphere was inferred correctly
        assert_eq!(parcellation.hemisphere, Hemisphere::Left);

        // Verify atlas name was extracted
        assert_eq!(parcellation.atlas_name, Some("aparc".to_string()));
    }

    #[test]
    fn test_is_gifti_label() {
        let path = PathBuf::from(GIFTI_LABEL_PATH);
        if !path.exists() {
            return;
        }

        assert!(is_gifti_label(&path), "Should detect as GIFTI label file");
    }

    #[test]
    fn test_gifti_label_region_lookup() {
        let path = PathBuf::from(GIFTI_LABEL_PATH);
        if !path.exists() {
            return;
        }

        let parcellation = read_gifti_label(&path).unwrap();

        // Test vertex region lookup
        assert_eq!(parcellation.get_vertex_region_name(0), Some("bankssts"));
        assert_eq!(parcellation.get_vertex_region_name(25), Some("caudalanteriorcingulate"));
        assert_eq!(parcellation.get_vertex_region_name(50), Some("caudalmiddlefrontal"));
        assert_eq!(parcellation.get_vertex_region_name(75), Some("cuneus"));
        assert_eq!(parcellation.get_vertex_region_name(99), Some("unknown"));
    }
}
