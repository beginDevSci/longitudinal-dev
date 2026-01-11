use web_sys::HtmlCanvasElement;

use crate::traits::RenderError;

/// Holds the wgpu device, queue, surface, and configuration.
pub struct DeviceContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

impl DeviceContext {
    /// Initialize wgpu using a browser canvas element.
    #[cfg(target_arch = "wasm32")]
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, RenderError> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::JsValue;

        web_sys::console::log_1(&JsValue::from_str("[wgpu] Creating instance..."));

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        web_sys::console::log_1(&JsValue::from_str("[wgpu] Instance created, creating surface..."));

        // Create surface from canvas for WASM target
        let canvas_element: web_sys::HtmlCanvasElement = canvas.unchecked_into();
        let width = canvas_element.width().max(1);
        let height = canvas_element.height().max(1);

        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas_element))
            .map_err(|e| RenderError::Message(format!("failed to create surface: {e:?}")))?;

        web_sys::console::log_1(&JsValue::from_str("[wgpu] Surface created, requesting adapter..."));

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| RenderError::Message("no suitable adapter found".into()))?;

        web_sys::console::log_1(&JsValue::from_str("[wgpu] Adapter acquired!"));

        // Log some basic adapter information for debugging in the browser console.
        {
            use wasm_bindgen::JsValue;

            let info = adapter.get_info();
            let msg = format!(
                "wgpu adapter: name='{}', vendor={}, device={}, backend={:?}",
                info.name, info.vendor, info.device, info.backend
            );
            web_sys::console::log_1(&JsValue::from_str(&msg));
        }

        web_sys::console::log_1(&JsValue::from_str("[wgpu] Requesting device..."));

        // Use conservative limits compatible with current browser implementations.
        // Avoid overriding adapter limits to reduce the risk of requesting
        // unsupported values such as maxInterStageShaderComponents.
        let limits = wgpu::Limits::downlevel_webgl2_defaults();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("wgpu_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: limits,
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .map_err(|e| RenderError::Message(format!("device request failed: {e:?}")))?;

        web_sys::console::log_1(&JsValue::from_str("[wgpu] Device acquired! Configuring surface..."));

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .first()
            .copied()
            .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        web_sys::console::log_1(&JsValue::from_str("[wgpu] DeviceContext initialization complete!"));

        Ok(Self {
            device,
            queue,
            surface,
            config,
        })
    }

    /// Stub for non-WASM targets (will fail at runtime if used)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new(_canvas: HtmlCanvasElement) -> Result<Self, RenderError> {
        Err(RenderError::Message(
            "wgpu renderer is only supported on WASM targets".into(),
        ))
    }
}
