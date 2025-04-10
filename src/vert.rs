use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vert {
    // Unfortunately the Pod trait does not like
    // it when glam vecs of different sizes are together
    // most likely it is due to the way that library places the actual
    // floats into an 128 value
    pub pos: [f32; 4],        // Vec4
    pub color: [f32; 4],      // Vec4
    pub tex_coords: [f32; 2], // Vec2
}

impl Vert {
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

// Constants below
// TODO: Create functions to invalidate the
// use of these

// Square verts and indicies (Manually set)
pub const VERTS: &[Vert] = &[
    Vert {
        pos: [0.5, 0.5, 0.0, 1.0],
        color: [1.0, 0.0, 0.0, 1.0],
        tex_coords: [1.0, 0.0],
    },
    Vert {
        pos: [0.5, -0.5, 0.0, 1.0],
        color: [0.0, 0.0, 1.0, 1.0],
        tex_coords: [1.0, 1.0],
    },
    Vert {
        pos: [-0.5, -0.5, 0.0, 1.0],
        color: [0.0, 1.0, 0.0, 1.0],
        tex_coords: [0.0, 1.0],
    },
    Vert {
        pos: [-0.5, 0.5, 0.0, 1.0],
        color: [1.0, 1.0, 0.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
];

// &[0, 1, 3, 1, 2, 3] clockwise order
pub const INDICES: &[u16] = &[3, 2, 1, 3, 1, 0]; // Counterclockwise order
