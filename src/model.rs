use glam::{Vec2, Vec4};

use crate::vert::Vert;

pub struct Model {
    pub verts: Vec<Vert>,
    pub indicies: Vec<u16>,
}

const BASE_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

impl Model {
    #[allow(dead_code)]
    pub fn square(size: f32) -> Self {
        let size = size.clamp(-1., 1.);

        let verts = vec![
            Vert::new(
                Vec4::new(size, size, 0.0, 1.0),
                BASE_COLOR,
                Vec2::new(1.0, 0.0),
            ),
            Vert::new(
                Vec4::new(size, -size, 0.0, 1.0),
                BASE_COLOR,
                Vec2::new(1.0, 1.0),
            ),
            Vert::new(
                Vec4::new(-size, -size, 0.0, 1.0),
                BASE_COLOR,
                Vec2::new(0.0, 1.0),
            ),
            Vert::new(
                Vec4::new(-size, size, 0.0, 1.0),
                BASE_COLOR,
                Vec2::new(0.0, 0.0),
            ),
        ];

        // &[0, 1, 3, 1, 2, 3] clockwise order
        let indicies: Vec<u16> = vec![3, 2, 1, 3, 1, 0];

        Self { verts, indicies }
    }
    #[allow(dead_code)]
    pub fn cube(size: f32) -> Self {
        let size = size.clamp(-1., 1.); // Clamp to NDC coordinates

        let verts: Vec<Vert> = vec![
            // Top
            Vert::new(
                Vec4::new(-size, -size, size, 1.),
                BASE_COLOR,
                Vec2::new(0., 0.),
            ),
            Vert::new(
                Vec4::new(size, -size, size, 1.),
                BASE_COLOR,
                Vec2::new(1., 0.),
            ),
            Vert::new(
                Vec4::new(size, size, size, 1.),
                BASE_COLOR,
                Vec2::new(1., 1.),
            ),
            Vert::new(
                Vec4::new(-size, size, size, 1.),
                BASE_COLOR,
                Vec2::new(0., 1.),
            ),
            // Bottom
            Vert::new(
                Vec4::new(-size, size, -size, 1.),
                BASE_COLOR,
                Vec2::new(1., 0.),
            ),
            Vert::new(
                Vec4::new(size, size, -size, 1.),
                BASE_COLOR,
                Vec2::new(0., 0.),
            ),
            Vert::new(
                Vec4::new(size, -size, -size, 1.),
                BASE_COLOR,
                Vec2::new(0., 1.),
            ),
            Vert::new(
                Vec4::new(-size, -size, -size, 1.),
                BASE_COLOR,
                Vec2::new(1., 1.),
            ),
            // Right
            Vert::new(
                Vec4::new(size, -size, -size, 1.),
                BASE_COLOR,
                Vec2::new(0., 0.),
            ),
            Vert::new(
                Vec4::new(size, size, -size, 1.),
                BASE_COLOR,
                Vec2::new(1., 0.),
            ),
            Vert::new(
                Vec4::new(size, size, size, 1.),
                BASE_COLOR,
                Vec2::new(1., 1.),
            ),
            Vert::new(
                Vec4::new(size, -size, size, 1.),
                BASE_COLOR,
                Vec2::new(0., 1.),
            ),
            // Left
            Vert::new(
                Vec4::new(-size, -size, size, 1.),
                BASE_COLOR,
                Vec2::new(1., 0.),
            ),
            Vert::new(
                Vec4::new(-size, size, size, 1.),
                BASE_COLOR,
                Vec2::new(0., 0.),
            ),
            Vert::new(
                Vec4::new(-size, size, -size, 1.),
                BASE_COLOR,
                Vec2::new(0., 1.),
            ),
            Vert::new(
                Vec4::new(-size, -size, -size, 1.),
                BASE_COLOR,
                Vec2::new(1., 1.),
            ),
            // Front
            Vert::new(
                Vec4::new(size, size, -size, 1.),
                BASE_COLOR,
                Vec2::new(1., 0.),
            ),
            Vert::new(
                Vec4::new(-size, size, -size, 1.),
                BASE_COLOR,
                Vec2::new(0., 0.),
            ),
            Vert::new(
                Vec4::new(-size, size, size, 1.),
                BASE_COLOR,
                Vec2::new(0., 1.),
            ),
            Vert::new(
                Vec4::new(size, size, size, 1.),
                BASE_COLOR,
                Vec2::new(1., 1.),
            ),
            // Back
            Vert::new(
                Vec4::new(size, -size, size, 1.),
                BASE_COLOR,
                Vec2::new(0., 0.),
            ),
            Vert::new(
                Vec4::new(-size, -size, size, 1.),
                BASE_COLOR,
                Vec2::new(1., 0.),
            ),
            Vert::new(
                Vec4::new(-size, -size, -size, 1.),
                BASE_COLOR,
                Vec2::new(1., 1.),
            ),
            Vert::new(
                Vec4::new(size, -size, -size, 1.),
                BASE_COLOR,
                Vec2::new(0., 1.),
            ),
        ];

        let indicies: Vec<u16> = vec![
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        Self { verts, indicies }
    }
}
