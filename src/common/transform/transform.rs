use nalgebra::{Isometry3,Vector3,Matrix4,Translation3,UnitQuaternion};
use nalgebra as na;

#[derive(Clone,Debug,PartialEq)]
pub struct Transform {
    scale:Vector3<f32>,
    isometry: Isometry3<f32>,

    pub global_matrix: Matrix4<f32>,
}

impl Transform {
    pub fn new(position: Translation3<f32>,rotation: UnitQuaternion<f32>,scale: Vector3<f32>) -> Self {
        Transform {
            isometry: Isometry3::from_parts(na::convert(position), na::convert(rotation)),
            scale: scale,
            global_matrix: na::one(),
        }
    }

    pub fn set_scale(&mut self,scale:Vector3<f32>) {
        self.scale = scale;
    }

    pub fn scale(&self) -> &Vector3<f32> {
        &self.scale
    }

    pub fn scale_mut(&mut self) -> &mut Vector3<f32> {
        &mut self.scale
    }

    pub fn global_matrix(&self) -> &Matrix4<f32> {
        &self.global_matrix
    }

    #[inline]
    pub fn matrix(&self) -> Matrix4<f32> {
        self.isometry.to_homogeneous().prepend_nonuniform_scaling(&self.scale)
    }

    #[inline]
    pub fn rotation(&self) -> &UnitQuaternion<f32> {
        &self.isometry.rotation
    }

    pub fn set_rotation(&mut self,rotation:UnitQuaternion<f32>)  {
        self.isometry.rotation = rotation;
    }

    pub fn set_rotation_euler(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.isometry.rotation = UnitQuaternion::from_euler_angles(x, y, z);
        self
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        let inv_scale = Vector3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
        self.isometry.inverse().to_homogeneous().append_nonuniform_scaling(&inv_scale)
    }

    pub fn global_view_matrix(&self) -> Matrix4<f32> {
        let mut res = self.global_matrix;
        {
            let mut slice3x3 = res.fixed_slice_mut::<na::U3, na::U3>(0, 0);
            assert!(slice3x3.try_inverse_mut());
        }
        let mut translation = -res.column(3).xyz();
        translation = res.fixed_slice::<na::U3, na::U3>(0, 0) * translation;
        let mut res_trans = res.column_mut(3);
        res_trans.x = translation.x;
        res_trans.y = translation.y;
        res_trans.z = translation.z;
        res
    }

    #[inline]
    pub fn set_position_z(&mut self, value: f32) -> &mut Self {
        self.isometry.translation.vector.z = value;
        self
    }

    pub fn set_position_x(&mut self,value: f32)  {
        self.isometry.translation.vector.x = value;
    }

    pub fn set_position_y(&mut self,value: f32)  {
        self.isometry.translation.vector.y = value;
    }

    pub fn set_position(&mut self,position: Vector3<f32>) -> &mut Self {
        self.isometry.translation.vector = position;
        self
    }

    
    pub fn position(&self) -> &Vector3<f32> {
        &self.isometry.translation.vector
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            isometry: Isometry3::identity(),
            scale: Vector3::from_element(1.0),
            global_matrix: na::one(),
        }
    }
}