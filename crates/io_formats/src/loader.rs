use gloo_net::http::Request;
use log::{error, info};

use crate::error::FormatError;
use crate::geometry::BrainGeometry;
use crate::statistics::{Analysis, Hemisphere, Statistic, StatisticData};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("network error: {0}")]
    Network(#[from] gloo_net::Error),
    #[error("http status {0}")]
    HttpStatus(u16),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("data error: {0}")]
    Data(#[from] FormatError),
}

fn decompress_gzip(bytes: &[u8]) -> Result<Vec<u8>, LoadError> {
    use flate2::bufread::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(bytes);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| LoadError::Parse(e.to_string()))?;
    Ok(out)
}

pub async fn load_geometry(
    base_path: &str,
    hemisphere: Hemisphere,
) -> Result<BrainGeometry, LoadError> {
    let hemi = hemisphere.as_str();
    let url = format!("{}/{}_geometry.bin.gz", base_path, hemi);
    info!("Loading geometry from: {}", url);

    let resp = Request::get(&url).send().await.map_err(|e| {
        error!("Failed to fetch geometry: {}", e);
        LoadError::Network(e)
    })?;

    if !resp.ok() {
        error!("Geometry HTTP error: status {}", resp.status());
        return Err(LoadError::HttpStatus(resp.status()));
    }

    let bytes = resp.binary().await.map_err(LoadError::Network)?;
    let decompressed = decompress_gzip(&bytes)?;
    let geom = BrainGeometry::from_bytes(&decompressed, hemisphere)?;

    info!("Loaded geometry: {} vertices, {} faces", geom.vertices.len(), geom.indices.len());
    Ok(geom)
}

pub async fn load_statistics(
    base_path: &str,
    hemisphere: Hemisphere,
    analysis: Analysis,
    statistic: Statistic,
) -> Result<StatisticData, LoadError> {
    let hemi = hemisphere.as_str();
    let ana = analysis.as_str();
    let stat = statistic.as_str();
    let url = format!("{}/{}_{}_{}.bin.gz", base_path, hemi, ana, stat);
    info!("Loading statistics from: {}", url);

    let resp = Request::get(&url).send().await.map_err(|e| {
        error!("Failed to fetch statistics: {}", e);
        LoadError::Network(e)
    })?;

    if !resp.ok() {
        error!("Statistics HTTP error: status {}", resp.status());
        return Err(LoadError::HttpStatus(resp.status()));
    }

    let bytes = resp.binary().await.map_err(LoadError::Network)?;
    let decompressed = decompress_gzip(&bytes)?;
    let data = StatisticData::from_bytes(&decompressed)?;

    info!(
        "Loaded statistics: {} vertices, {} volumes, range [{:.2}, {:.2}]",
        data.n_vertices, data.n_volumes, data.global_min, data.global_max
    );
    Ok(data)
}
