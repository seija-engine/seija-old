use nalgebra::{Matrix4};
use specs::{Component,HashMapStorage,Entity};

#[derive(Debug, Copy, Clone)]
pub struct Orthographic {
    matrix: Matrix4<f32>,
    inverse_matrix: Matrix4<f32>,
}

impl PartialEq for Orthographic {
    fn eq(&self,other:&Self) -> bool {
        self.matrix == other.matrix

    }
}

impl Orthographic {
    pub fn new(left:f32,right:f32, bottom: f32, top: f32, z_near: f32, z_far: f32) -> Self {
        let mut matrix = Matrix4::<f32>::identity();
        matrix[(0, 0)] = 2.0 / (right - left);
        matrix[(1, 1)] = -2.0 / (top - bottom);
        matrix[(2, 2)] = 1.0 / (z_far - z_near);
        matrix[(0, 3)] = -(right + left) / (right - left);
        matrix[(1, 3)] = -(top + bottom) / (top - bottom);
        matrix[(2, 3)] = -z_near / (z_far - z_near);
        Self {
            matrix,
            inverse_matrix: matrix
                .try_inverse()
                .expect("Camera projection matrix is not invertible. This is normally due to having inverse values being superimposed (near=far, right=left)"),
        }
    }

    #[inline]
    pub fn as_matrix(&self) -> &Matrix4<f32> {
        &self.matrix
    }

    #[inline]
    pub fn as_inverse_matrix(&self) -> &Matrix4<f32> {
        &self.inverse_matrix
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Projection {
    Orthographic(Orthographic)
}

impl Projection {
    pub fn orthographic(left: f32,right: f32,bottom: f32,top: f32,z_near: f32,z_far: f32) -> Projection {
        Projection::Orthographic(Orthographic::new(left, right, bottom, top, z_near, z_far))
    }

    pub fn as_orthographic(&self) -> Option<&Orthographic> {
        match *self {
            Projection::Orthographic(ref s) => Some(s),
        }
    }

    pub fn as_orthographic_mut(&mut self) -> Option<&mut Orthographic> {
        match *self {
            Projection::Orthographic(ref mut s) => Some(s)
        }
    }

    pub fn as_matrix(&self) -> &Matrix4<f32> {
        match *self {
            Projection::Orthographic(ref s) => s.as_matrix()
        }
    }

    pub fn as_inverse_matrix(&self) -> &Matrix4<f32> {
        match *self {
            Projection::Orthographic(ref s) => s.as_inverse_matrix()
        }
    }
}

impl From<Orthographic> for Projection {
    fn from(proj: Orthographic) -> Self {
        Projection::Orthographic(proj)
    }
}

impl From<Projection> for Camera {
    fn from(proj: Projection) -> Self {
        Camera { inner: proj }
    }
}



#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    inner: Projection,
}

impl Camera {
    pub fn standard_2d(width: f32, height: f32) -> Self {
        Self::from(Projection::orthographic(-width / 2.0,width / 2.0,-height / 2.0,height / 2.0,-2000.0,2000.0))
    }

    pub fn as_matrix(&self) -> &Matrix4<f32> {
        match self.inner {
            Projection::Orthographic(ref p) => p.as_matrix()
        }
    }

    
    pub fn as_inverse_matrix(&self) -> &Matrix4<f32> {
        match self.inner {
            Projection::Orthographic(ref p) => p.as_inverse_matrix()
        }
    }

    pub fn projection(&self) -> &Projection {
        &self.inner
    }


    pub fn projection_mut(&mut self) -> &mut Projection {
        &mut self.inner
    }
}

impl Component for Camera {
    type Storage = HashMapStorage<Self>;
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ActiveCamera {
    /// Camera entity
    pub entity: Option<Entity>,
}