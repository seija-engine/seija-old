use crate::render::{SpriteMesh};
use specs::{Component,DenseVecStorage};

pub struct Mesh2D {
    pub mesh:Option<SpriteMesh>,
    pub is_dirty:bool,
}

impl Default for Mesh2D {
    fn default() -> Self {
        Mesh2D {
            mesh:None,
            is_dirty:true
        }
    }
}

impl Mesh2D {
    pub fn new(sprite_mesh:SpriteMesh) -> Self {
        let mut ret_mesh = Mesh2D::default();
        ret_mesh.mesh = Some(sprite_mesh);
        ret_mesh
    }
}

impl Component for Mesh2D {
    type Storage = DenseVecStorage<Self>;
}