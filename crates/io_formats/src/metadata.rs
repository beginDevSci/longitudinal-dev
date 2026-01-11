use gloo_net::http::Request;

use crate::loader::LoadError;
use crate::statistics::{Analysis, Hemisphere, Statistic, StatisticMetadata};

pub async fn load_metadata(
    base_path: &str,
    hemisphere: Hemisphere,
    analysis: Analysis,
    statistic: Statistic,
) -> Result<StatisticMetadata, LoadError> {
    let hemi = hemisphere.as_str();
    let ana = analysis.as_str();
    let stat = statistic.as_str();
    let url = format!("{}/{}_{}_{}.json", base_path, hemi, ana, stat);

    let resp = Request::get(&url).send().await.map_err(LoadError::Network)?;
    if !resp.ok() {
        return Err(LoadError::HttpStatus(resp.status()));
    }
    let bytes = resp.binary().await.map_err(LoadError::Network)?;
    let text = String::from_utf8(bytes).map_err(|e| LoadError::Parse(e.to_string()))?;
    let meta: StatisticMetadata =
        serde_json::from_str(&text).map_err(|e| LoadError::Parse(e.to_string()))?;
    Ok(meta)
}
