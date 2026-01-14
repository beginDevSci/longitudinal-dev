use glam::{Mat4, Vec3};
use neuro_surface::BrainViewPreset;

use crate::traits::{MouseButton, MousePosition, ViewerEvent};

pub struct OrbitController {
    pub target: Vec3,
    pub distance: f32,
    pub theta: f32,
    pub phi: f32,
    is_dragging: bool,
    last_mouse: Option<MousePosition>,
}

impl Default for OrbitController {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 200.0,
            // Default to a slightly elevated lateral view (10°)
            // This shows most of the lateral surface while giving some depth
            // perception. Pure lateral (phi=0) can look too flat.
            theta: 0.0,
            phi: 0.17,  // ~10 degrees elevation
            is_dragging: false,
            last_mouse: None,
        }
    }
}

impl OrbitController {
    pub fn eye(&self) -> Vec3 {
        let x = self.distance * self.phi.cos() * self.theta.cos();
        let y = self.distance * self.phi.cos() * self.theta.sin();
        let z = self.distance * self.phi.sin();
        self.target + Vec3::new(x, y, z)
    }

    pub fn view_matrix(&self) -> Mat4 {
        // Use Z-up to match RAS coordinate convention (X=Right, Y=Anterior, Z=Superior)
        Mat4::look_at_rh(self.eye(), self.target, Vec3::Z)
    }

    pub fn handle_event(&mut self, event: &ViewerEvent) -> bool {
        match *event {
            ViewerEvent::MouseDown { position, button } => {
                if matches!(button, MouseButton::Left) {
                    self.is_dragging = true;
                    self.last_mouse = Some(position);
                    true
                } else {
                    false
                }
            }
            ViewerEvent::MouseUp { .. } => {
                self.is_dragging = false;
                self.last_mouse = None;
                true
            }
            ViewerEvent::MouseMove { position, delta } => {
                if self.is_dragging {
                    let (dx, dy) = delta;
                    let rot_speed = 0.005;
                    self.theta += dx * rot_speed;
                    self.phi = (self.phi + dy * rot_speed).clamp(-1.5, 1.5);
                    self.last_mouse = Some(position);
                    true
                } else {
                    false
                }
            }
            ViewerEvent::Wheel { delta_y } => {
                let zoom_factor = 1.0 + delta_y * 0.1;
                self.distance = (self.distance * zoom_factor).clamp(50.0, 500.0);
                true
            }
            _ => false,
        }
    }

    /// Set the camera to a preset view.
    pub fn set_preset(&mut self, preset: BrainViewPreset) {
        let (theta, phi) = preset.orbit_angles();
        self.theta = theta;
        self.phi = phi;
    }

    /// Get the minimum and maximum allowed elevation (phi) values.
    /// This prevents the camera from flipping upside-down.
    pub fn elevation_limits() -> (f32, f32) {
        (-1.5, 1.5)
    }

    /// Get the minimum and maximum allowed distance values.
    pub fn distance_limits() -> (f32, f32) {
        (50.0, 500.0)
    }
}
