use io_formats::geometry::BrainGeometry;
use io_formats::statistics::Hemisphere;

/// Surface for a single hemisphere.
#[derive(Debug, Clone)]
pub struct HemisphereSurface {
    /// Hemisphere identifier.
    pub hemisphere: Hemisphere,
    /// Surface geometry.
    pub geometry: BrainGeometry,
}

/// Combined brain surface representation used by the viewer.
#[derive(Debug, Clone, Default)]
pub struct BrainSurface {
    /// Left hemisphere surface, if loaded.
    pub left: Option<HemisphereSurface>,
    /// Right hemisphere surface, if loaded.
    pub right: Option<HemisphereSurface>,
}

impl BrainSurface {
    /// Iterate over available hemisphere surfaces (left first, then right).
    pub fn hemispheres(&self) -> impl Iterator<Item = &HemisphereSurface> {
        self.left.iter().chain(self.right.iter())
    }
}
