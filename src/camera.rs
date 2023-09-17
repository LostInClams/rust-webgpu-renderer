use cgmath;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl  CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_projection(&mut self, camera: &Camera) {
        self.view_proj = camera.generate_view_projection_matrix().into();
    }
}

pub struct Camera {
    pub position: cgmath::Point3<f32>,
    pub forward: cgmath::Vector3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect_ratio: f32,
    pub fov_vertical: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn generate_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // TODO: Cache matrix calculation
        let view = cgmath::Matrix4::look_at_rh(self.position, self.position + self.forward, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fov_vertical), self.aspect_ratio, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}
