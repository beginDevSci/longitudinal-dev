//! Brain view presets for standardized camera positions.
//!
//! Provides domain-specific camera views for neuroimaging applications,
//! based on the RAS (Right-Anterior-Superior) coordinate system.

use std::f32::consts::{FRAC_PI_2, PI};

/// Standard brain viewing angles.
///
/// These presets correspond to standard neuroimaging views based on
/// the RAS coordinate system where:
/// - X-axis: Right (+) / Left (-)
/// - Y-axis: Anterior (+) / Posterior (-)
/// - Z-axis: Superior (+) / Inferior (-)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrainViewPreset {
    /// View from the left side (looking at left hemisphere's lateral surface).
    LateralLeft,
    /// View from the right side (looking at right hemisphere's lateral surface).
    LateralRight,
    /// View from inside looking at left hemisphere's medial surface.
    MedialLeft,
    /// View from inside looking at right hemisphere's medial surface.
    MedialRight,
    /// View from above (looking down at the top of the brain).
    Dorsal,
    /// View from below (looking up at the bottom of the brain).
    Ventral,
    /// View from the front (looking at the face).
    Anterior,
    /// View from behind (looking at the back of the head).
    Posterior,
}

impl BrainViewPreset {
    /// Get all available presets.
    pub fn all() -> &'static [BrainViewPreset] {
        &[
            BrainViewPreset::LateralLeft,
            BrainViewPreset::LateralRight,
            BrainViewPreset::MedialLeft,
            BrainViewPreset::MedialRight,
            BrainViewPreset::Dorsal,
            BrainViewPreset::Ventral,
            BrainViewPreset::Anterior,
            BrainViewPreset::Posterior,
        ]
    }

    /// Get a human-readable name for the preset.
    pub fn name(&self) -> &'static str {
        match self {
            BrainViewPreset::LateralLeft => "Lateral Left",
            BrainViewPreset::LateralRight => "Lateral Right",
            BrainViewPreset::MedialLeft => "Medial Left",
            BrainViewPreset::MedialRight => "Medial Right",
            BrainViewPreset::Dorsal => "Dorsal",
            BrainViewPreset::Ventral => "Ventral",
            BrainViewPreset::Anterior => "Anterior",
            BrainViewPreset::Posterior => "Posterior",
        }
    }

    /// Get a short label for UI buttons.
    pub fn short_label(&self) -> &'static str {
        match self {
            BrainViewPreset::LateralLeft => "L-Lat",
            BrainViewPreset::LateralRight => "R-Lat",
            BrainViewPreset::MedialLeft => "L-Med",
            BrainViewPreset::MedialRight => "R-Med",
            BrainViewPreset::Dorsal => "Dor",
            BrainViewPreset::Ventral => "Ven",
            BrainViewPreset::Anterior => "Ant",
            BrainViewPreset::Posterior => "Pos",
        }
    }

    /// Get the orbit camera parameters (theta, phi) for this preset.
    ///
    /// Returns (azimuth/theta, elevation/phi) in radians where:
    /// - theta: horizontal rotation around the Z axis (0 = +X direction)
    /// - phi: vertical angle from horizontal plane (-π/2 to π/2)
    ///
    /// The camera uses a spherical coordinate system where:
    /// - theta=0, phi=0 looks from +X toward origin
    /// - theta increases counter-clockwise when viewed from above
    /// - phi > 0 looks from above, phi < 0 looks from below
    pub fn orbit_angles(&self) -> (f32, f32) {
        match self {
            // Lateral left: view from -X (looking at right hemisphere from left side)
            BrainViewPreset::LateralLeft => (PI, 0.0),
            // Lateral right: view from +X (looking at left hemisphere from right side)
            BrainViewPreset::LateralRight => (0.0, 0.0),
            // Medial left: view from +X but slightly rotated to see left medial
            BrainViewPreset::MedialLeft => (0.0, 0.0),
            // Medial right: view from -X to see right medial
            BrainViewPreset::MedialRight => (PI, 0.0),
            // Dorsal: view from +Z (looking down from above)
            BrainViewPreset::Dorsal => (0.0, FRAC_PI_2 - 0.01),
            // Ventral: view from -Z (looking up from below)
            BrainViewPreset::Ventral => (0.0, -FRAC_PI_2 + 0.01),
            // Anterior: view from +Y (looking at front/face)
            BrainViewPreset::Anterior => (FRAC_PI_2, 0.0),
            // Posterior: view from -Y (looking at back of head)
            BrainViewPreset::Posterior => (-FRAC_PI_2, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_presets() {
        let all = BrainViewPreset::all();
        assert_eq!(all.len(), 8);
    }

    #[test]
    fn test_preset_names() {
        assert_eq!(BrainViewPreset::LateralLeft.name(), "Lateral Left");
        assert_eq!(BrainViewPreset::Dorsal.name(), "Dorsal");
    }

    #[test]
    fn test_orbit_angles_range() {
        for preset in BrainViewPreset::all() {
            let (theta, phi) = preset.orbit_angles();
            assert!(theta >= -PI && theta <= PI);
            assert!(phi >= -FRAC_PI_2 && phi <= FRAC_PI_2);
        }
    }
}
