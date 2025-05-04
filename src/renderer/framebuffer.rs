use glow::HasContext;

#[derive(Copy, Clone)]
pub(super) struct Framebuffer(glow::Framebuffer);

impl Framebuffer {
    pub(super) fn bind(self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.0));
        }
    }

    pub(super) fn unbind(self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }

    pub(super) fn read_pixel_from_stencil(self, gl: &glow::Context, (x, y): (i32, i32)) -> u8 {
        let mut slice = [0; 1];
        unsafe {
            gl.read_pixels(
                x,
                y,
                1,
                1,
                glow::STENCIL_INDEX,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut slice)),
            );
        }

        slice[0]
    }
}

#[derive(Clone, Copy)]
pub(super) struct FramebufferBuilder {
    width: u32,
    height: u32,
    depth_buffer: bool,
}

impl FramebufferBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            depth_buffer: false,
        }
    }

    pub fn with_depth(mut self, flag: bool) -> Self {
        self.depth_buffer = flag;
        self
    }

    pub fn build(self, gl: &glow::Context) -> Framebuffer {
        unsafe {
            let fbo = gl.create_framebuffer().unwrap();
            let fbo = Framebuffer(fbo);

            fbo.bind(gl);

            {
                let color_buffer = gl.create_texture().unwrap();

                gl.bind_texture(glow::TEXTURE_2D, Some(color_buffer));
                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGB as i32,
                    self.width as i32,
                    self.height as i32,
                    0,
                    glow::RGB,
                    glow::UNSIGNED_INT,
                    glow::PixelUnpackData::Slice(None),
                );

                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MIN_FILTER,
                    glow::LINEAR as i32,
                );
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MAG_FILTER,
                    glow::LINEAR as i32,
                );

                gl.bind_texture(glow::TEXTURE_2D, None);

                gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT0,
                    glow::TEXTURE_2D,
                    Some(color_buffer),
                    0,
                );
            }

            if self.depth_buffer {
                let depth_stencil = gl.create_renderbuffer().unwrap();
                gl.bind_renderbuffer(glow::RENDERBUFFER, Some(depth_stencil));

                gl.renderbuffer_storage(
                    glow::RENDERBUFFER,
                    glow::DEPTH24_STENCIL8,
                    self.width as i32,
                    self.height as i32,
                );

                gl.bind_renderbuffer(glow::RENDERBUFFER, None);

                gl.framebuffer_renderbuffer(
                    glow::FRAMEBUFFER,
                    glow::DEPTH_STENCIL_ATTACHMENT,
                    glow::RENDERBUFFER,
                    Some(depth_stencil),
                );
            }

            assert_eq!(
                gl.check_framebuffer_status(glow::FRAMEBUFFER),
                glow::FRAMEBUFFER_COMPLETE
            );

            fbo.unbind(gl);

            fbo
        }
    }
}
