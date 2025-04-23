use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vert {
    // Unfortunately the Pod trait does not like
    // it when glam vecs of different sizes are together
    // most likely this is due to the way that library places the actual
    // floats into an 128 value
    pub pos: [f32; 4],        // Vec4
    pub color: [f32; 4],      // Vec4
    pub tex_coords: [f32; 2], // Vec2
}

impl Vert {
    pub fn new<U: Into<[f32; 4]>, V: Into<[f32; 2]>>(pos: U, color: U, uv: V) -> Self {
        Self {
            pos: pos.into(),
            color: color.into(),
            tex_coords: uv.into(),
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        // Return a buffer layout describing our verticies
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vert>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Vert color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Texture UV
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
