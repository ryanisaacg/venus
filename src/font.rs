use fontdue::Metrics;
use rustc_hash::FxHashMap as HashMap;

use crate::{Error, Rect, Texture, graphics::Graphics};

pub struct Font {
    pub font: fontdue::Font,
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
}
