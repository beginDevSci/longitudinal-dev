use serde::{Deserialize, Serialize};

/// Colormap options for neuroimaging overlays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColormapType {
    #[serde(rename = "Viridis")]
    Viridis,
    #[serde(rename = "RdBu")]
    RdBu,
    #[serde(rename = "Hot")]
    Hot,
}

