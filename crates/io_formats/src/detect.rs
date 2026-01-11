use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    FreeSurferSurface,
    FreeSurferCurv,
    Gifti,
    Nifti,
    /// Custom BLMM statistics format (BRS1 magic)
    BlmmStatistics,
    Unknown,
}

/// Check if file starts with BRS1 magic bytes for BLMM statistics format.
fn is_blmm_statistics(path: &Path) -> bool {
    use std::fs::File;
    use std::io::Read;

    if let Ok(mut file) = File::open(path) {
        let mut magic = [0u8; 4];
        if file.read_exact(&mut magic).is_ok() {
            return &magic == b"BRS1";
        }
    }
    false
}

/// Detect file format from byte content using magic bytes.
///
/// This is useful for web-fetched data where file extension is not available
/// or when the format needs to be detected from raw bytes.
pub fn detect_format_from_bytes(bytes: &[u8]) -> FileFormat {
    if bytes.len() < 4 {
        return FileFormat::Unknown;
    }

    // BRS1 magic (BLMM statistics)
    if &bytes[0..4] == b"BRS1" {
        return FileFormat::BlmmStatistics;
    }

    // NIfTI-1 magic: bytes 344-347 should contain "n+1\0" or "ni1\0"
    if bytes.len() >= 348 {
        let magic = &bytes[344..348];
        if magic == b"n+1\0" || magic == b"ni1\0" {
            return FileFormat::Nifti;
        }
    }

    // FreeSurfer surface magic: 0xFFFFFE (big-endian triangle magic)
    if bytes.len() >= 3 && bytes[0..3] == [0xFF, 0xFF, 0xFE] {
        return FileFormat::FreeSurferSurface;
    }

    // FreeSurfer curv magic: 0xFFFFFF (new curv format)
    if bytes.len() >= 3 && bytes[0..3] == [0xFF, 0xFF, 0xFF] {
        return FileFormat::FreeSurferCurv;
    }

    // GIFTI: XML file starting with <?xml or <GIFTI
    if bytes.len() >= 5 {
        let start = String::from_utf8_lossy(&bytes[..bytes.len().min(256)]);
        if start.contains("<?xml") || start.contains("<GIFTI") {
            return FileFormat::Gifti;
        }
    }

    FileFormat::Unknown
}

