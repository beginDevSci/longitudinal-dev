use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::window;

use crate::renderer::BrainRendererBackend;

/// Type alias for the renderer stored value handle.
pub type RendererHandle = StoredValue<RefCell<Option<Box<dyn BrainRendererBackend>>>, LocalStorage>;

pub fn start_render_loop(renderer: RendererHandle) {
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    // Simple FPS monitoring in debug builds (logged to console).
    #[cfg(debug_assertions)]
    let last_time_ms = Rc::new(RefCell::new(js_sys::Date::now()));
    #[cfg(debug_assertions)]
    let frame_count = Rc::new(RefCell::new(0u32));

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // Use try_borrow_mut via StoredValue to avoid panic if renderer is borrowed elsewhere
        let should_continue = renderer.with_value(|cell| {
            if let Ok(mut guard) = cell.try_borrow_mut() {
                if let Some(r) = guard.as_mut() {
                    if r.is_running() {
                        r.render();

                        // Debug-only FPS logging.
                        #[cfg(debug_assertions)]
                        {
                            let now = js_sys::Date::now();
                            let mut last = last_time_ms.borrow_mut();
                            let mut count = frame_count.borrow_mut();
                            *count += 1;
                            let dt = now - *last;
                            if dt >= 1000.0 {
                                let fps = (*count as f64) * 1000.0 / dt;
                                log::debug!("Render FPS (approx): {:.1}", fps);
                                *last = now;
                                *count = 0;
                            }
                        }

                        true // Continue the render loop
                    } else {
                        false // Renderer stopped
                    }
                } else {
                    false // No renderer
                }
            } else {
                // RefCell is borrowed elsewhere, skip this frame but continue loop
                true
            }
        });

        if should_continue {
            if let Some(win) = window() {
                let cb = f.borrow();
                if let Some(cb_ref) = cb.as_ref() {
                    let _ = win.request_animation_frame(cb_ref.as_ref().unchecked_ref());
                }
            }
        }
    }) as Box<dyn FnMut()>));

    if let Some(win) = window() {
        if let Some(cb) = g.borrow().as_ref() {
            let _ = win.request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }
}
