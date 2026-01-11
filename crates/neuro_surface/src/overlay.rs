use io_formats::statistics::StatisticData;

/// Range used for color mapping overlays.
#[derive(Debug, Clone, Copy)]
pub enum OverlayRange {
    /// Use the data's global min/max.
    Auto,
    /// Symmetric around zero with given half-range.
    Symmetric { max_abs: f32 },
    /// Explicit min/max values.
    Manual { min: f32, max: f32 },
}

/// Binding between an overlay volume and its visualization parameters.
#[derive(Debug, Clone)]
pub struct OverlayBinding {
    /// Underlying statistical data.
    pub data: StatisticData,
    /// Volume index inside `data`.
    pub volume_index: usize,
    /// Range configuration.
    pub range: OverlayRange,
}
