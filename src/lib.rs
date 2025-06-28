use std::fmt::Display;

use blinds::{CachedEventStream, Event, Window};
use font::Font;
use fontdue::Metrics;
use rustc_hash::FxHashSet as HashSet;

pub use blinds::Key;
pub use color::Color;
pub use shape::Rect;

use shape::orthographic_projection;
use texture_atlas::TextureHandle;

use graphics::Graphics;

mod color;
mod font;
mod graphics;
mod shape;
mod texture_atlas;

pub struct Venus {
    window: Window,
    event_stream: CachedEventStream,
    gfx: Graphics,
    just_pressed: HashSet<Key>,
    fonts: Vec<Font>,
    text_line_temp_buffer: Vec<(f32, Texture, Metrics)>,
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
                    just_pressed: HashSet::default(),
                    fonts: Vec::new(),
                    text_line_temp_buffer: Vec::new(),
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

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.just_pressed.contains(&key)
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
            uv: Rect {
                x: 0.,
                y: 0.,
                width: 1.,
                height: 1.,
            },
            width,
            height,
        }
    }

    pub async fn load_texture(&mut self, path: &str) -> Result<Texture, Error> {
        let bytes = load_file(path).await?;
        let image = image::load_from_memory(&bytes).map_err(|error| Error::ImageDecodeError {
            path: path.to_string(),
            error: Box::new(error),
        })?;
        Ok(self.new_texture_from_bytes(image.as_bytes(), image.width(), image.height()))
    }

    pub async fn load_font(&mut self, path: &str) -> Result<FontHandle, Error> {
        let bytes = load_file(path).await?;
        let font = Font::from_bytes(&bytes)?;
        let idx = self.fonts.len();
        self.fonts.push(font);
        Ok(FontHandle(idx as u32))
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

    pub fn draw_image(&mut self, texture: &Texture, x: f32, y: f32) {
        draw_image(&mut self.gfx, texture, x, y);
    }

    pub fn draw_text(&mut self, font: FontHandle, x: f32, y: f32, text: &str, size: u32) {
        self.draw_text_wrap(font, x, y, text, size, f32::MAX);
    }

    pub fn draw_text_wrap(
        &mut self,
        font: FontHandle,
        x: f32,
        y: f32,
        text: &str,
        size: u32,
        max_line_length: f32,
    ) {
        let font = &mut self.fonts[font.0 as usize];
        let mut cursor_x = x;
        let mut topline = y;

        let line_metrics = font.font.horizontal_line_metrics(size as f32);
        let line_height = line_metrics
            .map(|metrics| metrics.new_line_size)
            .unwrap_or(size as f32);

        let prev_ch = None;
        for ch in text.chars() {
            if let Some(prev_ch) = prev_ch {
                if let Some(kern) = font.font.horizontal_kern(prev_ch, ch, size as f32) {
                    cursor_x += kern;
                }
            }

            let mut x_position = cursor_x;
            let (texture, metrics) = font.rasterize(ch, size, &mut self.gfx);
            let mut flush = false;

            if ch == '\n' {
                flush = true;
            } else {
                cursor_x += metrics.advance_width;

                if cursor_x >= x + max_line_length {
                    flush = true;
                }
            }

            if flush {
                draw_text_line(
                    &mut self.text_line_temp_buffer,
                    &mut self.gfx,
                    topline,
                    size as f32,
                );
                cursor_x = x + metrics.advance_width;
                topline += line_height;
                x_position = x;
            }

            self.text_line_temp_buffer
                .push((x_position, texture.clone(), metrics.clone()));
        }
        draw_text_line(
            &mut self.text_line_temp_buffer,
            &mut self.gfx,
            topline,
            size as f32,
        );
    }

    pub async fn end_frame(&mut self) {
        self.gfx.flush();
        self.window.present();
        self.just_pressed.clear();
        loop {
            let event = self.event_stream.next_event().await;
            match event {
                None => break,
                Some(Event::KeyboardInput(e)) if e.is_presed() => {
                    self.just_pressed.insert(e.key());
                }
                _ => {}
            }
        }
    }
}

// Required because otherwise draw_text mutably borrows Venus twice
fn draw_image(gfx: &mut Graphics, texture: &Texture, x: f32, y: f32) {
    gfx.push_rect(
        Rect {
            x,
            y,
            width: texture.width as f32,
            height: texture.height as f32,
        },
        Color::WHITE,
        Some((texture.handle, texture.uv.clone())),
    );
}

fn draw_text_line(
    text_line_buffer: &mut Vec<(f32, Texture, Metrics)>,
    gfx: &mut Graphics,
    topline: f32,
    size: f32,
) {
    for (x_position, texture, metrics) in text_line_buffer.drain(..) {
        let y = topline + (size - metrics.height as f32) - (metrics.ymin as f32);
        draw_image(gfx, &texture, x_position, y);
    }
}

#[derive(Clone, Debug)]
pub struct Texture {
    handle: TextureHandle,
    uv: Rect,
    width: u32,
    height: u32,
}

impl Texture {
    pub fn sub_texture(&self, x: u32, y: u32, width: u32, height: u32) -> Texture {
        assert!(
            x + width < self.width && y + height < self.height,
            "sub-texture coordinates must be within the bounds of the texture"
        );
        let uv = Rect {
            x: self.uv.x + (x as f32 / self.width as f32) * self.uv.width,
            y: self.uv.y + (y as f32 / self.height as f32) * self.uv.height,
            width: (width as f32 / self.width as f32) * self.uv.width,
            height: (height as f32 / self.height as f32) * self.uv.height,
        };

        Texture {
            handle: self.handle,
            uv,
            width,
            height,
        }
    }
}

type OpaqueError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub enum Error {
    ImageDecodeError { path: String, error: OpaqueError },
    FileLoadError { path: String, error: std::io::Error },
    FontError(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ImageDecodeError { path, error: _ } => {
                write!(f, "Image decoding error when loading {path}")
            }
            Error::FileLoadError { path, error: _ } => write!(f, "Error loading file: {path}"),
            Error::FontError(error) => write!(f, "Error in font: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ImageDecodeError { path: _, error } => Some(error.as_ref()),
            Error::FileLoadError { path: _, error } => Some(error),
            Error::FontError(_) => None,
        }
    }
}

#[derive(Copy, Clone)]
pub struct FontHandle(u32);

pub async fn load_file(path: &str) -> Result<Vec<u8>, Error> {
    let bytes = platter::load_file(path)
        .await
        .map_err(|error| Error::FileLoadError {
            path: path.to_string(),
            error,
        })?;
    Ok(bytes)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sub_texture() {
        let texture = Texture {
            handle: TextureHandle::mock(),
            uv: Rect::new(0.0, 0.0, 0.5, 0.5),
            width: 192,
            height: 320,
        };
        // Basic width / height correctness
        let sub_texture = texture.sub_texture(0, 0, 32, 32);
        assert_eq!(
            sub_texture.width as f32 / texture.width as f32,
            sub_texture.uv.width / 2.0
        );
        assert_eq!(
            sub_texture.height as f32 / texture.height as f32,
            sub_texture.uv.height / 2.0
        );
    }
}
