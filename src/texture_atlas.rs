use std::num::NonZeroU32;

use glam::f32::Vec2;

use crate::shape::{IRect, Rect};

#[derive(Copy, Clone)]
pub struct TextureHandle {
    atlas: u32,
    index: u32,
}

impl TextureHandle {
    pub(crate) fn bind_point(&self) -> NonZeroU32 {
        bind_point_for_atlas(self.atlas)
    }
}

pub struct TextureAtlas {
    pages: Vec<TexturePage>,
}

impl TextureAtlas {
    pub fn new() -> TextureAtlas {
        TextureAtlas { pages: Vec::new() }
    }

    pub fn upload_image(
        &mut self,
        ctx: &golem::Context,
        image_data: &[u8],
        width: u32,
        height: u32,
    ) -> TextureHandle {
        let mut texture = None;
        for (page_index, page) in self.pages.iter_mut().enumerate() {
            let upload_result = page.upload_texture(image_data, width, height);
            if let Ok(index) = upload_result {
                texture = Some(TextureHandle {
                    atlas: page_index as u32,
                    index,
                });
                break;
            }
        }
        match texture {
            Some(texture) => texture,
            None => {
                let atlas = self.pages.len() as u32;
                let mut page = TexturePage::new(ctx);
                let index = page
                    .upload_texture(image_data, width, height)
                    .expect("uploading texture");
                page.backing_texture.set_active(bind_point_for_atlas(atlas));
                self.pages.push(page);
                TextureHandle { atlas, index }
            }
        }
    }

    pub fn uv(&self, texture: TextureHandle, uv: Rect) -> Rect {
        let region = &self.pages[texture.atlas as usize].texture_uvs[texture.index as usize];
        let texture_point = Vec2::new(region.x as f32, region.y as f32) / ATLAS_SIZE_VEC2;
        let uv_position = texture_point + uv.position();
        let uv_size = uv.size() / Vec2::new(region.width as f32, region.height as f32);
        Rect {
            x: uv_position.x,
            y: uv_position.y,
            width: uv_size.x,
            height: uv_size.y,
        }
    }
}

fn bind_point_for_atlas(atlas: u32) -> NonZeroU32 {
    // SAFETY: given an input of 0, 1 will be passed to the function.
    unsafe { NonZeroU32::new_unchecked(atlas + 1) }
}

struct TexturePage {
    backing_texture: golem::Texture,
    cursor_x: u32,
    cursor_y: u32,
    line_height: u32,
    texture_uvs: Vec<IRect>,
}

const ATLAS_SIZE: u32 = 2048;
const ATLAS_SIZE_VEC2: Vec2 = Vec2::new(ATLAS_SIZE as f32, ATLAS_SIZE as f32);

#[derive(Debug)]
enum TextureAllocationError {
    CantFit,
}

impl TexturePage {
    fn new(ctx: &golem::Context) -> TexturePage {
        let mut backing_texture = golem::Texture::new(ctx).expect("failed to allocate a texture");
        backing_texture.set_image(None, ATLAS_SIZE, ATLAS_SIZE, golem::ColorFormat::RGBA);
        TexturePage {
            backing_texture,
            cursor_x: 0,
            cursor_y: 0,
            line_height: 0,
            texture_uvs: Vec::new(),
        }
    }

    fn upload_texture(
        &mut self,
        image_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<u32, TextureAllocationError> {
        if self.cursor_x + width >= ATLAS_SIZE {
            if self.cursor_y + self.line_height + height >= ATLAS_SIZE {
                return Err(TextureAllocationError::CantFit);
            }
            self.cursor_y += self.line_height;
            self.cursor_x = 0;
            self.line_height = 0;
        }

        self.backing_texture.set_subimage(
            image_data,
            self.cursor_x,
            self.cursor_y,
            width,
            height,
            golem::ColorFormat::RGBA,
        );
        let index = self.texture_uvs.len() as u32;
        self.texture_uvs.push(IRect {
            x: self.cursor_x as i32,
            y: self.cursor_y as i32,
            width: width as i32,
            height: height as i32,
        });
        self.cursor_x += width;
        self.line_height = self.line_height.max(height);

        Ok(index)
    }
}