pub fn detect_format(path: &Path) -> FileFormat {
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if filename.ends_with(".nii.gz") {
        return FileFormat::Nifti;
    }

    // Check extension first
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    // For .bin or .bin.gz files, check magic bytes for BLMM format
    if ext == "bin" || filename.ends_with(".bin.gz") {
        if is_blmm_statistics(path) {
            return FileFormat::BlmmStatistics;
        }
    }

    match ext {
        "pial" | "white" | "inflated" => FileFormat::FreeSurferSurface,
        "curv" => FileFormat::FreeSurferCurv,
        "gii" => FileFormat::Gifti,
        "nii" => FileFormat::Nifti,
        _ => FileFormat::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_format, detect_format_from_bytes, FileFormat};
    use std::path::Path;

    #[test]
    fn detects_freesurfer_surface_by_ext() {
        assert_eq!(
            detect_format(Path::new("lh.pial")),
            FileFormat::FreeSurferSurface
        );
        assert_eq!(
            detect_format(Path::new("rh.white")),
            FileFormat::FreeSurferSurface
        );
    }

    #[test]
    fn detects_nifti_by_ext() {
        assert_eq!(detect_format(Path::new("foo.nii")), FileFormat::Nifti);
        assert_eq!(detect_format(Path::new("bar.nii.gz")), FileFormat::Nifti);
    }

    // ========================================================================
    // detect_format_from_bytes tests
    // ========================================================================

    #[test]
    fn detect_bytes_freesurfer_surface_magic() {
        // Real FreeSurfer surface magic: 0xFFFFFE (big-endian triangle magic)
        let bytes = [0xFF, 0xFF, 0xFE, 0x00, 0x00, 0x00];
        assert_eq!(
            detect_format_from_bytes(&bytes),
            FileFormat::FreeSurferSurface
        );
    }

    #[test]
    fn detect_bytes_freesurfer_curv_magic() {
        // Real FreeSurfer curv magic: 0xFFFFFF (new curv format)
        let bytes = [0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00];
        assert_eq!(detect_format_from_bytes(&bytes), FileFormat::FreeSurferCurv);
    }

    #[test]
    fn detect_bytes_blmm_statistics() {
        // BRS1 magic for BLMM statistics
        let bytes = b"BRS1\x01\x00\x00\x00";
        assert_eq!(detect_format_from_bytes(bytes), FileFormat::BlmmStatistics);
    }

    #[test]
    fn detect_bytes_nifti1_n_plus_1() {
        // NIfTI-1 with "n+1\0" magic at offset 344
        let mut bytes = vec![0u8; 352];
        bytes[344..348].copy_from_slice(b"n+1\0");
        assert_eq!(detect_format_from_bytes(&bytes), FileFormat::Nifti);
    }

    #[test]
    fn detect_bytes_nifti1_ni1() {
        // NIfTI-1 with "ni1\0" magic at offset 344
        let mut bytes = vec![0u8; 352];
        bytes[344..348].copy_from_slice(b"ni1\0");
        assert_eq!(detect_format_from_bytes(&bytes), FileFormat::Nifti);
    }

    #[test]
    fn detect_bytes_gifti_xml_declaration() {
        // GIFTI with XML declaration
        let bytes = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<GIFTI>";
        assert_eq!(detect_format_from_bytes(bytes), FileFormat::Gifti);
    }

    #[test]
    fn detect_bytes_gifti_direct_tag() {
        // GIFTI starting directly with <GIFTI tag
        let bytes = b"<GIFTI Version=\"1.0\">";
        assert_eq!(detect_format_from_bytes(bytes), FileFormat::Gifti);
    }

    #[test]
    fn detect_bytes_unknown_short() {
        // Too short to identify
        let bytes = [0x00, 0x01];
        assert_eq!(detect_format_from_bytes(&bytes), FileFormat::Unknown);
    }

    #[test]
    fn detect_bytes_unknown_random() {
        // Random data that doesn't match any format
        let bytes = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
        assert_eq!(detect_format_from_bytes(&bytes), FileFormat::Unknown);
    }

    #[test]
    fn detect_bytes_from_real_freesurfer_surface_fixture() {
        use crate::freesurfer_real::LH_PIAL_PATH;
        let path = std::path::Path::new(LH_PIAL_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", LH_PIAL_PATH);
            return;
        }
        let bytes = std::fs::read(path).expect("Failed to read fixture");
        assert_eq!(
            detect_format_from_bytes(&bytes),
            FileFormat::FreeSurferSurface,
            "Real FreeSurfer surface should be detected"
        );
    }

    #[test]
    fn detect_bytes_from_real_freesurfer_curv_fixture() {
        use crate::freesurfer_real::LH_CURV_PATH;
        let path = std::path::Path::new(LH_CURV_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", LH_CURV_PATH);
            return;
        }
        let bytes = std::fs::read(path).expect("Failed to read fixture");
        assert_eq!(
            detect_format_from_bytes(&bytes),
            FileFormat::FreeSurferCurv,
            "Real FreeSurfer curv should be detected"
        );
    }

    #[test]
    fn detect_bytes_from_real_nifti_fixture() {
        use crate::nifti_real::NIFTI_3D_FLOAT32_PATH;
        let path = std::path::Path::new(NIFTI_3D_FLOAT32_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", NIFTI_3D_FLOAT32_PATH);
            return;
        }
        let bytes = std::fs::read(path).expect("Failed to read fixture");
        assert_eq!(
            detect_format_from_bytes(&bytes),
            FileFormat::Nifti,
            "Real NIfTI should be detected"
        );
    }

    #[test]
    fn detect_bytes_from_real_gifti_surface_fixture() {
        use crate::gifti_real::GIFTI_SURFACE_ASCII_PATH;
        let path = std::path::Path::new(GIFTI_SURFACE_ASCII_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", GIFTI_SURFACE_ASCII_PATH);
            return;
        }
        let bytes = std::fs::read(path).expect("Failed to read fixture");
        assert_eq!(
            detect_format_from_bytes(&bytes),
            FileFormat::Gifti,
            "Real GIFTI surface should be detected"
        );
    }

    #[test]
    fn detect_bytes_from_real_gifti_func_fixture() {
        use crate::gifti_real::GIFTI_FUNC_PATH;
        let path = std::path::Path::new(GIFTI_FUNC_PATH);
        if !path.exists() {
            eprintln!("Skipping test: {} not found", GIFTI_FUNC_PATH);
            return;
        }
        let bytes = std::fs::read(path).expect("Failed to read fixture");
        assert_eq!(
            detect_format_from_bytes(&bytes),
            FileFormat::Gifti,
            "Real GIFTI functional should be detected"
        );
    }
}
