//! Helper functions for input event processing.
//!
//! These are pure utility functions that extract common patterns from
//! mouse and touch event handling, reducing code duplication in the
//! main component.

use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

use crate::types::MouseButton;

/// Extract canvas-relative coordinates from a mouse event.
///
/// Returns `Some((x, y))` if the event target is a canvas element,
/// `None` otherwise.
pub fn canvas_coords_from_mouse(ev: &web_sys::MouseEvent) -> Option<(f32, f32)> {
    let rect = ev
        .target()
        .and_then(|t| t.dyn_into::<HtmlCanvasElement>().ok())
        .map(|canvas| canvas.get_bounding_client_rect())?;

    let x = ev.client_x() as f32 - rect.left() as f32;
    let y = ev.client_y() as f32 - rect.top() as f32;
    Some((x, y))
}

/// Extract canvas-relative coordinates from a touch point.
///
/// Requires the bounding rect to be passed in (from the event target).
pub fn canvas_coords_from_touch(touch: &web_sys::Touch, rect: &web_sys::DomRect) -> (f32, f32) {
    let x = touch.client_x() as f32 - rect.left() as f32;
    let y = touch.client_y() as f32 - rect.top() as f32;
    (x, y)
}

/// Convert a mouse button code to our MouseButton enum.
pub fn mouse_button_from_code(button: i16) -> MouseButton {
    match button {
        0 => MouseButton::Left,
        1 => MouseButton::Middle,
        _ => MouseButton::Right,
    }
}

/// Calculate the distance between two touch points.
pub fn touch_distance(t1: &web_sys::Touch, t2: &web_sys::Touch) -> f32 {
    let dx = (t2.client_x() - t1.client_x()) as f32;
    let dy = (t2.client_y() - t1.client_y()) as f32;
    (dx * dx + dy * dy).sqrt()
}

/// Calculate the center point between two touch points.
pub fn touch_center(t1: &web_sys::Touch, t2: &web_sys::Touch) -> (f32, f32) {
    let center_x = (t1.client_x() + t2.client_x()) as f32 / 2.0;
    let center_y = (t1.client_y() + t2.client_y()) as f32 / 2.0;
    (center_x, center_y)
}

/// Calculate zoom delta from pinch gesture scale change.
///
/// Returns a wheel-like delta value: negative for zoom in, positive for zoom out.
pub fn pinch_zoom_delta(start_distance: f32, new_distance: f32) -> f32 {
    let scale_change = new_distance / start_distance;
    if scale_change > 1.0 {
        -0.5 * (scale_change - 1.0) // Zoom in
    } else {
        0.5 * (1.0 - scale_change) // Zoom out
    }
}

/// Get the bounding rect from a touch event's target canvas.
pub fn canvas_rect_from_touch(ev: &web_sys::TouchEvent) -> Option<web_sys::DomRect> {
    ev.target()
        .and_then(|t| t.dyn_into::<HtmlCanvasElement>().ok())
        .map(|canvas| canvas.get_bounding_client_rect())
}
