use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::f32::consts;

use winit::keyboard::KeyCode;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_slice(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
]);

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

pub struct CameraController {
    pub speed: f32,
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub i_pressed: bool,
    pub k_pressed: bool,
    pub j_pressed: bool,
    pub l_pressed: bool,
    pub reset_pressed: bool,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Camera {
            // Alex, do not forget that this is right handed
            eye: Vec3::new(0.0, 0.0, 2.0), // wgpu tutorial sets these as 1.0, 2.0
            target: Vec3::ZERO,
            up: Vec3::Y, // Y-up unit vector
            aspect,
            fovy: consts::FRAC_PI_4,
            znear: 0.1,
            zfar: 100.0,
        }
    }
    pub fn reset_view(&mut self) {
        self.eye = Vec3::new(0.0, 0.0, 2.0);
        self.target = Vec3::ZERO;
    }
    pub fn view_proj_matrix(&self) -> Mat4 {
        // Right hand perspective
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);

        proj * view
    }
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = (OPENGL_TO_WGPU_MATRIX * camera.view_proj_matrix()).to_cols_array_2d();
    }

    pub fn bind_desc<'a>(&self) -> wgpu::BindGroupLayoutDescriptor<'a> {
        wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        }
    }
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            up_pressed: false,
            down_pressed: false,
            left_pressed: false,
            right_pressed: false,
            i_pressed: false,
            k_pressed: false,
            j_pressed: false,
            l_pressed: false,
            reset_pressed: false,
        }
    }

    pub fn process_events(&mut self, pressed: bool, keycode: KeyCode) -> bool {
        match keycode {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.up_pressed = pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.left_pressed = pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.down_pressed = pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.right_pressed = pressed;
                true
            }
            KeyCode::KeyI => {
                self.i_pressed = pressed;
                true
            }
            KeyCode::KeyJ => {
                self.j_pressed = pressed;
                true
            }
            KeyCode::KeyK => {
                self.k_pressed = pressed;
                true
            }
            KeyCode::KeyL => {
                self.l_pressed = pressed;
                true
            }
            KeyCode::KeyR => {
                self.reset_pressed = pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        if self.reset_pressed {
            camera.reset_view();
        }

        // Forward direction related vectors
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.length();

        // Forward backward movement
        if self.up_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.down_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        // Other directions
        let right = forward_norm.cross(camera.up).normalize();
        let up = forward_norm.cross(right).normalize();

        // Left and right panning
        // Right now only moves the "eye" directly
        // TODO: Make it so that it moves relative to the target, similar to moving forward
        if self.right_pressed {
            camera.eye.x += self.speed;
            camera.target.x += self.speed;
        }

        if self.left_pressed {
            camera.eye.x -= self.speed;
            camera.target.x -= self.speed;
        }

        // Get the forward direction and magnitude again
        let forward = camera.target - camera.eye;
        let forward_mag = forward.length();

        // Left right rotation
        if self.j_pressed {
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }

        if self.l_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }

        // Up down rotation
        if self.i_pressed {
            camera.eye = camera.target - (forward + up * self.speed).normalize() * forward_mag;
        }

        if self.k_pressed {
            camera.eye = camera.target - (forward - up * self.speed).normalize() * forward_mag;
        }
    }
}
