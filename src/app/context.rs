use std::num::NonZeroU32;

use egui_winit::winit;
use glutin::context::{ContextApi, GlProfile, NotCurrentGlContext, Version};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::prelude::GlSurface;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::HasWindowHandle;

use crate::app::{WINDOW_HEIGHT, WINDOW_WIDTH};

pub(super) struct GlutinWindowContext {
    window: winit::window::Window,
    gl_context: glutin::context::PossiblyCurrentContext,
    gl_display: glutin::display::Display,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl GlutinWindowContext {
    unsafe fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        log::warn!("Create config");
        let winit_window_builder = winit::window::WindowAttributes::default()
            .with_resizable(false)
            .with_inner_size(winit::dpi::LogicalSize {
                width: WINDOW_WIDTH as f32,
                height: WINDOW_HEIGHT as f32,
            })
            .with_title("Final project")
            .with_visible(false);

        let config_template_builder = glutin::config::ConfigTemplateBuilder::new()
            .prefer_hardware_accelerated(None)
            .with_depth_size(24)
            .with_stencil_size(0)
            .with_multisampling(4)
            .with_transparency(false);

        log::debug!("trying to get gl_config");
        let (mut window, gl_config) = glutin_winit::DisplayBuilder::new()
            .with_preference(glutin_winit::ApiPreference::FallbackEgl)
            .with_window_attributes(Some(winit_window_builder.clone()))
            .build(
                event_loop,
                config_template_builder,
                |mut config_iterator| {
                    config_iterator.next().expect(
                        "failed to find a matching configuration for creating glutin config",
                    )
                },
            )
            .expect("failed to create gl_config");
        let gl_display = gl_config.display();
        log::debug!("found gl_config: {:?}", &gl_config);

        let raw_window_handle = window.as_ref().map(|w| {
            w.window_handle()
                .expect("failed to get window handle")
                .as_raw()
        });
        log::debug!("raw window handle: {:?}", raw_window_handle);
        let context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_profile(GlProfile::Core)
            .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 6))))
            .build(raw_window_handle);

        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .expect("failed to create gl_context")
        };

        let window = window.take().unwrap_or_else(|| {
            log::debug!("window doesn't exist yet. creating one now with finalize_window");
            glutin_winit::finalize_window(event_loop, winit_window_builder.clone(), &gl_config)
                .expect("failed to finalize glutin window")
        });

        let (width, height): (u32, u32) = window.inner_size().into();
        let width = NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN);
        let height = NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN);
        let surface_attributes =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(
                    window
                        .window_handle()
                        .expect("failed to get window handle")
                        .as_raw(),
                    width,
                    height,
                );
        log::debug!(
            "creating surface with attributes: {:?}",
            &surface_attributes
        );
        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attributes)
                .unwrap()
        };
        log::debug!("surface created successfully: {gl_surface:?}.making context current");
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        gl_surface
            .set_swap_interval(
                &gl_context,
                glutin::surface::SwapInterval::Wait(NonZeroU32::MIN),
            )
            .unwrap();

        Self {
            window,
            gl_context,
            gl_display,
            gl_surface,
        }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        use glutin::surface::GlSurface;
        self.gl_surface.resize(
            &self.gl_context,
            physical_size.width.try_into().unwrap(),
            physical_size.height.try_into().unwrap(),
        );
    }

    pub fn swap_buffers(&self) -> glutin::error::Result<()> {
        use glutin::surface::GlSurface;
        self.gl_surface.swap_buffers(&self.gl_context)
    }

    pub fn get_proc_address(&self, addr: &std::ffi::CStr) -> *const std::ffi::c_void {
        use glutin::display::GlDisplay;
        self.gl_display.get_proc_address(addr)
    }
}

pub(super) fn create_display(event_loop: &ActiveEventLoop) -> (GlutinWindowContext, glow::Context) {
    let glutin_window_context = unsafe { GlutinWindowContext::new(event_loop) };
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s)
                .expect("failed to construct C string from string for gl proc address");

            glutin_window_context.get_proc_address(&s)
        })
    };

    (glutin_window_context, gl)
}
