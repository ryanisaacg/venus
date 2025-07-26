use fontdue::Metrics;
use rustc_hash::FxHashMap as HashMap;

use crate::{Error, Rect, Texture, graphics::Graphics};

pub struct Font {
    font: fontdue::Font,
    characters: HashMap<(char, u32), (Texture, Metrics)>,
}

impl Font {
    pub fn from_bytes(bytes: &[u8]) -> Result<Font, Error> {
        let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
            .map_err(Error::FontError)?;
        Ok(Font {
            font,
            characters: HashMap::default(),
        })
    }

    pub fn metrics(&self, ch: char, size: u32) -> Metrics {
        match self.characters.get(&(ch, size)) {
            Some((_texture, size)) => size.clone(),
            _ => self.font.metrics(ch, size as f32),
        }
    }

    pub fn rasterize(
        &mut self,
        ch: char,
        size: u32,
        graphics: &mut Graphics,
    ) -> &(Texture, Metrics) {
        self.characters.entry((ch, size)).or_insert_with(|| {
            let (metrics, buffer) = self.font.rasterize(ch, size as f32);
            let buffer: Vec<_> = buffer
                .into_iter()
                .map(|coverage| [255, 255, 255, coverage])
                .flatten()
                .collect();
            let width = metrics.width as u32;
            let height = metrics.height as u32;
            let handle = graphics.new_texture_from_bytes(&buffer, width, height);
            let texture = Texture {
                handle,
                uv: Rect {
                    x: 0.,
                    y: 0.,
                    width: 1.,
                    height: 1.,
                },
                width,
                height,
            };
            (texture, metrics)
        })
    }

    pub fn text_width(&self, text: &str, size: u32) -> f32 {
        let mut width = 0.0;

        let prev_ch = None;
        for ch in text.chars() {
            if let Some(prev_ch) = prev_ch {
                if let Some(kern) = self.font.horizontal_kern(prev_ch, ch, size as f32) {
                    width += kern;
                }
            }
            let metrics = self.metrics(ch, size);
            width += metrics.advance_width;
        }

        width
    }

    pub fn line_height(&self, size: u32) -> f32 {
        let line_metrics = self.font.horizontal_line_metrics(size as f32);
        line_metrics
            .map(|metrics| metrics.new_line_size)
            .unwrap_or(size as f32)
    }
}

#[derive(Default)]
pub struct TextRenderer {
    word_buffer: String,
    character_buffer: Vec<(Texture, char, f32, f32)>,
}

impl TextRenderer {
    pub fn layout_text(
        &mut self,
        gfx: &mut Graphics,
        font: &mut Font,
        x: f32,
        y: f32,
        text: &str,
        size: u32,
        max_line_length: f32,
    ) {
        let mut cursor_x = x;
        let mut topline = y;

        let line_height = font.line_height(size);

        let mut prev_ch = None;
        for ch in text.chars() {
            // TODO: also break on other characters like '-'
            if !ch.is_whitespace() {
                self.word_buffer.push(ch);
                continue;
            }

            // Handle one word at a time
            let word_length = font.text_width(&self.word_buffer, size);

            if cursor_x + word_length > x + max_line_length {
                cursor_x = x;
                topline += line_height;
                prev_ch = None;
            }

            for ch in self.word_buffer.drain(..) {
                Self::push_character(
                    ch,
                    &mut cursor_x,
                    topline,
                    size,
                    &mut prev_ch,
                    gfx,
                    font,
                    &mut self.character_buffer,
                );
            }

            if ch == '\n' {
                cursor_x = x;
                topline += line_height;
                prev_ch = None;
            } else {
                Self::push_character(
                    ch,
                    &mut cursor_x,
                    topline,
                    size,
                    &mut prev_ch,
                    gfx,
                    font,
                    &mut self.character_buffer,
                );
            }
        }

        for ch in self.word_buffer.drain(..) {
            Self::push_character(
                ch,
                &mut cursor_x,
                topline,
                size,
                &mut prev_ch,
                gfx,
                font,
                &mut self.character_buffer,
            );
        }
    }

    pub fn characters(&mut self) -> impl Iterator<Item = (Texture, char, f32, f32)> {
        self.character_buffer.drain(..)
    }

    fn push_character(
        ch: char,
        cursor_x: &mut f32,
        topline: f32,
        size: u32,
        prev_ch: &mut Option<char>,
        gfx: &mut Graphics,
        font: &mut Font,
        character_buffer: &mut Vec<(Texture, char, f32, f32)>,
    ) {
        if let Some(prev_ch) = prev_ch {
            if let Some(kern) = font.font.horizontal_kern(*prev_ch, ch, size as f32) {
                *cursor_x += kern;
            }
        }
        let (texture, metrics) = font.rasterize(ch, size, gfx);
        let y = topline + ((size as f32 - metrics.height as f32) - (metrics.ymin as f32));
        character_buffer.push((texture.clone(), ch, *cursor_x, y));
        *cursor_x += metrics.advance_width;
        *prev_ch = Some(ch);
    }
}
