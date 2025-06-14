use std::fmt::Display;

use blinds::{CachedEventStream, Window};

pub use blinds::Key;
pub use color::Color;
pub use shape::Rect;

use shape::orthographic_projection;
use texture_atlas::TextureHandle;

use graphics::Graphics;

mod color;
mod graphics;
mod shape;
mod texture_atlas;

pub struct Venus {
    window: Window,
    event_stream: CachedEventStream,
    gfx: Graphics,
}

impl Venus {
    pub fn run<T: Future<Output = ()>, F: FnOnce(Venus) -> T + 'static>(f: F) {
        blinds::run(
            blinds::Settings::default(),
            async move |window, event_stream| {
                #[cfg(not(target_arch = "wasm32"))]
                let golem = unsafe {
                    golem::Context::from_loader_function_cstr(|func| window.get_proc_address(func))
                };
                #[cfg(target_arch = "wasm32")]
                let golem = golem::Context::from_webgl2_context(window.webgl2_context());
                let golem = golem.expect("graphics initialization");
                let mut venus = Venus {
                    window,
                    event_stream: CachedEventStream::new(event_stream),
                    gfx: Graphics::new(golem),
                };
                venus
                    .gfx
                    .set_projection_matrix(orthographic_projection(0.0, 0.0, 1024.0, 768.0));
                f(venus).await
            },
        );
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.event_stream.cache().key(key)
    }

    pub fn clear(&self, c: Color) {
        self.gfx.clear(c);
    }

    pub fn new_texture_from_bytes(
        &mut self,
        image_data: &[u8],
        width: u32,
        height: u32,
    ) -> Texture {
        let handle = self.gfx.new_texture_from_bytes(image_data, width, height);
        Texture {
            handle,
            width,
            height,
        }
    }

    pub async fn load_texture(&mut self, path: &str) -> Result<Texture, Error> {
        let bytes = platter::load_file(path)
            .await
            .map_err(|error| Error::FileLoadError {
                path: path.to_string(),
                error,
            })?;
        let image = image::load_from_memory(&bytes).map_err(|error| Error::ImageDecodeError {
            path: path.to_string(),
            error: Box::new(error),
        })?;
        Ok(self.new_texture_from_bytes(image.as_bytes(), image.width(), image.height()))
    }

    pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.gfx.push_rect(
            Rect {
                x,
                y,
                width,
                height,
            },
            color,
            None,
        );
    }

    pub fn draw_image(&mut self, x: f32, y: f32, texture: &Texture) {
        self.gfx.push_rect(
            Rect {
                x,
                y,
                width: texture.width as f32,
                height: texture.height as f32,
            },
            Color::WHITE,
            Some((
                texture.handle,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 1.0,
                    height: 1.0,
                },
            )),
        );
    }

    pub async fn end_frame(&mut self) {
        self.gfx.flush();
        self.window.present();
        loop {
            let event = self.event_stream.next_event().await;
            if event.is_none() {
                break;
            }
        }
    }
}

pub struct Texture {
    handle: TextureHandle,
    width: u32,
    height: u32,
}

#[derive(Debug)]
pub enum Error {
    ImageDecodeError {
        path: String,
        error: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
    FileLoadError {
        path: String,
        error: std::io::Error,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ImageDecodeError { path, error: _ } => {
                write!(f, "Image decoding error when loading {path}")
            }
            Error::FileLoadError { path, error: _ } => write!(f, "Error loading file: {path}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ImageDecodeError { path: _, error } => Some(error.as_ref()),
            Error::FileLoadError { path: _, error } => Some(error),
        }
    }
}
