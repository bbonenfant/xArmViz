use cgmath::Matrix4;
use super::{Projection, View};

/// This is a matrix used to convert a ViewProjection matrix in OpenGL format
///   to one that is in WGPU format.
#[cfg_attr(rustfmt, rustfmt_skip)]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);


/// Representation of the 3D Camera.
#[derive(Clone, Copy)]
pub struct Camera {
    projection: Projection,
    view: View,
}

impl Camera {

    /// Create a new Camera object from a View and Projection.
    pub fn new(view: View, projection: Projection) -> Self {
        return Camera { view, projection }
    }

    /// Get a reference to the Projection object.
    pub fn get_projection(&self) -> &Projection { &self.projection }

    /// Get a reference to the View object.
    pub fn get_view(&self) -> &View { &self.view }

    /// Set the View object.
    pub fn set_view(&mut self, view: View) { self.view = view; }

    /// Build the View-Projection matrix describing the current Camera.
    ///   Uses the WGPU format -- NOT the OpenGL format.
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        return OPENGL_TO_WGPU_MATRIX * self.projection.as_matrix() * self.view.as_matrix();
    }
}
