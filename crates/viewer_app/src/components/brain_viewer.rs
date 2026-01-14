//! Main BrainViewer component for 3D brain surface visualization.
//!
//! Provides an interactive 3D viewer for brain surface statistics with:
//! - Async loading with gzip decompression
//! - Hemisphere, statistic, and threshold controls
//! - Mouse drag/scroll, keyboard navigation, click-to-select interaction
//! - ARIA labels, focus ring, keyboard navigation for accessibility

mod canvas_info_overlay;
mod error_overlay;
mod focus_mode_header;
mod input_helpers;
mod loading_overlay;
mod url_state;

use canvas_info_overlay::CanvasInfoOverlay;
use error_overlay::ErrorOverlay;
use focus_mode_header::FocusModeHeader;
use loading_overlay::LoadingOverlay;

use std::cell::RefCell;

use leptos::callback::Callback;
use leptos::html::{Canvas, Div};
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;

use crate::data::{
    load_metadata, load_statistics, StatisticData, StatisticMetadata, VolumeLabel,
};
use crate::preferences;
use crate::renderer::{start_render_loop, BrainRendererBackend};
use crate::types::{
    Analysis, BrainViewPreset, ColormapType, Hemisphere, LayoutMode, ModifierKeys, MousePosition,
    Statistic, VertexInfo, ViewerEvent,
};

use super::analysis_tools_panel::AnalysisToolsPanel;

/// Find the first meaningful contrast index, skipping "Intercept" which is usually
/// not the most interesting effect to show by default.
fn find_first_meaningful_contrast(volumes: &[VolumeLabel]) -> u32 {
    // Look for the first volume that's not "Intercept"
    for vol in volumes {
        let label_lower = vol.label.to_lowercase();
        if !label_lower.contains("intercept") {
            return vol.index;
        }
    }
    // Fallback to index 0 if all are intercept-like or empty
    0
}

