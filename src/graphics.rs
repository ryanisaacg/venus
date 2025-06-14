use std::num::NonZeroU32;

use glam::Mat3;
use golem::{
    Attribute, AttributeType, ElementBuffer, GeometryMode, ShaderDescription, ShaderProgram,
    Uniform, UniformType, UniformValue, VertexBuffer,
};

use crate::{
    Color,
    shape::Rect,
    texture_atlas::{TextureAtlas, TextureHandle},
};

pub struct Graphics {
    ctx: golem::Context,
    vb: VertexBuffer,
    eb: ElementBuffer,
    shader: ShaderProgram,
    vertex_data: Vec<f32>,
    index_data: Vec<u32>,
    vertices: u32,
    atlas: TextureAtlas,
    bound_texture: Option<NonZeroU32>,
}

impl Graphics {
    pub fn new(ctx: golem::Context) -> Graphics {
        use golem::Dimension::*;
        let mut shader = ShaderProgram::new(
            &ctx,
            ShaderDescription {
                vertex_input: &[
                    Attribute::new("vert_color", AttributeType::Vector(D4)),
                    Attribute::new("vert_position", AttributeType::Vector(D2)),
                    Attribute::new("vert_uv", AttributeType::Vector(D2)),
                ],
                fragment_input: &[
                    Attribute::new("frag_color", AttributeType::Vector(D4)),
                    Attribute::new("frag_uv", AttributeType::Vector(D2)),
                ],
                uniforms: &[
                    Uniform::new("image", UniformType::Sampler2D),
                    Uniform::new("projection", UniformType::Matrix(D3)),
                ],
                vertex_shader: r#" void main() {
                vec3 transformed = projection * vec3(vert_position, 1.0);
                gl_Position = vec4(transformed.xy, 0, 1);
                frag_uv = vert_uv;
                frag_color = vert_color;
            }"#,
                fragment_shader: r#" void main() {
                vec4 tex = vec4(1);
                if(frag_uv.x >= 0.0 && frag_uv.y >= 0.0) {
                    tex = texture(image, frag_uv);
                }
                gl_FragColor = tex * frag_color;
            }"#,
            },
        )
        .expect("compiling shaders");
        shader.bind();
        shader
            .set_uniform(
                "projection",
                UniformValue::Matrix3(
                    // identity matrix
                    [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
                ),
            )
            .expect("setting projection matrix");
        let vb = VertexBuffer::new(&ctx).expect("creating vertex buffer");
        let eb = ElementBuffer::new(&ctx).expect("create element buffer");
        shader.bind();
        ctx.set_blend_mode(Some(Default::default()));

        Graphics {
            ctx,
            vb,
            eb,
            shader,
            vertex_data: Vec::new(),
            index_data: Vec::new(),
            vertices: 0,
            atlas: TextureAtlas::new(),
            bound_texture: None,
        }
    }

    pub fn clear(&self, color: Color) {
        self.ctx.set_clear_color(color.r, color.g, color.b, color.a);
        self.ctx.clear();
    }

    pub fn set_projection_matrix(&mut self, matrix: Mat3) {
        self.flush();
        self.shader.bind();
        let mut data = [0.0; 9];
        matrix.write_cols_to_slice(&mut data);
        self.shader
            .set_uniform("projection", UniformValue::Matrix3(data))
            .expect("set projection matrix");
    }

    pub fn new_texture_from_bytes(
        &mut self,
        image_data: &[u8],
        width: u32,
        height: u32,
    ) -> TextureHandle {
        self.atlas
            .upload_image(&self.ctx, image_data, width, height)
    }

    pub fn push_rect(
        &mut self,
        region: Rect,
        color: Color,
        texture: Option<(TextureHandle, Rect)>,
    ) {
        let uv = if let Some((texture, uv)) = texture {
            let bind_point = texture.bind_point();
            if let Some(currently_bound) = self.bound_texture {
                if bind_point != currently_bound {
                    self.flush();
                }
            }
            self.shader
                .set_uniform("image", UniformValue::Int(bind_point.get() as i32))
                .expect("change active image");
            self.bound_texture = Some(bind_point);
            self.atlas.uv(texture, uv)
        } else {
            Rect {
                x: -1.0,
                y: -1.0,
                width: 0.0,
                height: 0.0,
            }
        };
        let index = self.vertices;
        self.push_vertex(region.x, region.y, color, uv.x, uv.y);
        self.push_vertex(
            region.x + region.width,
            region.y,
            color,
            uv.x + uv.width,
            uv.y,
        );
        self.push_vertex(
            region.x + region.width,
            region.y + region.height,
            color,
            uv.x + uv.width,
            uv.y + uv.height,
        );
        self.push_vertex(
            region.x,
            region.y + region.height,
            color,
            uv.x,
            uv.y + uv.height,
        );
        self.index_data.extend_from_slice(&[
            index,
            index + 1,
            index + 2,
            index,
            index + 2,
            index + 3,
        ]);
    }

    pub fn flush(&mut self) {
        if self.vertices == 0 {
            return;
        }

        self.vb.set_data(&self.vertex_data);
        self.eb.set_data(&self.index_data);
        // TODO-someday: maybe switch to draw_prepared, which requires more care to be taken with
        // safety but incurs less overhead
        // SAFETY: index data is only pushed to valid vertex indices above
        unsafe {
            self.shader
                .draw(
                    &self.vb,
                    &self.eb,
                    0..self.index_data.len(),
                    GeometryMode::Triangles,
                )
                .expect("flush to the GPU");
        }
        self.vertex_data.clear();
        self.index_data.clear();
        self.vertices = 0;
    }

    fn push_vertex(&mut self, x: f32, y: f32, color: Color, u: f32, v: f32) {
        self.vertex_data
            .extend_from_slice(&[color.r, color.g, color.b, color.a, x, y, u, v]);
        self.vertices += 1;
    }
}