#[component]
pub fn BrainViewer(
    #[prop(default = "/data".to_string())] data_base_path: String,
    #[prop(optional)] parcellation_base_path: Option<String>,
) -> impl IntoView {
    // Load user preferences from localStorage (or defaults on non-WASM)
    let prefs = preferences::load_preferences();

    let (hemisphere, set_hemisphere) = signal(Hemisphere::Left);
    let (current_view, set_current_view) = signal(Option::<BrainViewPreset>::None);
    let (analysis, set_analysis) = signal(Analysis::Design1);
    let (statistic, set_statistic) = signal(Statistic::TStat);
    let (volume_idx, set_volume_idx) = signal(0u32);
    let (n_volumes, set_n_volumes) = signal(1u32);
    let (threshold, set_threshold) = signal(Option::<f32>::None);
    let (colormap, set_colormap) = signal(prefs.colormap);
    let (symmetric, set_symmetric) = signal(prefs.symmetric);
    let (layout, set_layout) = signal(prefs.layout);
    let (selected_vertex, set_selected_vertex) = signal(Option::<VertexInfo>::None);
    // Hover tracking - set in event handlers, display UI not yet implemented
    let (_hovered_vertex, set_hovered_vertex) = signal(Option::<VertexInfo>::None);
    let (is_loading, set_is_loading) = signal(true);
    let (loading_stage, set_loading_stage) = signal("Initializing...".to_string());
    let (loading_progress, set_loading_progress) = signal(0u8); // 0-100
    let (error, set_error) = signal(Option::<String>::None);
    let (can_retry, set_can_retry) = signal(false);
    let (stat_metadata, set_stat_metadata) = signal(Option::<StatisticMetadata>::None);
    let (stat_range, set_stat_range) = signal(Option::<(f32, f32)>::None);
    let (current_stats, set_current_stats) = signal(Option::<StatisticData>::None);
    let (initialized, set_initialized) = signal(false);
    let (region_selection_enabled, set_region_selection_enabled) =
        signal(prefs.region_selection_enabled);
    // Region hover tracking - set in event handlers, display UI not yet implemented
    let (_hovered_region, set_hovered_region) = signal(Option::<String>::None);
    let (selected_region, set_selected_region) = signal(Option::<String>::None);
    // Annotation tracking - populated from renderer, display UI not yet implemented
    let (_annotations, set_annotations) = signal(Vec::<VertexInfo>::new());
    let (share_link, set_share_link) = signal(Option::<String>::None);
    let (color_mode, set_color_mode) = signal(prefs.color_mode);
    let (parc_display_mode, set_parc_display_mode) = signal(prefs.parc_display_mode);
    // ROI feature signals - UI toggles not yet implemented, values read from prefs
    let (roi_drawing_enabled, _set_roi_drawing_enabled) = signal(false);
    let (roi_overlay_visible, _set_roi_overlay_visible) = signal(prefs.roi_overlay_visible);
    let (roi_brush_radius, _set_roi_brush_radius) = signal(2.0f32);
    let (_roi_vertex_count, set_roi_vertex_count) = signal(0usize);
    let (aria_status, set_aria_status) = signal(String::new());
    // Accessibility feature - UI toggle not yet implemented
    let (high_contrast, _set_high_contrast) = signal(false);
    // Picking state - tracked in poll loop, display indicator not yet implemented
    let (_is_picking, set_is_picking) = signal(false);
    let (is_fullscreen, set_is_fullscreen) = signal(false);
    let (is_focus_mode, set_is_focus_mode) = signal(false);
    let (show_interaction_hint, set_show_interaction_hint) = signal(true);

    // Touch gesture tracking for mobile
    let (touch_start_distance, set_touch_start_distance) = signal(Option::<f32>::None);
    let (last_touch_center, set_last_touch_center) = signal(Option::<(f32, f32)>::None);

    // Ref for the canvas container (used for fullscreen)
    let canvas_container_ref = NodeRef::<Div>::new();

    // Apply initial view state from URL query parameters (if present).
    // This runs once on component creation (WASM only).
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(search) = window.location().search() {
                let url_state = url_state::parse_url_query_params(&search);

                if let Some(h) = url_state.hemisphere {
                    set_hemisphere.set(h);
                }
                if let Some(s) = url_state.statistic {
                    set_statistic.set(s);
                }
                if let Some(v) = url_state.volume_idx {
                    set_volume_idx.set(v);
                }
                if let Some(l) = url_state.layout {
                    set_layout.set(l);
                }
                if let Some(t) = url_state.threshold {
                    set_threshold.set(Some(t));
                }
                if let Some(r) = url_state.region_selection_enabled {
                    set_region_selection_enabled.set(r);
                }
                if let Some(c) = url_state.color_mode {
                    set_color_mode.set(c);
                }
                if let Some(p) = url_state.parc_display_mode {
                    set_parc_display_mode.set(p);
                }
            }
        }
    }
    let canvas_ref = NodeRef::<Canvas>::new();

    // Store renderer in Leptos's local storage arena.
    // The handle is Send+Sync (can be captured by closures), but the contents
    // can be !Send (wgpu types). This is the idiomatic Leptos 0.8 pattern.
    let renderer = StoredValue::new_local(RefCell::new(None::<Box<dyn BrainRendererBackend>>));

    // Poll pending-pick state and completed picks periodically.
    // This drives the "Picking…" UI indicator and propagates async pick results to signals.
    #[cfg(target_arch = "wasm32")]
    {
        use std::cell::Cell;
        use std::rc::Rc;

        let set_is_picking_signal = set_is_picking.clone();
        let set_selected_vertex_poll = set_selected_vertex.clone();
        let set_selected_region_poll = set_selected_region.clone();
        let set_annotations_poll = set_annotations.clone();
        let set_aria_status_poll = set_aria_status.clone();
        let set_roi_vertex_count_poll = set_roi_vertex_count.clone();
        let roi_overlay_visible_poll = roi_overlay_visible.clone();
        let roi_drawing_enabled_poll = roi_drawing_enabled.clone();

        // Track when picking started for debounce (only show "Picking..." after 150ms)
        let picking_start_time: Rc<Cell<Option<f64>>> = Rc::new(Cell::new(None));
        let prev_shown_picking = Rc::new(Cell::new(false));
        let prev_roi_vertex_count: Rc<Cell<usize>> = Rc::new(Cell::new(0));
        const PICKING_DEBOUNCE_MS: f64 = 150.0;

        if let Some(window) = web_sys::window() {
            let cb = Closure::wrap(Box::new(move || {
                // Extract ALL data from the renderer closure (Leptos 0.8 pattern)
                // Returns: (is_picking, click_data, roi_vertex_count)
                let poll_result: Option<(
                    bool, // is_picking
                    Option<(
                        crate::types::VertexInfo,
                        Option<String>,
                        Vec<crate::types::VertexInfo>,
                    )>,
                    usize, // current ROI vertex count
                )> = renderer.with_value(|cell| {
                    if let Ok(mut guard) = cell.try_borrow_mut() {
                        if let Some(r) = guard.as_mut() {
                            let is_picking = r.has_pending_pick();

                            // Poll for completed pick results
                            let result = r.poll_completed_pick();
                            let mut click_data = None;
                            let mut roi_count = 0usize;

                            #[cfg(feature = "wgpu-renderer")]
                            {
                                use crate::renderer::wgpu_adapter::WgpuRendererAdapter;
                                let any = r.as_any_mut();
                                if let Some(wgpu_adapter) =
                                    any.downcast_mut::<WgpuRendererAdapter>()
                                {
                                    // Get current ROI vertex count (for live updates during painting)
                                    roi_count = wgpu_adapter.current_roi_vertex_count();

                                    // Process click result if any
                                    if let Some(ref info) = result.clicked {
                                        let region = info.surface_id.and_then(|sid| {
                                            wgpu_adapter.get_region_info(sid, info.index)
                                        });
                                        let annos: Vec<crate::types::VertexInfo> = wgpu_adapter
                                            .annotations()
                                            .iter()
                                            .map(|(_, v)| v.clone())
                                            .collect();
                                        click_data = Some((info.clone(), region, annos));
                                    }
                                }
                            }

                            return Some((is_picking, click_data, roi_count));
                        }
                    }
                    None
                });

                // Set signals outside the StoredValue closure
                if let Some((is_picking, click_data, roi_count)) = poll_result {
                    let now = js_sys::Date::now();

                    // Debounce the "Picking..." indicator to avoid flicker from quick hover picks
                    let should_show_picking = if is_picking {
                        match picking_start_time.get() {
                            None => {
                                // Just started picking, record the time
                                picking_start_time.set(Some(now));
                                false // Don't show yet
                            }
                            Some(start) => {
                                // Show only if picking for longer than debounce threshold
                                now - start >= PICKING_DEBOUNCE_MS
                            }
                        }
                    } else {
                        // Not picking, clear the start time
                        picking_start_time.set(None);
                        false
                    };

                    // Only update signal if the shown state changed
                    if should_show_picking != prev_shown_picking.get() {
                        prev_shown_picking.set(should_show_picking);
                        set_is_picking_signal.set(should_show_picking);
                    }

                    // Update ROI vertex count if it changed (for live painting feedback)
                    if roi_count != prev_roi_vertex_count.get() {
                        prev_roi_vertex_count.set(roi_count);
                        set_roi_vertex_count_poll.set(roi_count);

                        // Push ROI masks to GPU while drawing/overlay is visible for live feedback
                        if roi_overlay_visible_poll.get() || roi_drawing_enabled_poll.get() {
                            #[cfg(feature = "wgpu-renderer")]
                            {
                                use crate::renderer::wgpu_adapter::WgpuRendererAdapter;

                                renderer.update_value(|cell| {
                                    if let Ok(mut guard) = cell.try_borrow_mut() {
                                        if let Some(r) = guard.as_mut() {
                                            let any = r.as_any_mut();
                                            if let Some(wgpu_adapter) =
                                                any.downcast_mut::<WgpuRendererAdapter>()
                                            {
                                                wgpu_adapter.push_all_roi_masks_to_gpu();
                                            }
                                        }
                                    }
                                });
                            }
                        }
                    }

                    // Update selection signals if we got a click result
                    if let Some((info, region, annos)) = click_data {
                        let vertex_idx = info.index;
                        let hemi = info
                            .hemisphere()
                            .map(|h| match h {
                                crate::types::Hemisphere::Left => "LH",
                                crate::types::Hemisphere::Right => "RH",
                            })
                            .unwrap_or("");
                        let msg = match &region {
                            Some(name) => {
                                format!("Selected vertex {} {} in region {}", vertex_idx, hemi, name)
                            }
                            None => format!("Selected vertex {} {}", vertex_idx, hemi),
                        };
                        set_aria_status_poll.set(msg);
                        set_selected_region_poll.set(region);
                        set_selected_vertex_poll.set(Some(info));
                        set_annotations_poll.set(annos);
                    }
                }
            }) as Box<dyn FnMut()>);
            let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                50, // Poll every 50ms for responsive UI
            );
            cb.forget();
        }
    }

    // Initialize renderer on first render (simulating on_mount)
    let data_base_path_init = data_base_path.clone();
    let parcellation_base_path_init = parcellation_base_path.clone();
    Effect::new(move |_| {
        if initialized.get() {
            return;
        }

        let canvas = match canvas_ref.get() {
            Some(c) => c,
            None => return, // Canvas not ready yet
        };

        set_initialized.set(true);
        let canvas_el: HtmlCanvasElement = <HtmlCanvasElement as Clone>::clone(&canvas);
        let data_base_path = data_base_path_init.clone();
        let parcellation_base_path = parcellation_base_path_init.clone();
        // StoredValue handle is Copy, so captured directly
        spawn_local(async move {
            set_is_loading.set(true);
            set_error.set(None);
            set_can_retry.set(false);
            set_loading_stage.set("Initializing...".to_string());
            set_loading_progress.set(0);

            #[cfg(feature = "wgpu-renderer")]
            {
                use crate::renderer::WgpuRendererAdapter;
                use io_formats::statistics::Hemisphere as IoHemisphere;
                use neuro_surface::{BrainSurface, HemisphereSurface};

                // Stage 1: Load brain surface geometry
                set_loading_stage.set("Loading brain surface...".to_string());
                set_loading_progress.set(10);

                // Load both hemispheres' geometry for the wgpu renderer
                let lh_url = format!("{}/lh.pial", data_base_path);
                let rh_url = format!("{}/rh.pial", data_base_path);
                let (left_geom_result, right_geom_result) = futures::join!(
                    crate::data::fetch_surface(&lh_url, Hemisphere::Left),
                    crate::data::fetch_surface(&rh_url, Hemisphere::Right)
                );

                set_loading_progress.set(30);

                // Build BrainSurface from available hemispheres
                let mut brain_surface = BrainSurface::default();

                if let Ok(geom) = left_geom_result {
                    brain_surface.left = Some(HemisphereSurface {
                        hemisphere: IoHemisphere::Left,
                        geometry: geom,
                    });
                }

                if let Ok(geom) = right_geom_result {
                    brain_surface.right = Some(HemisphereSurface {
                        hemisphere: IoHemisphere::Right,
                        geometry: geom,
                    });
                }

                // Require at least one hemisphere to be loaded
                if brain_surface.left.is_none() && brain_surface.right.is_none() {
                    set_error.set(Some("Unable to load brain surface: No hemisphere geometry could be loaded. Please check that the surface files exist and are accessible.".to_string()));
                    set_can_retry.set(true);
                    set_is_loading.set(false);
                    return;
                }

                // Stage 2: Load statistics and metadata
                set_loading_stage.set("Loading statistics...".to_string());
                set_loading_progress.set(40);

                // Load statistics and metadata for the currently selected hemisphere
                // (overlay data is per-hemisphere for now)
                let (stats_result, meta_result) = futures::join!(
                    load_statistics(
                        &data_base_path,
                        hemisphere.get_untracked(),
                        analysis.get_untracked(),
                        statistic.get_untracked()
                    ),
                    load_metadata(
                        &data_base_path,
                        hemisphere.get_untracked(),
                        analysis.get_untracked(),
                        statistic.get_untracked()
                    )
                );

                set_loading_progress.set(60);

                let stats = match stats_result {
                    Ok(s) => s,
                    Err(e) => {
                        set_error.set(Some(format!("Unable to load statistics data: {}", e)));
                        set_can_retry.set(true);
                        set_is_loading.set(false);
                        return;
                    }
                };

                let meta = match meta_result {
                    Ok(m) => m,
                    Err(e) => {
                        set_error.set(Some(format!("Unable to load statistics metadata: {}", e)));
                        set_can_retry.set(true);
                        set_is_loading.set(false);
                        return;
                    }
                };

                // Update UI state with loaded data
                set_stat_metadata.set(Some(meta.clone()));
                set_n_volumes.set(stats.n_volumes as u32);
                set_current_stats.set(Some(stats.clone()));

                // Smart defaults: select first meaningful contrast (skip Intercept)
                let smart_volume_idx = find_first_meaningful_contrast(&meta.volumes);
                set_volume_idx.set(smart_volume_idx);

                // Smart defaults: apply suggested threshold from metadata
                let initial_threshold = meta.suggested_threshold;
                if let Some(t) = initial_threshold {
                    set_threshold.set(Some(t));
                }

                // Update range for the selected volume
                let range = stats
                    .volume_ranges
                    .get(smart_volume_idx as usize)
                    .cloned()
                    .unwrap_or((stats.global_min, stats.global_max));
                set_stat_range.set(Some(range));

                // Stage 3: Initialize renderer
                set_loading_stage.set("Initializing renderer...".to_string());
                set_loading_progress.set(70);

                match WgpuRendererAdapter::new(canvas_el.clone()).await {
                    Ok(mut r) => {
                        // Dual hemisphere mode: upload both hemispheres to the GPU
                        r.set_brain_surfaces(&brain_surface);

                        set_loading_progress.set(80);

                        // Try to load parcellations for both hemispheres, if available.
                        // This uses a default atlas ("aparc") under /parcellations; failures are non-fatal.
                        #[cfg(all(feature = "web-loaders", target_arch = "wasm32"))]
                        if let Some(parc_base) = parcellation_base_path.as_ref() {
                            set_loading_stage.set("Loading parcellations...".to_string());
                            let base_parc_url = format!(
                                "{}/parcellations",
                                parc_base.trim_end_matches('/')
                            );
                            if let Ok((lh_parc, rh_parc)) = crate::data::fetch_both_parcellations(
                                &base_parc_url,
                                "aparc",
                                "annot",
                            )
                            .await
                            {
                                let lh = neuro_surface::Parcellation::from_io_formats(&lh_parc);
                                let rh = neuro_surface::Parcellation::from_io_formats(&rh_parc);
                                r.set_parcellation(
                                    crate::renderer::wgpu_adapter::SURFACE_ID_LEFT,
                                    lh,
                                );
                                r.set_parcellation(
                                    crate::renderer::wgpu_adapter::SURFACE_ID_RIGHT,
                                    rh,
                                );
                            }
                        }

                        set_loading_stage.set("Finalizing...".to_string());
                        set_loading_progress.set(90);

                        r.set_overlay_from_stats(&stats, smart_volume_idx as usize, initial_threshold);
                        // Honor current hemisphere selection by hiding the other side.
                        r.show_hemisphere(hemisphere.get_untracked());

                        // Enable or disable region selection mode based on current UI state
                        r.set_region_selection_mode(region_selection_enabled.get_untracked());
                        renderer.update_value(|cell| *cell.borrow_mut() = Some(Box::new(r)));
                        start_render_loop(renderer);

                        set_loading_progress.set(100);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("WebGPU initialization failed: Your browser may not support WebGPU, or it may be disabled. Please try a recent version of Chrome, Edge, or Firefox. Details: {}", e)));
                        set_can_retry.set(true);
                    }
                }
                set_is_loading.set(false);
            }
        });
    });

    // Cleanup on unmount - StoredValue handle is Send+Sync so this works
    on_cleanup(move || {
        renderer.update_value(|cell| {
            if let Some(r) = cell.borrow_mut().as_mut() {
                r.stop();
            }
        });
    });

    // React to hemisphere changes - reload geometry
    let data_base_path_geom = data_base_path.clone();
    {
        Effect::new(move |prev_hem: Option<Hemisphere>| {
            let hem = hemisphere.get();
            // Skip if this is the first run or hemisphere hasn't changed
            if prev_hem.is_none() || prev_hem == Some(hem) {
                return hem;
            }

            let data_base_path = data_base_path_geom.clone();

            spawn_local(async move {
                set_is_loading.set(true);
                let geom_result = crate::data::fetch_surface(
                    &format!("{}/{}.pial", data_base_path, hem.as_str()),
                    hem,
                )
                .await;

                match geom_result {
                    Ok(geom) => {
                        renderer.update_value(|cell| {
                            if let Some(r) = cell.borrow_mut().as_mut() {
                                r.set_geometry(geom);
                            }
                        });
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Unable to load brain surface: {}", e)));
                    }
                }
                set_is_loading.set(false);
            });

            hem
        });
    }

    // Auto-adjust statistic when analysis changes (Compare only has Chi2/Chi2lp)
    {
        Effect::new(move |prev_analysis: Option<Analysis>| {
            let ana = analysis.get();
            let stat = statistic.get();

            // Skip if this is the first run
            if prev_analysis.is_none() {
                return ana;
            }

            // If switching to Compare and current stat isn't valid for Compare, switch to Chi2
            if ana == Analysis::Compare && stat.is_design_stat() {
                set_statistic.set(Statistic::Chi2);
            }
            // If switching away from Compare and current stat is Compare-only, switch to TStat
            else if ana != Analysis::Compare && stat.is_compare_only() {
                set_statistic.set(Statistic::TStat);
            }

            ana
        });
    }

    // React to statistics changes - reload statistics
    let data_base_path_stats = data_base_path.clone();
    {
        Effect::new(move |prev: Option<(Hemisphere, Analysis, Statistic)>| {
            let current = (hemisphere.get(), analysis.get(), statistic.get());

            // Skip if this is the first run or nothing changed
            if prev.is_none() || prev == Some(current) {
                return current;
            }

            let (hem, ana, stat) = current;
            let data_base_path = data_base_path_stats.clone();
            spawn_local(async move {
                set_is_loading.set(true);
                let (stats_result, meta_result) = futures::join!(
                    load_statistics(&data_base_path, hem, ana, stat),
                    load_metadata(&data_base_path, hem, ana, stat)
                );

                // Apply smart defaults when both stats and metadata are available
                if let (Ok(stats), Ok(meta)) = (&stats_result, &meta_result) {
                    set_stat_metadata.set(Some(meta.clone()));
                    set_n_volumes.set(stats.n_volumes as u32);

                    // Smart defaults: select first meaningful contrast (skip Intercept)
                    let smart_volume_idx = find_first_meaningful_contrast(&meta.volumes);
                    set_volume_idx.set(smart_volume_idx);

                    // Smart defaults: apply suggested threshold from metadata
                    if let Some(t) = meta.suggested_threshold {
                        set_threshold.set(Some(t));
                    }

                    // Use per-volume range for the selected volume
                    let range = stats
                        .volume_ranges
                        .get(smart_volume_idx as usize)
                        .cloned()
                        .unwrap_or((stats.global_min, stats.global_max));
                    set_stat_range.set(Some(range));
                    set_current_stats.set(Some(stats.clone()));

                    renderer.update_value(|cell| {
                        if let Some(r) = cell.borrow_mut().as_mut() {
                            r.set_statistics(stats.clone());
                            // Ensure only the selected hemisphere is shown after reload.
                            #[cfg(feature = "wgpu-renderer")]
                            {
                                use crate::renderer::wgpu_adapter::WgpuRendererAdapter;
                                let any = r.as_any_mut();
                                if let Some(wgpu_adapter) =
                                    any.downcast_mut::<WgpuRendererAdapter>()
                                {
                                    wgpu_adapter.show_hemisphere(hem);
                                }
                            }
                        }
                    });
                } else if let Ok(stats) = stats_result {
                    // Fallback: stats loaded but metadata failed - use index 0
                    set_n_volumes.set(stats.n_volumes as u32);
                    set_volume_idx.set(0);
                    let range = stats
                        .volume_ranges
                        .first()
                        .cloned()
                        .unwrap_or((stats.global_min, stats.global_max));
                    set_stat_range.set(Some(range));
                    set_current_stats.set(Some(stats.clone()));
                    renderer.update_value(|cell| {
                        if let Some(r) = cell.borrow_mut().as_mut() {
                            r.set_statistics(stats);
                        }
                    });
                }

                set_is_loading.set(false);
            });

            current
        });
    }

    // React to volume / threshold changes
    {
        Effect::new(move |_| {
            let vol = volume_idx.get();
            let thr = threshold.get();  // Read threshold unconditionally to ensure tracking
            renderer.update_value(|cell| {
                if let Some(r) = cell.borrow_mut().as_mut() {
                    r.set_volume(vol);
                    r.set_threshold(thr);
                }
            });
            // Update stat_range when volume changes
            if let Some(stats) = current_stats.get() {
                let range = stats
                    .volume_ranges
                    .get(vol as usize)
                    .cloned()
                    .unwrap_or((stats.global_min, stats.global_max));
                set_stat_range.set(Some(range));
            }
        });
    }

    // React to colormap changes
    {
        Effect::new(move |_| {
            let cm = colormap.get();
            renderer.update_value(|cell| {
                if let Some(r) = cell.borrow_mut().as_mut() {
                    r.set_colormap(cm);
                }
            });
        });
    }

    // React to symmetric toggle changes
    {
        Effect::new(move |_| {
            let sym = symmetric.get();
            renderer.update_value(|cell| {
                if let Some(r) = cell.borrow_mut().as_mut() {
                    r.set_symmetric(sym);
                }
            });
        });
    }

    // React to layout changes
    {
        Effect::new(move |_| {
            let mode = layout.get();
            renderer.update_value(|cell| {
                if let Some(r) = cell.borrow_mut().as_mut() {
                    r.set_layout(mode);
                }
            });
        });
    }

    // React to color mode changes (overlay vs parcellation)
    {
        let color_mode = color_mode.clone();
        Effect::new(move |_| {
            let mode = color_mode.get();
            #[cfg(feature = "wgpu-renderer")]
            {
                use crate::renderer::wgpu_adapter::WgpuRendererAdapter;
                use core_render::ColorSource;

                renderer.update_value(|cell| {
                    if let Some(r) = cell.borrow_mut().as_mut() {
                        let any = r.as_any_mut();
                        if let Some(wgpu_adapter) = any.downcast_mut::<WgpuRendererAdapter>() {
                            match mode.as_str() {
                                "parcellation" => {
                                    // Ensure parcellation colors are uploaded before switching.
                                    wgpu_adapter.set_parcellation_colors_from_stored();
                                    wgpu_adapter.set_color_source(ColorSource::Parcellation);
                                }
                                _ => {
                                    wgpu_adapter.set_color_source(ColorSource::Overlay);
                                }
                            }
                        }
                    }
                });
            }
        });
    }

    // React to hemisphere changes: ensure only that hemisphere's surface is shown.
    {
        Effect::new(move |_| {
            let hemi = hemisphere.get();
            #[cfg(feature = "wgpu-renderer")]
            {
                use crate::renderer::wgpu_adapter::WgpuRendererAdapter;
                renderer.update_value(|cell| {
                    if let Some(r) = cell.borrow_mut().as_mut() {
                        let any = r.as_any_mut();
                        if let Some(wgpu_adapter) = any.downcast_mut::<WgpuRendererAdapter>() {
                            wgpu_adapter.show_hemisphere(hemi);
                        }
                    }
                });
            }
        });
    }

    // React to parcellation display mode changes
    {
        let parc_display_mode = parc_display_mode.clone();
        Effect::new(move |_| {
            let mode = parc_display_mode.get();
            #[cfg(feature = "wgpu-renderer")]
            {
                use crate::renderer::wgpu_adapter::WgpuRendererAdapter;
                use core_render::ParcellationDisplay;

                renderer.update_value(|cell| {
                    if let Some(r) = cell.borrow_mut().as_mut() {
                        let any = r.as_any_mut();
                        if let Some(wgpu_adapter) = any.downcast_mut::<WgpuRendererAdapter>() {
                            let display = match mode.as_str() {
                                "edges" => ParcellationDisplay::Edges,
                                "fill_edges" => ParcellationDisplay::FillAndEdges,
                                _ => ParcellationDisplay::Fill,
                            };
                            wgpu_adapter.set_parcellation_display(display);
                        }
                    }
                });
            }
        });
    }

    // React to region selection mode changes (wgpu renderer only)
    {
        Effect::new(move |_| {
            let enabled = region_selection_enabled.get();
            #[cfg(feature = "wgpu-renderer")]
            {
                use crate::renderer::wgpu_adapter::WgpuRendererAdapter;

                renderer.update_value(|cell| {
                    if let Some(r) = cell.borrow_mut().as_mut() {
                        let any = r.as_any_mut();
                        if let Some(wgpu_adapter) = any.downcast_mut::<WgpuRendererAdapter>() {
                            wgpu_adapter.set_region_selection_mode(enabled);
                        }
                    }
                });
            }
        });
    }

    // React to ROI drawing mode changes (wgpu)
    {
        Effect::new(move |_| {
            let enabled = roi_drawing_enabled.get();
            #[cfg(feature = "wgpu-renderer")]
            {
                use crate::renderer::wgpu_adapter::WgpuRendererAdapter;

                renderer.update_value(|cell| {
                    if let Some(r) = cell.borrow_mut().as_mut() {
                        let any = r.as_any_mut();
                        if let Some(wgpu_adapter) = any.downcast_mut::<WgpuRendererAdapter>() {
                            wgpu_adapter.set_roi_drawing_mode(enabled);
                        }
                    }
                });
            }
        });
    }

    // React to ROI overlay visibility changes (wgpu)
    {
        Effect::new(move |_| {
            let visible = roi_overlay_visible.get();
            let drawing = roi_drawing_enabled.get();
            #[cfg(feature = "wgpu-renderer")]
            {
                use crate::renderer::wgpu_adapter::WgpuRendererAdapter;

                renderer.update_value(|cell| {
                    if let Some(r) = cell.borrow_mut().as_mut() {
                        let any = r.as_any_mut();
                        if let Some(wgpu_adapter) = any.downcast_mut::<WgpuRendererAdapter>() {
                            wgpu_adapter.set_roi_visualization_enabled(visible || drawing);
                        }
                    }
                });
            }
        });
    }

    // React to ROI brush radius changes (wgpu)
    {
        Effect::new(move |_| {
            let radius = roi_brush_radius.get();
            #[cfg(feature = "wgpu-renderer")]
            {
                use crate::renderer::wgpu_adapter::WgpuRendererAdapter;

                renderer.update_value(|cell| {
                    if let Some(r) = cell.borrow_mut().as_mut() {
                        let any = r.as_any_mut();
                        if let Some(wgpu_adapter) = any.downcast_mut::<WgpuRendererAdapter>() {
                            wgpu_adapter.set_brush_radius(radius);
                        }
                    }
                });
            }
        });
    }

    // -------------------------------------------------------------------------
    // Preference persistence effects: save to localStorage when signals change
    // -------------------------------------------------------------------------

    // Save colormap preference
    {
        let colormap = colormap.clone();
        Effect::new(move |prev: Option<ColormapType>| {
            let cm = colormap.get();
            // Skip initial run
            if prev.is_some() {
                preferences::save_colormap(cm);
            }
            cm
        });
    }

    // Save symmetric preference
    {
        let symmetric = symmetric.clone();
        Effect::new(move |prev: Option<bool>| {
            let sym = symmetric.get();
            if prev.is_some() {
                preferences::save_symmetric(sym);
            }
            sym
        });
    }

    // Save layout preference
    {
        let layout = layout.clone();
        Effect::new(move |prev: Option<LayoutMode>| {
            let mode = layout.get();
            if prev.is_some() {
                preferences::save_layout(mode);
            }
            mode
        });
    }

    // Save color mode preference
    {
        let color_mode = color_mode.clone();
        Effect::new(move |prev: Option<String>| {
            let mode = color_mode.get();
            if prev.is_some() {
                preferences::save_color_mode(&mode);
            }
            mode
        });
    }

    // Save parcellation display mode preference
    {
        let parc_display_mode = parc_display_mode.clone();
        Effect::new(move |prev: Option<String>| {
            let mode = parc_display_mode.get();
            if prev.is_some() {
                preferences::save_parc_display_mode(&mode);
            }
            mode
        });
    }

    // Save region selection enabled preference
    {
        let region_selection_enabled = region_selection_enabled.clone();
        Effect::new(move |prev: Option<bool>| {
            let enabled = region_selection_enabled.get();
            if prev.is_some() {
                preferences::save_region_selection_enabled(enabled);
            }
            enabled
        });
    }

    // Save ROI overlay visible preference
    {
        let roi_overlay_visible = roi_overlay_visible.clone();
        Effect::new(move |prev: Option<bool>| {
            let visible = roi_overlay_visible.get();
            if prev.is_some() {
                preferences::save_roi_overlay_visible(visible);
            }
            visible
        });
    }

    // Mouse/keyboard handlers
    let handle_canvas_click = {
        move |ev: leptos::ev::MouseEvent| {
            if let Some((x, y)) = input_helpers::canvas_coords_from_mouse(&ev) {
                let modifiers = ModifierKeys::from_mouse_event(&ev);
                // Queue the click event - async pick results are propagated via polling
                renderer.with_value(|cell| {
                    if let Some(r) = cell.borrow_mut().as_mut() {
                        r.handle_event(ViewerEvent::Click {
                            position: MousePosition { x, y },
                            modifiers,
                        });
                    }
                });
            }
        }
    };

    let handle_mouse_down = {
        move |ev: leptos::ev::MouseEvent| {
            if let Some((x, y)) = input_helpers::canvas_coords_from_mouse(&ev) {
                let button = input_helpers::mouse_button_from_code(ev.button());
                let modifiers = ModifierKeys::from_mouse_event(&ev);
                renderer.update_value(|cell| {
                    if let Ok(mut guard) = cell.try_borrow_mut() {
                        if let Some(r) = guard.as_mut() {
                            r.handle_event(ViewerEvent::MouseDown {
                                position: MousePosition { x, y },
                                button,
                                modifiers,
                            });
                        }
                    }
                });
            }
        }
    };

    let handle_mouse_up = {
        move |ev: leptos::ev::MouseEvent| {
            if let Some((x, y)) = input_helpers::canvas_coords_from_mouse(&ev) {
                let button = input_helpers::mouse_button_from_code(ev.button());
                let modifiers = ModifierKeys::from_mouse_event(&ev);
                renderer.update_value(|cell| {
                    if let Ok(mut guard) = cell.try_borrow_mut() {
                        if let Some(r) = guard.as_mut() {
                            r.handle_event(ViewerEvent::MouseUp {
                                position: MousePosition { x, y },
                                button,
                                modifiers,
                            });
                        }
                    }
                });
            }
        }
    };

    let (last_mouse_pos, set_last_mouse_pos) = signal(Option::<(f32, f32)>::None);

    let handle_mouse_move = {
        move |ev: leptos::ev::MouseEvent| {
            let rect = ev
                .target()
                .and_then(|t| t.dyn_into::<HtmlCanvasElement>().ok())
                .map(|canvas| canvas.get_bounding_client_rect());
            if let Some(rect) = rect {
                let x = ev.client_x() as f32 - rect.left() as f32;
                let y = ev.client_y() as f32 - rect.top() as f32;

                let delta = if let Some((lx, ly)) = last_mouse_pos.get() {
                    (x - lx, y - ly)
                } else {
                    (0.0, 0.0)
                };

                set_last_mouse_pos.set(Some((x, y)));

                // Extract hover data from closure
                let hover_data: Option<(VertexInfo, Option<String>)> =
                    renderer.with_value(|cell| {
                        match cell.try_borrow_mut() {
                            Ok(mut guard) => {
                                if let Some(r) = guard.as_mut() {
                                    let result = r.handle_event(ViewerEvent::MouseMove {
                                        position: MousePosition { x, y },
                                        delta,
                                    });
                                    if let Some(info) = result.hovered.clone() {
                                        let mut region: Option<String> = None;
                                        if let Some(surface_id) = info.surface_id {
                                            #[cfg(feature = "wgpu-renderer")]
                                            {
                                                use crate::renderer::wgpu_adapter::WgpuRendererAdapter;
                                                let any = r.as_any_mut();
                                                if let Some(wgpu_adapter) =
                                                    any.downcast_mut::<WgpuRendererAdapter>()
                                                {
                                                    region = wgpu_adapter
                                                        .get_region_info(surface_id, info.index);
                                                }
                                            }
                                        }
                                        return Some((info, region));
                                    }
                                }
                            }
                            Err(_) => {
                                // Borrow conflict - skip this frame but don't log (too noisy)
                            }
                        }
                        None
                    });

                // Set signals outside the closure
                match hover_data {
                    Some((info, region)) => {
                        set_hovered_region.set(region);
                        set_hovered_vertex.set(Some(info));
                    }
                    None => {
                        set_hovered_vertex.set(None);
                        set_hovered_region.set(None);
                    }
                }
            }
        }
    };

    let handle_wheel = {
        move |ev: leptos::ev::WheelEvent| {
            ev.prevent_default();
            let delta_y = ev.delta_y() as f32 / 100.0; // Normalize scroll
            renderer.update_value(|cell| {
                if let Some(r) = cell.borrow_mut().as_mut() {
                    r.handle_event(ViewerEvent::Wheel { delta_y });
                }
            });
        }
    };

    // Touch event handlers for mobile gestures
    // Uses native web_sys::TouchEvent for touch list access
    let handle_touch_start = {
        move |ev: leptos::ev::TouchEvent| {
            ev.prevent_default();
            // Access the native web_sys::TouchEvent
            let native_ev: &web_sys::TouchEvent = ev.as_ref();
            let touches = native_ev.touches();

            if let Some((canvas, rect)) = input_helpers::canvas_and_rect_from_touch(native_ev) {
                if touches.length() == 1 {
                    // Single finger - track position for drag rotation
                    if let Some(touch) = touches.get(0) {
                        let (x, y) = input_helpers::canvas_coords_from_touch(&touch, &canvas, &rect);
                        set_last_touch_center.set(Some((x, y)));
                        set_touch_start_distance.set(None);
                    }
                } else if touches.length() == 2 {
                    // Two fingers - track distance for pinch zoom
                    if let (Some(t1), Some(t2)) = (touches.get(0), touches.get(1)) {
                        let distance = input_helpers::touch_distance(&t1, &t2);
                        set_touch_start_distance.set(Some(distance));
                        set_last_touch_center.set(Some(input_helpers::touch_center(&t1, &t2)));
                    }
                }
            }

            // Dismiss interaction hint on touch
            if show_interaction_hint.get_untracked() {
                set_show_interaction_hint.set(false);
            }
        }
    };

    let handle_touch_move = {
        move |ev: leptos::ev::TouchEvent| {
            ev.prevent_default();
            let native_ev: &web_sys::TouchEvent = ev.as_ref();
            let touches = native_ev.touches();

            if let Some((canvas, rect)) = input_helpers::canvas_and_rect_from_touch(native_ev) {
                if touches.length() == 1 {
                    // Single finger drag - rotate view
                    if let Some(touch) = touches.get(0) {
                        let (x, y) = input_helpers::canvas_coords_from_touch(&touch, &canvas, &rect);

                        if let Some((last_x, last_y)) = last_touch_center.get() {
                            let delta = (x - last_x, y - last_y);
                            renderer.update_value(|cell| {
                                if let Some(r) = cell.borrow_mut().as_mut() {
                                    r.handle_event(ViewerEvent::MouseMove {
                                        position: MousePosition { x, y },
                                        delta,
                                    });
                                }
                            });
                        }
                        set_last_touch_center.set(Some((x, y)));
                    }
                } else if touches.length() == 2 {
                    // Two finger pinch - zoom
                    if let (Some(t1), Some(t2)) = (touches.get(0), touches.get(1)) {
                        let new_distance = input_helpers::touch_distance(&t1, &t2);

                        if let Some(start_distance) = touch_start_distance.get() {
                            let delta_y = input_helpers::pinch_zoom_delta(start_distance, new_distance);
                            renderer.update_value(|cell| {
                                if let Some(r) = cell.borrow_mut().as_mut() {
                                    r.handle_event(ViewerEvent::Wheel { delta_y });
                                }
                            });
                            set_touch_start_distance.set(Some(new_distance));
                        }
                    }
                }
            }
        }
    };

    let handle_touch_end = {
        move |_ev: leptos::ev::TouchEvent| {
            // Reset touch tracking state
            set_touch_start_distance.set(None);
            set_last_touch_center.set(None);
        }
    };

    // Sharing: encode current view state into a URL query string.
    let on_copy_view_link = {
        let hemisphere = hemisphere.clone();
        let statistic = statistic.clone();
        let volume_idx = volume_idx.clone();
        let threshold = threshold.clone();
        let layout = layout.clone();
        let region_selection_enabled = region_selection_enabled.clone();
        move |_| {
            let state = url_state::ShareableState {
                hemisphere: hemisphere.get_untracked(),
                statistic: statistic.get_untracked(),
                volume_idx: volume_idx.get_untracked(),
                layout: layout.get_untracked(),
                threshold: threshold.get_untracked(),
                region_selection_enabled: region_selection_enabled.get_untracked(),
            };
            let qs = url_state::build_share_query_string(&state);

            if let Some(window) = web_sys::window() {
                if let Ok(loc) = window.location().href() {
                    // Strip existing query / fragment
                    let base = loc
                        .split(&['?', '#'][..])
                        .next()
                        .unwrap_or(&loc)
                        .to_string();
                    let full = format!("{base}?{qs}");
                    set_share_link.set(Some(full));
                }
            }
        }
    };

    // Screenshot: open the current canvas as a data URL in a new tab.
    let on_export_screenshot = {
        let canvas_ref = canvas_ref.clone();
        move |_| {
            if let Some(canvas) = canvas_ref.get() {
                if let Ok(data_url) = canvas.to_data_url() {
                    if let Some(window) = web_sys::window() {
                        let _ = window.open_with_url(&data_url);
                    }
                }
            }
        }
    };

    // Create callback for view preset changes
    let on_view_preset = {
        Callback::new(move |preset: BrainViewPreset| {
            set_current_view.set(Some(preset));
            renderer.update_value(|cell| {
                if let Some(r) = cell.borrow_mut().as_mut() {
                    r.set_view_preset(preset);
                }
            });
        })
    };

    // On-canvas control handlers
    let handle_zoom_in = {
        move |_: leptos::ev::MouseEvent| {
            renderer.update_value(|cell| {
                if let Some(r) = cell.borrow_mut().as_mut() {
                    // Negative delta_y = zoom in (closer)
                    r.handle_event(ViewerEvent::Wheel { delta_y: -2.0 });
                }
            });
        }
    };

    let handle_zoom_out = {
        move |_: leptos::ev::MouseEvent| {
            renderer.update_value(|cell| {
                if let Some(r) = cell.borrow_mut().as_mut() {
                    // Positive delta_y = zoom out (farther)
                    r.handle_event(ViewerEvent::Wheel { delta_y: 2.0 });
                }
            });
        }
    };

    let handle_reset_view = {
        let on_view_preset = on_view_preset.clone();
        move |_: leptos::ev::MouseEvent| {
            // Reset to lateral left as the default view
            on_view_preset.run(BrainViewPreset::LateralLeft);
        }
    };

    let handle_fullscreen_toggle = {
        move |_: leptos::ev::MouseEvent| {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(container) = canvas_container_ref.get() {
                    let document = web_sys::window()
                        .and_then(|w| w.document());

                    if let Some(doc) = document {
                        let is_currently_fullscreen = doc.fullscreen_element().is_some();

                        if is_currently_fullscreen {
                            let _ = doc.exit_fullscreen();
                            set_is_fullscreen.set(false);
                        } else {
                            let element: &web_sys::Element = container.as_ref();
                            let _ = element.request_fullscreen();
                            set_is_fullscreen.set(true);
                        }
                    }
                }
            }
        }
    };

    // Focus mode toggle - custom overlay (different from browser fullscreen)
    let handle_focus_mode_toggle = {
        move |_: leptos::ev::MouseEvent| {
            let current = is_focus_mode.get_untracked();
            set_is_focus_mode.set(!current);

            // Prevent body scroll when focus mode is active
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                    if let Some(body) = document.body() {
                        if !current {
                            // Entering focus mode - prevent scroll
                            let _ = body.style().set_property("overflow", "hidden");
                        } else {
                            // Exiting focus mode - restore scroll
                            let _ = body.style().remove_property("overflow");
                        }
                    }
                }
            }
        }
    };

    // Hide interaction hint on first mouse interaction
    let handle_canvas_interaction = {
        move || {
            if show_interaction_hint.get_untracked() {
                set_show_interaction_hint.set(false);
            }
        }
    };

    let handle_key_down = {
        let on_view_preset = on_view_preset.clone();
        move |ev: leptos::ev::KeyboardEvent| {
            let key = ev.key();
            let modifiers = ModifierKeys::from_keyboard_event(&ev);

            // Prevent default for navigation keys and shortcuts
            if [
                "ArrowUp",
                "ArrowDown",
                "ArrowLeft",
                "ArrowRight",
                "+",
                "-",
                "=",
                "R",
                "r",
                "f",
                "F",
                "Escape",
                "1",
                "2",
                "3",
                "4",
                "5",
                "6",
                "7",
                "8",
            ]
            .contains(&key.as_str())
            {
                ev.prevent_default();
            }
            // Prevent default for undo/redo shortcuts
            if (key == "z" || key == "Z" || key == "y" || key == "Y")
                && (modifiers.ctrl || modifiers.meta)
            {
                ev.prevent_default();
            }

            // Escape: exit focus mode, clear selection, or exit fullscreen
            if key == "Escape" {
                // Priority: focus mode > selection > fullscreen
                if is_focus_mode.get_untracked() {
                    set_is_focus_mode.set(false);
                    #[cfg(target_arch = "wasm32")]
                    {
                        // Restore body scroll
                        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                            if let Some(body) = document.body() {
                                let _ = body.style().remove_property("overflow");
                            }
                        }
                    }
                } else if selected_vertex.get_untracked().is_some() {
                    set_selected_vertex.set(None);
                } else {
                    #[cfg(target_arch = "wasm32")]
                    {
                        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                            if doc.fullscreen_element().is_some() {
                                let _ = doc.exit_fullscreen();
                                set_is_fullscreen.set(false);
                            }
                        }
                    }
                }
            }

            // Fullscreen toggle (F key)
            if key == "f" || key == "F" {
                #[cfg(target_arch = "wasm32")]
                {
                    if let Some(container) = canvas_container_ref.get() {
                        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                            let is_currently_fullscreen = doc.fullscreen_element().is_some();
                            if is_currently_fullscreen {
                                let _ = doc.exit_fullscreen();
                                set_is_fullscreen.set(false);
                            } else {
                                let element: &web_sys::Element = container.as_ref();
                                let _ = element.request_fullscreen();
                                set_is_fullscreen.set(true);
                            }
                        }
                    }
                }
                return;
            }

            // Camera preset shortcuts (1-8)
            let preset = match key.as_str() {
                "1" => Some(BrainViewPreset::LateralLeft),
                "2" => Some(BrainViewPreset::LateralRight),
                "3" => Some(BrainViewPreset::MedialLeft),
                "4" => Some(BrainViewPreset::MedialRight),
                "5" => Some(BrainViewPreset::Dorsal),
                "6" => Some(BrainViewPreset::Ventral),
                "7" => Some(BrainViewPreset::Anterior),
                "8" => Some(BrainViewPreset::Posterior),
                _ => None,
            };

            if let Some(p) = preset {
                on_view_preset.run(p);
                return;
            }

            renderer.update_value(|cell| {
                if let Some(r) = cell.borrow_mut().as_mut() {
                    r.handle_event(ViewerEvent::KeyDown { key, modifiers });
                }
            });
        }
    };

    view! {
        <div
            class=move || {
                let base = "brain-viewer-container focus:outline-none focus:ring-2 focus:ring-[var(--color-focus-ring)] p-2 sm:p-3 md:p-3 lg:pl-0 lg:pr-3";
                if is_focus_mode.get() {
                    format!("{} brain-viewer-focus-active", base)
                } else {
                    base.to_string()
                }
            }
            style:background-color=move || if high_contrast.get() { "black" } else { "var(--color-neutral-900)" }
            tabindex="0"
            aria-label="Interactive 3D brain surface visualization. Use arrow keys to rotate, plus/minus to zoom, R to reset. On touch devices: drag to rotate, pinch to zoom."
            role="application"
            on:keydown=handle_key_down
        >
            // Focus mode overlay
            <Show when=move || is_focus_mode.get()>
                <FocusModeHeader
                    hemisphere=hemisphere.into()
                    set_hemisphere=set_hemisphere
                    threshold=threshold.into()
                    set_threshold=set_threshold
                    stat_metadata=stat_metadata
                    stat_range=stat_range
                    colormap=colormap
                    symmetric=symmetric
                    on_exit=Callback::new(move |_| {
                        set_is_focus_mode.set(false);
                        // Restore body scroll when exiting focus mode
                        #[cfg(target_arch = "wasm32")]
                        {
                            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                                if let Some(body) = document.body() {
                                    let _ = body.style().remove_property("overflow");
                                }
                            }
                        }
                    })
                />
            </Show>

            // Mobile-first responsive layout (2-column on desktop):
            // Single-column layout: Canvas + legend + Analysis Tools
            <div class=move || {
                let base = "brain-viewer-layout flex flex-col gap-3";
                if is_focus_mode.get() {
                    format!("{} brain-viewer-layout-focus", base)
                } else {
                    base.to_string()
                }
            }>
                // Canvas area: wrapper for canvas + legend strip
                <div class="brain-viewer-canvas-area">
                // Canvas container
                <div
                    node_ref=canvas_container_ref
                    class=move || {
                        let base = "brain-viewer-canvas-wrapper relative";
                        if is_fullscreen.get() {
                            format!("{} fullscreen-canvas-container bg-[var(--color-neutral-900)]", base)
                        } else {
                            base.to_string()
                        }
                    }
                >
                    <canvas
                        node_ref=canvas_ref
                        class=move || {
                            let base = "w-full border border-[var(--color-border-default)] rounded-[var(--radius-panel)] shadow-[var(--shadow-md)] bg-[var(--color-neutral-900)] touch-none";
                            if is_fullscreen.get() {
                                format!("{} h-full", base)
                            } else {
                                // Responsive canvas height: smaller on mobile, larger on desktop
                                format!("{} h-64 sm:h-80 md:h-96 lg:h-[28rem]", base)
                            }
                        }
                        width="800"
                        height="600"
                        on:click=handle_canvas_click
                        on:mousedown={
                            let handle_canvas_interaction = handle_canvas_interaction.clone();
                            move |ev| {
                                handle_canvas_interaction();
                                handle_mouse_down(ev);
                            }
                        }
                        on:mouseup=handle_mouse_up
                        on:mousemove=handle_mouse_move
                        on:wheel=handle_wheel
                        on:touchstart=handle_touch_start
                        on:touchmove=handle_touch_move
                        on:touchend=handle_touch_end
                        on:touchcancel=handle_touch_end
                        aria-label="3D brain surface visualization. Touch: drag to rotate, pinch to zoom."
                        role="img"
                    />
                    // Loading overlay with progress indicator
                    <Show when=move || is_loading.get()>
                        <LoadingOverlay
                            progress=loading_progress.into()
                            stage=loading_stage.into()
                        />
                    </Show>

                    // Error overlay with retry option
                    {move || if let Some(err) = error.get() {
                        let on_retry = Callback::new(move |_| {
                            set_error.set(None);
                            set_can_retry.set(false);
                            set_initialized.set(false);
                        });
                        view! {
                            <ErrorOverlay
                                message=err
                                can_retry=can_retry.get()
                                on_retry=on_retry
                            />
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }}
                    // Live region for selection/ROI updates
                    <div class="sr-only" role="status" aria-live="polite">
                        {move || aria_status.get()}
                    </div>

                    // Screen reader summary of current visualization state
                    <div class="sr-only" aria-label="Current visualization summary">
                        {move || {
                            let hemi_name = match hemisphere.get() {
                                Hemisphere::Left => "Left hemisphere",
                                Hemisphere::Right => "Right hemisphere",
                            };
                            let stat_name = match statistic.get() {
                                Statistic::TStat => "T-statistic",
                                Statistic::Beta => "Beta coefficient",
                                Statistic::LogP => "Log p-value",
                                Statistic::Sigma2 => "Residual variance",
                                Statistic::Chi2 => "Chi-squared",
                                Statistic::Chi2lp => "Chi-squared log p-value",
                            };
                            let thr_text = threshold.get()
                                .map(|t| format!(" Threshold: {:.2}", t))
                                .unwrap_or_default();
                            let vol_text = stat_metadata.get()
                                .and_then(|m| m.volumes.iter().find(|v| v.index == volume_idx.get()).map(|v| v.label.clone()))
                                .unwrap_or_else(|| format!("Contrast {}", volume_idx.get()));

                            format!(
                                "Showing {} for {} on {}. {}.",
                                stat_name, vol_text, hemi_name, thr_text
                            )
                        }}
                    </div>

                    // Interaction hint overlay (shown on first load)
                    {move || if show_interaction_hint.get() && !is_loading.get() {
                        view! {
                            // Mouse hint (hidden on touch devices)
                            <div class="hidden md:block absolute top-2 sm:top-4 left-1/2 -translate-x-1/2 px-2 sm:px-3 py-1 sm:py-1.5 bg-[var(--color-neutral-800)]/90 text-[var(--color-text-secondary)] text-xs rounded-full pointer-events-none animate-pulse">
                                "Drag to rotate • Scroll to zoom"
                            </div>
                            // Touch hint (shown on touch devices via CSS @media (pointer: coarse))
                            <div class="brain-viewer-touch-hint md:hidden absolute top-2 left-1/2 -translate-x-1/2 px-2 py-1 bg-[var(--color-neutral-800)]/90 text-[var(--color-text-secondary)] text-xs rounded-full pointer-events-none animate-pulse">
                                "Drag to rotate • Pinch to zoom"
                            </div>
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }}

                    // Consolidated info overlay (top-right corner)
                    // Contains: stat name/range, threshold slider, and selected vertex info
                    <Show when=move || !is_loading.get()>
                        <CanvasInfoOverlay
                            stat_metadata=stat_metadata.into()
                            stat_range=stat_range.into()
                            symmetric=symmetric.into()
                            threshold=threshold.into()
                            set_threshold=set_threshold
                            selected_vertex=selected_vertex.into()
                            selected_region=selected_region.into()
                            set_selected_vertex=set_selected_vertex
                        />
                    </Show>

                    // On-canvas control buttons (responsive sizing via CSS)
                    <div class="brain-viewer-canvas-controls absolute bottom-2 right-2 sm:bottom-4 sm:right-4 flex flex-col gap-0.5 sm:gap-1 bg-[var(--color-bg-surface)]/90 backdrop-blur-sm rounded-[var(--radius-md)] border border-[var(--color-border-default)] p-0.5 sm:p-1 shadow-lg">
                        <button
                            class="w-8 h-8 flex items-center justify-center text-[var(--color-text-primary)] hover:bg-[var(--color-bg-subtle)] rounded-[var(--radius-md)] transition-colors focus:outline-none focus:ring-2 focus:ring-[var(--color-focus-ring)] focus:ring-offset-1"
                            on:click=handle_zoom_in
                            title="Zoom in (+)"
                            aria-label="Zoom in"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <circle cx="11" cy="11" r="8"></circle>
                                <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                                <line x1="11" y1="8" x2="11" y2="14"></line>
                                <line x1="8" y1="11" x2="14" y2="11"></line>
                            </svg>
                        </button>
                        <button
                            class="w-8 h-8 flex items-center justify-center text-[var(--color-text-primary)] hover:bg-[var(--color-bg-subtle)] rounded-[var(--radius-md)] transition-colors focus:outline-none focus:ring-2 focus:ring-[var(--color-focus-ring)] focus:ring-offset-1"
                            on:click=handle_zoom_out
                            title="Zoom out (-)"
                            aria-label="Zoom out"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <circle cx="11" cy="11" r="8"></circle>
                                <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                                <line x1="8" y1="11" x2="14" y2="11"></line>
                            </svg>
                        </button>
                        <div class="w-6 h-px bg-[var(--color-border-default)] mx-auto"></div>
                        <button
                            class="w-8 h-8 flex items-center justify-center text-[var(--color-text-primary)] hover:bg-[var(--color-bg-subtle)] rounded-[var(--radius-md)] transition-colors focus:outline-none focus:ring-2 focus:ring-[var(--color-focus-ring)] focus:ring-offset-1"
                            on:click=handle_reset_view
                            title="Reset view (R)"
                            aria-label="Reset view"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"></path>
                                <path d="M3 3v5h5"></path>
                            </svg>
                        </button>
                        <div class="w-6 h-px bg-[var(--color-border-default)] mx-auto"></div>
                        // Focus mode (expand) button
                        <button
                            class=move || {
                                let base = "w-8 h-8 flex items-center justify-center hover:bg-[var(--color-bg-subtle)] rounded-[var(--radius-md)] transition-colors focus:outline-none focus:ring-2 focus:ring-[var(--color-focus-ring)] focus:ring-offset-1";
                                if is_focus_mode.get() {
                                    format!("{} text-[var(--color-accent-500)]", base)
                                } else {
                                    format!("{} text-[var(--color-text-primary)]", base)
                                }
                            }
                            on:click=handle_focus_mode_toggle
                            title="Focus mode (ESC to exit)"
                            aria-label=move || if is_focus_mode.get() { "Exit focus mode" } else { "Enter focus mode" }
                        >
                            {move || if is_focus_mode.get() {
                                // X icon when in focus mode
                                view! {
                                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <line x1="18" y1="6" x2="6" y2="18"></line>
                                        <line x1="6" y1="6" x2="18" y2="18"></line>
                                    </svg>
                                }.into_any()
                            } else {
                                // Expand icon (maximize-2 from lucide)
                                view! {
                                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <polyline points="15 3 21 3 21 9"></polyline>
                                        <polyline points="9 21 3 21 3 15"></polyline>
                                        <line x1="21" y1="3" x2="14" y2="10"></line>
                                        <line x1="3" y1="21" x2="10" y2="14"></line>
                                    </svg>
                                }.into_any()
                            }}
                        </button>
                        <button
                            class=move || {
                                let base = "w-8 h-8 flex items-center justify-center hover:bg-[var(--color-bg-subtle)] rounded-[var(--radius-md)] transition-colors focus:outline-none focus:ring-2 focus:ring-[var(--color-focus-ring)] focus:ring-offset-1";
                                if is_fullscreen.get() {
                                    format!("{} text-[var(--color-accent-500)]", base)
                                } else {
                                    format!("{} text-[var(--color-text-primary)]", base)
                                }
                            }
                            on:click=handle_fullscreen_toggle
                            title="Toggle fullscreen (F)"
                            aria-label=move || if is_fullscreen.get() { "Exit fullscreen" } else { "Enter fullscreen" }
                        >
                            {move || if is_fullscreen.get() {
                                view! {
                                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <path d="M8 3v3a2 2 0 0 1-2 2H3"></path>
                                        <path d="M21 8h-3a2 2 0 0 1-2-2V3"></path>
                                        <path d="M3 16h3a2 2 0 0 1 2 2v3"></path>
                                        <path d="M16 21v-3a2 2 0 0 1 2-2h3"></path>
                                    </svg>
                                }.into_any()
                            } else {
                                view! {
                                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <path d="M8 3H5a2 2 0 0 0-2 2v3"></path>
                                        <path d="M21 8V5a2 2 0 0 0-2-2h-3"></path>
                                        <path d="M3 16v3a2 2 0 0 0 2 2h3"></path>
                                        <path d="M16 21h3a2 2 0 0 0 2-2v-3"></path>
                                    </svg>
                                }.into_any()
                            }}
                        </button>
                    </div>
                </div>
                // Action toolbar below canvas (share/export buttons)
                <div class="brain-viewer-actions-bar">
                    <div class="brain-viewer-actions-compact" role="toolbar" aria-label="Viewer actions">
                        <button
                            class="action-icon-btn"
                            on:click=on_export_screenshot
                            title="Export screenshot"
                            aria-label="Export current view as screenshot"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                                <polyline points="7 10 12 15 17 10"></polyline>
                                <line x1="12" y1="15" x2="12" y2="3"></line>
                            </svg>
                        </button>
                        <button
                            class="action-icon-btn"
                            on:click=on_copy_view_link
                            title="Copy view link"
                            aria-label="Copy shareable link to current view"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                                <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
                            </svg>
                        </button>
                    </div>
                    // Share link display (if active)
                    {move || if let Some(link) = share_link.get() {
                        view! {
                            <div class="share-link-display">
                                <input
                                    type="text"
                                    class="share-link-input"
                                    readonly=true
                                    value=link
                                    aria-label="Shareable view link"
                                />
                            </div>
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }}
                </div>
                // Analysis Tools Panel - below legend strip (contains all controls)
                <AnalysisToolsPanel
                    // ControlsCard props
                    hemisphere=hemisphere
                    set_hemisphere=set_hemisphere
                    volume_idx=volume_idx
                    set_volume_idx=set_volume_idx
                    n_volumes=n_volumes.into()
                    stat_metadata=stat_metadata
                    colormap=colormap
                    set_colormap=set_colormap
                    symmetric=symmetric
                    set_symmetric=set_symmetric
                    analysis=analysis
                    set_analysis=set_analysis
                    statistic=statistic
                    set_statistic=set_statistic
                    // CameraPresets props
                    on_view_preset=on_view_preset
                    current_view=current_view
                    // Shared
                    disabled=is_loading
                    // VertexSummaryTable props
                    statistics=current_stats.into()
                />
                </div>
            </div>
        </div>
    }
}
